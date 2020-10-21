#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

// TODO:

// 10 ms logic.
// Search for and remove insances of .ok() - Create struct var with output, determine health.
// Validate checksums
// Set latch status
// J1772 AC charging logic
// Fix comments
// Handle power-on with cable in place (sticks in Charge Port Error)
// Learn how to handle interrupt sources
// RTC setup - look at existing driver and compare
// IVT-S Read Voltage
// SimpBMS Voltage
// EEPROM Settings
// Handle door stuff better.

extern crate cortex_m;
extern crate panic_halt;

use cortex_m_rt::entry;
extern crate stm32f7xx_hal as hal;
use hal::{
    interrupt, pac,
    prelude::*,
    serial::{self, Serial},
    timer::{Event, Timer},
};

// CP ECU Signal Input
use hal::gpio::gpiog::PG2;
use hal::gpio::{Edge, ExtiPin, Floating, Input};

// Elapsed_MS stuff...
use core::cell::{Cell, RefCell};
use core::ops::DerefMut;
use cortex_m::interrupt::{free, Mutex};

// CAN
use hal::can::Can;
use hal::can::CanBitTiming;
use hal::can::CanConfig;
use hal::can::CanFilterConfig;
use hal::can::RxFifo;

// Aliases
use cpcontrol::can_receive_logic::init as can_receive_logic;
use cpcontrol::fifty_ms_loop::init as fifty_ms_loop;
use cpcontrol::five_hundred_ms_loop::init as five_hundred_ms_loop;
use cpcontrol::hundred_ms_loop::init as hundred_ms_loop;
use cpcontrol::process_serial::init as process_serial;
use cpcontrol::ten_ms_loop::init as ten_ms_loop;
use cpcontrol::thousand_ms_loop::init as thousand_ms_loop;
use cpcontrol::two_fifty_ms_loop::init as two_fifty_ms_loop;
use cpcontrol::types::*;

// Random Rust notes:
// fn main() { needs to be fn main() -> ! to show it will never return.
// fn main() { << Standard exampel
// References to |cs| refer to critical (non-preemptible) sections

// Dragons ahead.
static ELAPSED_MS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0u32));
static TIMER_TIM2: Mutex<RefCell<Option<Timer<pac::TIM2>>>> = Mutex::new(RefCell::new(None));

// Another Semaphore / Mutex for use with the Fault Line input
static SEMAPHORE: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
static FAULT_LINE: Mutex<RefCell<Option<PG2<Input<Floating>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Unwrap the PAC crate.
    let mut p = pac::Peripherals::take().unwrap();
    let mut syscfg = p.SYSCFG;
    let mut exti = p.EXTI;

    // GPIO G for Fault and Latch I/O (PG2 for Fault (Read), and PG3 for Latch (Push-Pull High
    // output)).
    let gpiog = p.GPIOG.split();
    let mut fault_in = gpiog.pg2.into_floating_input();
    fault_in.make_interrupt_source(&mut syscfg, &mut p.RCC);
    fault_in.trigger_on_edge(&mut exti, Edge::RISING_FALLING);
    fault_in.enable_interrupt(&mut exti);
    let mut latch_out = gpiog.pg3.into_push_pull_output();

    // GPIO D for CAN (PD0,1) and USART3 (PD8,9)
    let gpiod = p.GPIOD.split();

    // Freeze RCC and System Clocks *After* setting EXTI items.
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.mhz()).freeze();

    // AF7 -> Alternate Function 7 -> USART for PD8/9.
    // This is totally board specific, and need to figure out
    // how to make this more generic.
    let tx_pin = gpiod.pd8.into_alternate_af7();
    let rx_pin = gpiod.pd9.into_alternate_af7();

    let serial = Serial::new(
        p.USART3,
        (tx_pin, rx_pin),
        clocks,
        serial::Config {
            baud_rate: 230_400.bps(),
            oversampling: serial::Oversampling::By16,
            character_match: None,
        },
    );
    let mut timer = Timer::tim2(p.TIM2, 1.khz(), clocks, &mut rcc.apb1);
    timer.listen(Event::TimeOut);

    // Configure interrupt related items
    free(|cs| {
        TIMER_TIM2.borrow(cs).replace(Some(timer));
    });
    free(|cs| {
        FAULT_LINE.borrow(cs).replace(Some(fault_in));
    });
    // Enable interrupts
    cpcontrol::hardware_init::init();

    let (mut tx, mut rx) = serial.split();

    const TEN_MS: u32 = 10;
    let mut previous_10_ms_ts = 0;
    let mut ten_ms_counter: u16 = 0;

    const FIFTY_MS: u32 = 50;
    let mut previous_50_ms_ts = 0;
    let mut fifty_ms_counter: u8 = 0;

    const HUNDRED_MS: u32 = 100;
    let mut previous_100_ms_ts = 0;
    let mut hundred_ms_counter: u8 = 0;

    const TWO_FIFTY_MS: u32 = 250;
    let mut previous_250_ms_ts = 0;
    let mut two_fifty_ms_counter: u8 = 0;

    const FIVE_HUNDRED_MS: u32 = 500;
    let mut previous_500_ms_ts = 0;
    let mut five_hundred_ms_counter: u8 = 0;

    const THOUSAND_MS: u32 = 1000;
    let mut previous_1000_ms_ts = 0;
    let mut thousand_ms_counter: u8 = 0;

    // Set up CAN shit.
    pub const CONTROL_CAN_CONFIG: CanConfig = CanConfig {
        loopback_mode: false,
        silent_mode: false,
        ttcm: false,
        abom: true,
        awum: false,
        nart: false,
        rflm: false,
        txfp: false,
        // TODO - update CAN impl to calculate these
        /// Control CAN bus is configured for 500K
        bit_timing: CanBitTiming {
            prescaler: 5, // 6 (6, 1, 15, 2 -> 5, 0, 14 , 1)   --- 48 is (6 , 1, 13, 2 -> 5, 0, 12,1)
            sjw: 0,       // CAN_SJW_1TQ
            bs1: 14,      // CAN_BS1_15TQ
            bs2: 1,       // CAN_BS2_2TQ
        },
    };

    let can1_tx = gpiod.pd1.into_alternate_af9();
    let can1_rx = gpiod.pd0.into_alternate_af9();

    let hv_can = Can::can1(
        p.CAN1,
        (can1_tx, can1_rx),
        &mut rcc.apb1,
        &CONTROL_CAN_CONFIG,
    )
    .expect("Failed to configure control CAN (CAN1)");
    let can_filter: CanFilterConfig = CanFilterConfig::default();
    hv_can.configure_filter(&can_filter).ok();

    // Create the status structure
    let mut cp_state = CPState::new();
    // Status queue things
    // Too many of these items slows down serial console, which slows down
    // all of the loops.

    // Test output
    latch_out.set_high().ok();
    // Main control loop here.
    // Process serial input
    // Run X ms loops (10, 100, 1000)
    loop {
        let elapsed = free(|cs| ELAPSED_MS.borrow(cs).get());

        // Highly interactive pieces:
        // CAN reception
        for fifo in &[RxFifo::Fifo0, RxFifo::Fifo1] {
            if let Ok(rx_frame) = hv_can.receive(fifo) {
                cp_state = can_receive_logic(&rx_frame, elapsed, cp_state);
            }
        }

        // Serial input
        if let Ok(received) = rx.read() {
            cp_state = process_serial(received, elapsed, cp_state);
        }

        // 10 ms - Done
        if (elapsed - previous_10_ms_ts) >= TEN_MS {
            previous_10_ms_ts = elapsed;
            ten_ms_counter = ten_ms_loop(&mut tx, &mut cp_state, elapsed, ten_ms_counter, &hv_can);
            // Once run, flip it off.
            if cp_state.quiet_to_verbose {
                cp_state.quiet_to_verbose = false;
            }
            if cp_state.print_menu_request {
                cp_state.print_menu_request = false;
            }
        }

        // 50 ms - Done
        if (elapsed - previous_50_ms_ts) >= FIFTY_MS {
            previous_50_ms_ts = elapsed;
            fifty_ms_counter = fifty_ms_loop(fifty_ms_counter, &hv_can);
        }

        // 100 ms - Done
        if (elapsed - previous_100_ms_ts) >= HUNDRED_MS {
            previous_100_ms_ts = elapsed;
            hundred_ms_counter = hundred_ms_loop(hundred_ms_counter, &mut cp_state, &hv_can);
        }

        // 250 ms - Done
        if (elapsed - previous_250_ms_ts) >= TWO_FIFTY_MS {
            previous_250_ms_ts = elapsed;
            two_fifty_ms_counter = two_fifty_ms_loop(two_fifty_ms_counter, &hv_can);
        }

        // 500 ms - Done
        if (elapsed - previous_500_ms_ts) >= FIVE_HUNDRED_MS {
            previous_500_ms_ts = elapsed;
            five_hundred_ms_counter =
                five_hundred_ms_loop(five_hundred_ms_counter, &mut cp_state, &hv_can);
        }

        // 1000 ms - Done
        if (elapsed - previous_1000_ms_ts) >= THOUSAND_MS {
            previous_1000_ms_ts = elapsed;
            thousand_ms_counter =
                thousand_ms_loop(thousand_ms_counter, &mut cp_state, &hv_can, &SEMAPHORE);
        }
    }
}

#[interrupt]
fn TIM2() {
    free(|cs| {
        if let Some(ref mut tim2) = TIMER_TIM2.borrow(cs).borrow_mut().deref_mut() {
            tim2.clear_interrupt(Event::TimeOut);
        }

        let cell = ELAPSED_MS.borrow(cs);
        let val = cell.get();
        cell.replace(val + 1);
    });
}

#[interrupt]
fn EXTI2() {
    // This is going to fire for all pins associated with this interrupt, which is going to be all
    // of them ending in 2 - PA2,PB2,...PG2, etc. So avoid using any more pins with the end in 2
    // until it is known how to differentiate between them.
    free(|cs| {
        match FAULT_LINE.borrow(cs).borrow_mut().as_mut() {
            // Clear the push button interrupt
            Some(b) => {
                b.clear_interrupt_pending_bit();
                if b.is_high().unwrap_or(false) {
                    SEMAPHORE.borrow(cs).set(true);
                } else {
                    SEMAPHORE.borrow(cs).set(false);
                }
            }

            // This should never happen
            None => (),
        }
    });
}
