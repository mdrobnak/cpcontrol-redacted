#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

// TODO:

// Validate that serial doesn't mess with timing on new code, maybe move back to 115.2k?
// 10 ms logic.
// Search for and remove insances of .ok() - Create struct var with output, determine health.
// J1772 AC charging logic
// Handle power-on with cable in place (sticks in Charge Port Error)
// Learn how to handle interrupt sources
// RTC setup - Alarm code needs to be implemented, or using math
// IVT-S Read Voltage
// SimpBMS Voltage
// EEPROM Settings
// Handle door stuff better.
// Serial for Wifi

extern crate cortex_m;
extern crate panic_halt;

// Entrypoint
use cortex_m_rt::entry;

#[cfg(feature = "nucleof767zi")]
extern crate stm32f7xx_hal as hal;

#[cfg(any(
    feature = "nucleof446re",
    feature = "production",
    feature = "twentyfour",
))]
extern crate stm32f4xx_hal as hal;

// General HAL items
use hal::{
    interrupt, pac,
    prelude::*,
    timer::{Event, Timer},
};

// CP ECU Signal Input
// Used to clear the pending interrupt bit in the interrupt handler.
use hal::gpio::ExtiPin;

// Elapsed_MS stuff...
use core::cell::{Cell, RefCell};
use core::ops::DerefMut;
use cortex_m::interrupt::{free, Mutex};

// CAN
use hal::can::CanFilterConfig;
use hal::can::RxFifo;

// RTC
use chrono::NaiveDate;
use rtcc::Rtcc;

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
    // Hardware to initialize:
    // Fault Input
    // Latch Output
    // HV CAN Tx, Rx
    // Clocks
    // Serial port
    // RTC (No alarms yet)
    // TIM2 SysTick
    // TODO: Second CAN bus.

    let (fault_in, mut latch_out, hv_can, serial, timer, mut rtc) =
        cpcontrol::hardware_init::init_devices();

    // Interrupts / Mutexes
    free(|cs| {
        TIMER_TIM2.borrow(cs).replace(Some(timer));
    });
    free(|cs| {
        FAULT_LINE.borrow(cs).replace(Some(fault_in));
    });
    cpcontrol::hardware_init::enable_interrupts();

    let (mut tx, mut rx) = serial.split();

    let can_filter: CanFilterConfig = CanFilterConfig::default();
    hv_can.configure_filter(&can_filter).ok();

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

    let datetime = NaiveDate::from_ymd(2020, 10, 25).and_hms(23, 59, 45);
    rtc.set_datetime(&datetime).unwrap();
    // Create the status structure
    let mut cp_state = CPState::new();
    // This is pretty dumb, but, eh.
    let mut rtc_data = RTCUpdate::new();
    // Status queue things
    // Too many of these items slows down serial console, which slows down
    // all of the loops.

    // Main control loop here.
    // Process serial input
    // Run X ms loops (10, 100, 1000)

    loop {
        let elapsed = free(|cs| ELAPSED_MS.borrow(cs).get());
        let time = rtc.get_datetime().unwrap();

        // Highly interactive pieces:
        // CAN reception
        for fifo in &[RxFifo::Fifo0, RxFifo::Fifo1] {
            if let Ok(rx_frame) = hv_can.receive(fifo) {
                can_receive_logic(&rx_frame, elapsed, &mut cp_state);
            }
        }

        // Serial input (and some output) - BUT - only gets called when there is input!
        if let Ok(received) = rx.read() {
            cp_state = process_serial(
                received,
                elapsed,
                cp_state,
                &mut tx,
                &mut rtc,
                &mut rtc_data,
            );
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
                &mut rtc,
                &rtc_data,
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
                //            if !cp_state.latch_enabled {
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

#[cfg(feature = "nucleof767zi")]
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

#[cfg(any(
    feature = "nucleof446re",
    feature = "production",
    feature = "twentyfour",
))]
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
