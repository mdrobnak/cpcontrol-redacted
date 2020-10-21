#![deny(warnings)]
use crate::types::*;
use core::fmt::Write;

macro_rules! uprint {
    ($serial:expr, $($arg:tt)*) => {
        $serial.write_fmt(format_args!($($arg)*)).ok()
    };
}

macro_rules! uprintln {
    ($serial:expr, $fmt:expr) => {
        uprint!($serial, concat!($fmt, "\n"))
    };
    ($serial:expr, $fmt:expr, $($arg:tt)*) => {
        uprint!($serial, concat!($fmt, "\n"), $($arg)*)
    };
}
pub fn serial_console(
    tx: &mut SerialConsoleOutput,
    cp_state: &CPState,
    sys_ticks: u32,
    ten_ms_counter: u16,
    verbose_console: bool,
    print_header: bool,
    print_menu: bool,
) {
    const NO_ATTRIB: &str = "\x1B[0m";

    if verbose_console {
        if print_header {
            print_header_to_serial(tx, verbose_console);
        }

        if ten_ms_counter % 10 == 0 {
            let mut line = 14;
            for i in cp_state.activity_list.iter() {
                uprintln!(tx, "\x1B[{};3H{}", line, i);
                line = line + 1;
            }
        }

        if ten_ms_counter % 50 == 0 {
            uprintln!(
                tx,
                "\x1B[20HInit: true\x1B[20;20HFaulted?: {}\x1B[20;40HVehicle Locked: {}",
                !cp_state.cp_init,
                cp_state.vehicle_locked
            );
        }
        uprint!(
            tx,
            "\x1B[21HUptime: {}\x1B[21;20HCharge Type: {}",
            sys_ticks,
            cp_state.charger_type
        );
        uprint!(tx, "\x1B[21;40HLED State: ");
        uprintln!(tx, "\x1B[34mBlue Solid{}", NO_ATTRIB);
        uprint!(tx, "\x1B[22HState: {} \x1B[0K", cp_state.charge_state);
        uprintln!(tx, "\x1B[22;30HCAN Loop Status: Probably Fine.");
    } else {
        if ten_ms_counter % 50 == 0 {
            if print_menu {
                print_header_to_serial(tx, verbose_console);
            } else if print_header {
                uprintln!(
                    tx,
                    "Press v to enable verbose statistics. Press m for a list of commands."
                );
            }
            uprint!(tx, "State: {}  Charging: ", cp_state.charge_state);
            if cp_state.charger_relay_enabled {
                uprint!(tx, "Enabled   ");
            } else {
                uprint!(tx, "Disabled  ");
            }
            uprintln!(tx, "Uptime: {}", sys_ticks);
        }
    }
}
pub fn print_header_to_serial(tx: &mut SerialConsoleOutput, verbose_console: bool) {
    if verbose_console {
        uprintln!(tx, "\x1B[2J\x1B[HCommands: ");
    } else {
        uprintln!(tx, "Commands: ");
    }
    uprintln!(tx, "a / A - Enable / Disable charging auto-start.");
    uprintln!(tx, "c / C - Start / End Charge.");
    uprintln!(tx, "d / D - Open / Close door.");
    uprintln!(tx, "e - Clear / rEfresh the screen.");
    uprintln!(tx, "l / L - Lock / Unlock doors.");
    uprintln!(tx, "m - Show menu with verbose disabled.");
    uprintln!(tx, "r / R - Start / End Rainbow / Rave mode.");
    uprintln!(tx, "v / V - Enable / Disable verbose.");
    if verbose_console {
        verbose_footer(tx);
    }
}
#[rustfmt::skip]
pub fn verbose_footer(tx: &mut SerialConsoleOutput) {
    uprintln!(tx, "Command? ");
    uprintln!(tx, "");
    uprintln!(tx, "                          Activity");
    uprintln!(tx, "+--------------------------------------------------------------+");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "|                                                              |");
    uprintln!(tx, "+--------------------------------------------------------------+");
}
