//#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

// TODO:

// 10 ms logic.
// Search for and remove insances of .ok() - Create struct var with output, determine health.
// J1772 AC charging logic
// Handle power-on with cable in place (sticks in Charge Port Error)
// Learn how to handle interrupt sources
// RTC setup - look at existing driver and compare - Found stm32f3xx-hal example.
// IVT-S Read Voltage
// SimpBMS Voltage
// EEPROM Settings
// Handle door stuff better.

extern crate cortex_m;
extern crate panic_halt;

use chrono::NaiveTime;

#[cfg(feature = "nucleo767zi")]
use chrono::NaiveDate;

use cortex_m_rt::entry;

#[cfg(feature = "nucleo767zi")]
extern crate stm32f7xx_hal as hal;
#[cfg(feature = "nucleo767zi")]
use hal::rtc::Rtc;
#[cfg(feature = "nucleo767zi")]
use hal::serial;
#[cfg(feature = "nucleo767zi")]
use rtcc::Rtcc;

#[cfg(feature = "f4board")]
extern crate stm32f4xx_hal as hal;
#[cfg(feature = "f4board")]
use hal::serial::config::Config;

use hal::{
    interrupt, pac,
    prelude::*,
    serial::Serial,
    timer::{Event, Timer},
};

// CP ECU Signal Input
use hal::gpio::{Edge, ExtiPin};

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
static FAULT_LINE: Mutex<RefCell<Option<FaultLinePin>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Unwrap the PAC crate.
    // For F7, this needs to be writable. For F4 not.
    #[cfg(feature = "nucleo767zi")]
    let mut p = pac::Peripherals::take().unwrap();
    #[cfg(feature = "f4board")]
    let p = pac::Peripherals::take().unwrap();
    let mut syscfg = p.SYSCFG;
    let mut exti = p.EXTI;

    // GPIO G for Fault and Latch I/O (PG2 for Fault (Read), and PG3 for Latch (Push-Pull High
    // output)).
    #[cfg(feature = "nucleo767zi")]
    let gpiog = p.GPIOG.split();
    #[cfg(feature = "nucleo767zi")]
    let mut fault_in = gpiog.pg2.into_floating_input();
    #[cfg(feature = "nucleo767zi")]
    fault_in.make_interrupt_source(&mut syscfg, &mut p.RCC);
    #[cfg(feature = "nucleo767zi")]
    let mut latch_out = gpiog.pg3.into_push_pull_output();

    // GPIO B for Fault and Latch I/O (PB3 for Fault (Read), and PB5 for Latch (Push-Pull High
    // output)). Also CAN Bus 1.
    #[cfg(feature = "f4board")]
    let gpiob = p.GPIOB.split();
    #[cfg(feature = "f4board")]
    let mut fault_in = gpiob.pb3.into_floating_input();

    #[cfg(feature = "f4board")]
    fault_in.make_interrupt_source(&mut syscfg);
    #[cfg(feature = "f4board")]
    let mut latch_out = gpiob.pb5.into_push_pull_output();

    // Set trigger and enable interrupt on all boards.
    fault_in.trigger_on_edge(&mut exti, Edge::RISING_FALLING);
    fault_in.enable_interrupt(&mut exti);

    // GPIO D for CAN (PD0,1) and USART3 (PD8,9) on STM32F767
    #[cfg(feature = "nucleo767zi")]
    let gpiod = p.GPIOD.split();

    // Freeze RCC and System Clocks *After* setting EXTI items.
    // Run both boards at 180 as we don't need the extra 36MHz speed.
    let mut rcc = p.RCC.constrain();
    #[cfg(feature = "nucleo767zi")]
    let clocks = rcc.cfgr.sysclk(180.mhz()).freeze();
    #[cfg(feature = "f4board")]
    let clocks = rcc.cfgr.sysclk(180.mhz()).freeze();

    // RTC?
    /*
    #[cfg(feature = "nucleo767zi")]
    let mut rtc = Rtc::new(
        p.RTC,
        255,
        127,
        true,
        &mut rcc.apb1,
        &mut rcc.bdcr,
        &mut p.PWR,
    );
    #[cfg(feature = "nucleo767zi")]
    let datetime = NaiveDate::from_ymd(2018, 8, 20).and_hms(19, 59, 58);
    #[cfg(feature = "nucleo767zi")]
    rtc.set_datetime(&datetime).unwrap();
    */
    // AF7 -> Alternate Function 7 -> USART for PD8/9.
    // This is totally board specific, and need to figure out
    // how to make this more generic.
    #[cfg(feature = "nucleo767zi")]
    let tx_pin = gpiod.pd8.into_alternate_af7();
    #[cfg(feature = "nucleo767zi")]
    let rx_pin = gpiod.pd9.into_alternate_af7();

    // Nucleo F767 Serial
    #[cfg(feature = "nucleo767zi")]
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
    #[cfg(feature = "f4board")]
    let gpioa = p.GPIOA.split();
    #[cfg(feature = "f4board")]
    let tx_pin = gpioa.pa2.into_alternate_af7();
    #[cfg(feature = "f4board")]
    let rx_pin = gpioa.pa3.into_alternate_af7();
    #[cfg(feature = "f4board")]
    let serial = Serial::usart2(
        p.USART2,
        (tx_pin, rx_pin),
        Config::default().baudrate(230400.bps()),
        clocks,
    )
    .unwrap();

    // 1000 Hz Timer (1ms)
    #[cfg(feature = "nucleo767zi")]
    let mut timer = Timer::tim2(p.TIM2, 1.khz(), clocks, &mut rcc.apb1);

    #[cfg(feature = "f4board")]
    let mut timer = Timer::tim2(p.TIM2, 1.khz(), clocks);

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
    // Bit Timing for 180MHz System Clock
    // (45MHz APB1)
    #[cfg(feature = "f4board")]
    const BIT_TIMING: CanBitTiming = CanBitTiming {
        prescaler: 4, // 5
        sjw: 0,       // CAN_SJW_1TQ
        bs1: 14,      // CAN_BS1_15TQ
        bs2: 1,       // CAN_BS2_2TQ
    };
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
        // HV CAN bus is configured for 500K
        bit_timing: BIT_TIMING,
    };

    #[cfg(feature = "nucleo767zi")]
    let can1_tx = gpiod.pd1.into_alternate_af9();
    #[cfg(feature = "nucleo767zi")]
    let can1_rx = gpiod.pd0.into_alternate_af9();

    #[cfg(feature = "f4board")]
    let can1_tx = gpiob.pb9.into_alternate_af9();
    #[cfg(feature = "f4board")]
    let can1_rx = gpiob.pb8.into_alternate_af9();

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

    // Main control loop here.
    // Process serial input
    // Run X ms loops (10, 100, 1000)


    let time = NaiveTime::from_hms(19, 59, 58);
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
            ten_ms_counter = ten_ms_loop(
                &mut tx,
                &mut cp_state,
                elapsed,
                ten_ms_counter,
                &hv_can,
                time,
            );
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
            if !cp_state.cp_init || cp_state.charger_relay_enabled {
                // If the relay is enabled, this value should be low.
                // If the fault line is active (and therefore cp_init is false), this should be
                // low.
                latch_out.set_low().ok();
            } else {
                latch_out.set_high().ok();
            }


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

#[cfg(feature = "nucleo767zi")]
#[interrupt]
fn EXTI2() {
    // This is going to fire for all pins associated with this interrupt, which is going to be all
    // of them ending in 2 - PA2,PB2,...PG2, etc. So avoid using any more pins with the end in 2
    // until it is known how to differentiate between them.
    // Answer: "using EXTI_PR you have to detect which pin generated interrupt"
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

#[cfg(feature = "f4board")]
#[interrupt]
fn EXTI3() {
    // This is going to fire for all pins associated with this interrupt, which is going to be all
    // of them ending in 3 - PA3,PB3,...PG2, etc. So avoid using any more pins with the end in 2
    // until it is known how to differentiate between them.
    // Answer: "using EXTI_PR you have to detect which pin generated interrupt"
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
