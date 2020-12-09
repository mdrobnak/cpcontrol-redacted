#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

// TODO:

// Validate that serial doesn't mess with timing on new code, maybe move back to 115.2k?
// 10 ms logic.
// Search for and remove insances of .ok() - Create struct var with output, determine health.
// Handle power-on with cable in place (sticks in Charge Port Error)
// Learn how to handle interrupt sources
// RTC setup - Alarm code needs to be implemented, or using math
// IVT-S Read Voltage
// SimpBMS Voltage
// EEPROM Settings
// Serial for Wifi

extern crate panic_halt;

#[cfg(feature = "nucleof767zi")]
extern crate stm32f7xx_hal as hal;

#[cfg(any(
    feature = "nucleof446re",
    feature = "production",
    feature = "twentyfour",
))]
extern crate stm32f4xx_hal as hal;

// General HAL items
use hal::prelude::*;

// RTIC related for monotonic counter.
use cortex_m::peripheral::DWT;
use rtic::cyccnt::U32Ext as _;

// CP ECU Signal Input
// Used to clear the pending interrupt bit in the interrupt handler.
use core::cell::{Cell, RefCell};
use cortex_m::interrupt::{free, Mutex};
use hal::gpio::ExtiPin;

// CAN
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
use cpcontrol::sched;
use cpcontrol::serial_console::serial_console;
use cpcontrol::ten_ms_loop::init as ten_ms_loop;
use cpcontrol::thousand_ms_loop::init as thousand_ms_loop;
use cpcontrol::two_fifty_ms_loop::init as two_fifty_ms_loop;
use cpcontrol::types::*;

// Semaphore / Mutex for use with the Fault Line input
static SEMAPHORE: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
static FAULT_LINE: Mutex<RefCell<Option<FaultLinePin>>> = Mutex::new(RefCell::new(None));

#[rtic::app(device = hal::pac, monotonic = rtic::cyccnt::CYCCNT, peripherals = true)]
const APP: () = {
    struct Resources {
        tx: SerialConsoleOutput,
        rx: SerialConsoleInput,
        cp_state: CPState,
        hv_can: HVCAN,
        latch_out: LatchOutPin,
        rtc: Rtc,
        rtc_data: RTCUpdate,
        elapsed: u32,
        ten_ms_counter: u16,
        fifty_ms_counter: u8,
        hundred_ms_counter: u8,
        five_hundred_ms_counter: u8,
        thousand_ms_counter: u8,
        two_fifty_ms_counter: u8,
    }
    #[init(schedule = [half_ms,one_ms,ten_ms,fifty_ms,hundred_ms,two_fifty_ms,five_hundred_ms,thousand_ms])]
    fn init(mut cx: init::Context) -> init::LateResources {
        // Hardware to initialize:
        // Fault Input
        // Latch Output
        // HV CAN Tx, Rx
        // Clocks
        // Serial port
        // RTC (No alarms yet)
        // TIM2 SysTick - SKIP
        // TODO: Second CAN bus.

        let (fault_in, latch_out, hv_can, mut serial, mut rtc, clock) =
            cpcontrol::hardware_init::init_devices(cx.device);

        serial.listen(hal::serial::Event::Rxne);

        free(|cs| {
            FAULT_LINE.borrow(cs).replace(Some(fault_in));
        });

        let (tx, rx) = serial.split();

        // Initialize (enable) the monotonic timer (CYCCNT)
        cx.core.DCB.enable_trace();
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        DWT::unlock();
        cx.core.DWT.enable_cycle_counter();

        let now = cx.start;

        // Compute intervals (in clock cycles) and initialize counters
        let half_ms = (clock * 0.001 * 0.5) as u32;

        let one_ms = (clock * 0.001) as u32;
        let elapsed: u32 = 0;

        let ten_ms = (clock * 0.01) as u32;
        let ten_ms_counter: u16 = 0;

        let fifty_ms = (clock * 0.05) as u32;
        let fifty_ms_counter: u8 = 0;

        let hundred_ms = (clock * 0.1) as u32;
        let hundred_ms_counter: u8 = 0;

        let two_fifty_ms = (clock * 0.25) as u32;
        let two_fifty_ms_counter: u8 = 0;

        let five_hundred_ms = (clock * 0.5) as u32;
        let five_hundred_ms_counter: u8 = 0;

        let thousand_ms = (clock * 1.00) as u32;
        let thousand_ms_counter: u8 = 0;

        // Macro takes in input and schedules at a given interval.
        sched!(
            cx,
            now,
            [
                half_ms,
                one_ms,
                ten_ms,
                fifty_ms,
                hundred_ms,
                two_fifty_ms,
                five_hundred_ms,
                thousand_ms
            ]
        );

        // Set initial date for RTC
        let datetime = NaiveDate::from_ymd(2020, 12, 02).and_hms(09, 20, 00);
        rtc.set_datetime(&datetime).unwrap();
        // Create the status structure
        let cp_state = CPState::new();
        // This is pretty dumb, but, eh.
        let rtc_data = RTCUpdate::new();
        // Status queue things

        init::LateResources {
            cp_state,
            rtc,
            rtc_data,
            rx,
            tx,
            hv_can,
            latch_out,
            elapsed,
            ten_ms_counter,
            fifty_ms_counter,
            five_hundred_ms_counter,
            hundred_ms_counter,
            thousand_ms_counter,
            two_fifty_ms_counter,
        }
    }

    #[idle(resources = [tx,cp_state,rtc,rtc_data,ten_ms_counter,elapsed])]
    fn idle(mut ctx: idle::Context) -> ! {
        loop {
            // Double lock achieved by splitting out the resources,
            // don't try and borrow all of ctx.
            let rtc = &mut ctx.resources.rtc;
            let tx = &mut ctx.resources.tx;
            let cp_state = &mut ctx.resources.cp_state;
            let rtc_data = &mut ctx.resources.rtc_data;
            let ten_ms_counter = &mut ctx.resources.ten_ms_counter;
            let elapsed = &mut ctx.resources.elapsed;
            elapsed.lock(|elapsed| {
                ten_ms_counter.lock(|ten_ms_counter| {
                    rtc_data.lock(|rtc_data| {
                        rtc.lock(|mut rtc| {
                            tx.lock(|mut tx| {
                                cp_state.lock(|cp| {
                                    let time = rtc.get_datetime().unwrap();
                                    serial_console(
                                        &mut tx,
                                        &cp,
                                        *elapsed,
                                        *ten_ms_counter,
                                        time,
                                        &mut rtc,
                                        &rtc_data,
                                    );
                                });
                            });
                        });
                    });
                });
            });
        }
    }

    #[task(schedule = [half_ms], resources = [cp_state,hv_can,elapsed])]
    fn half_ms(mut ctx: half_ms::Context, interval: u32) {
        let mut cp_state = ctx.resources.cp_state;
        let hv_can = ctx.resources.hv_can;
        let elapsed = &mut ctx.resources.elapsed;

        for fifo in &[RxFifo::Fifo0, RxFifo::Fifo1] {
            if let Ok(rx_frame) = hv_can.receive(fifo) {
                elapsed.lock(|elapsed| {
                    can_receive_logic(&rx_frame, *elapsed, &mut cp_state);
                });
            }
        }

        // Play it again, Sam.
        ctx.schedule
            .half_ms(ctx.scheduled + interval.cycles(), interval)
            .unwrap();
    }

    // Setting to higher priority to ensure counting always counts.
    #[task(priority = 2, schedule = [one_ms], resources = [elapsed])]
    fn one_ms(ctx: one_ms::Context, interval: u32) {
        *ctx.resources.elapsed += 1;
        // Play it again, Sam.
        ctx.schedule
            .one_ms(ctx.scheduled + interval.cycles(), interval)
            .unwrap();
    }
    #[task(schedule = [ten_ms], resources = [elapsed, cp_state,ten_ms_counter,hv_can])]
    fn ten_ms(mut ctx: ten_ms::Context, interval: u32) {
        let mut ten_ms_counter = ctx.resources.ten_ms_counter;
        let mut cp_state = ctx.resources.cp_state;
        let mut hv_can = ctx.resources.hv_can;
        let elapsed = &mut ctx.resources.elapsed;
        elapsed.lock(|elapsed| {
            ten_ms_loop(&mut cp_state, *elapsed, &mut ten_ms_counter, &mut hv_can);
        });
        // Once run, flip it off.
        if cp_state.quiet_to_verbose {
            cp_state.quiet_to_verbose = false;
        }
        if cp_state.print_menu_request {
            cp_state.print_menu_request = false;
        }

        // Play it again, Sam.
        ctx.schedule
            .ten_ms(ctx.scheduled + interval.cycles(), interval)
            .unwrap();
    }
    #[task(schedule = [fifty_ms], resources = [elapsed,cp_state,fifty_ms_counter,hv_can])]
    fn fifty_ms(mut ctx: fifty_ms::Context, interval: u32) {
        let mut fifty_ms_counter = ctx.resources.fifty_ms_counter;
        let mut cp_state = ctx.resources.cp_state;
        let hv_can = ctx.resources.hv_can;
        let elapsed = &mut ctx.resources.elapsed;
        elapsed.lock(|elapsed| {
            fifty_ms_loop(*elapsed, &mut fifty_ms_counter, &mut cp_state, &hv_can);
        });

        // Play it again, Sam.
        ctx.schedule
            .fifty_ms(ctx.scheduled + interval.cycles(), interval)
            .unwrap();
    }

    #[task(schedule = [hundred_ms], resources = [elapsed,cp_state,hundred_ms_counter,hv_can,latch_out])]
    fn hundred_ms(mut ctx: hundred_ms::Context, interval: u32) {
        let mut hundred_ms_counter = ctx.resources.hundred_ms_counter;
        let mut cp_state = ctx.resources.cp_state;
        let hv_can = ctx.resources.hv_can;
        let elapsed = &mut ctx.resources.elapsed;
        let latch_out = ctx.resources.latch_out;
        elapsed.lock(|elapsed| {
            hundred_ms_loop(
                *elapsed,
                &mut hundred_ms_counter,
                &mut cp_state,
                &hv_can,
                &SEMAPHORE,
            );
        });
        // Insert Enable is set low during charge.
        if cp_state.charger_relay_enabled {
            latch_out.set_low().ok();
        } else {
            latch_out.set_high().ok();
        }

        // Play it again, Sam.
        ctx.schedule
            .hundred_ms(ctx.scheduled + interval.cycles(), interval)
            .unwrap();
    }

    #[task(schedule = [two_fifty_ms], resources = [elapsed,cp_state,two_fifty_ms_counter,hv_can])]
    fn two_fifty_ms(mut ctx: two_fifty_ms::Context, interval: u32) {
        let mut two_fifty_ms_counter = ctx.resources.two_fifty_ms_counter;
        let mut cp_state = ctx.resources.cp_state;
        let hv_can = ctx.resources.hv_can;
        let elapsed = &mut ctx.resources.elapsed;
        elapsed.lock(|elapsed| {
            two_fifty_ms_loop(*elapsed, &mut two_fifty_ms_counter, &mut cp_state, &hv_can);
        });

        // Play it again, Sam.
        ctx.schedule
            .two_fifty_ms(ctx.scheduled + interval.cycles(), interval)
            .unwrap();
    }

    #[task(schedule = [five_hundred_ms], resources = [elapsed,cp_state,five_hundred_ms_counter,hv_can])]
    fn five_hundred_ms(mut ctx: five_hundred_ms::Context, interval: u32) {
        let mut five_hundred_ms_counter = ctx.resources.five_hundred_ms_counter;
        let mut cp_state = ctx.resources.cp_state;
        let hv_can = ctx.resources.hv_can;
        let elapsed = &mut ctx.resources.elapsed;
        elapsed.lock(|elapsed| {
            five_hundred_ms_loop(
                *elapsed,
                &mut five_hundred_ms_counter,
                &mut cp_state,
                &hv_can,
            );
        });

        // Play it again, Sam.
        ctx.schedule
            .five_hundred_ms(ctx.scheduled + interval.cycles(), interval)
            .unwrap();
    }
    #[task(schedule = [thousand_ms], resources = [elapsed,cp_state,thousand_ms_counter,hv_can])]
    fn thousand_ms(mut ctx: thousand_ms::Context, interval: u32) {
        let mut thousand_ms_counter = ctx.resources.thousand_ms_counter;
        let mut cp_state = ctx.resources.cp_state;
        let hv_can = ctx.resources.hv_can;
        let elapsed = &mut ctx.resources.elapsed;
        elapsed.lock(|elapsed| {
            thousand_ms_loop(*elapsed, &mut thousand_ms_counter, &mut cp_state, &hv_can);
        });

        // Play it again, Sam.
        ctx.schedule
            .thousand_ms(ctx.scheduled + interval.cycles(), interval)
            .unwrap();
    }

    #[task(binds = USART2, resources = [elapsed,rx,tx,cp_state,rtc,rtc_data])]
    fn USART2(mut ctx: USART2::Context) {
        // Only elapsed needs a lock, but because it's part of the context
        // it needs to be split out.
        let elapsed = &mut ctx.resources.elapsed;
        let cp_state = ctx.resources.cp_state;
        let tx = ctx.resources.tx;
        let rtc = ctx.resources.rtc;
        let rtc_data = ctx.resources.rtc_data;
        if let Ok(received) = ctx.resources.rx.read() {
            elapsed.lock(|elapsed| {
                process_serial(
                    received, // command
                    *elapsed, cp_state, tx, rtc, rtc_data,
                );
            });
        }
    }

    #[task(binds = EXTI3)]
    fn EXTI3(_ctx: EXTI3::Context) {
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

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn TIM3();
        fn TIM4();
    }
};
