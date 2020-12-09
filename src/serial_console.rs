#![deny(warnings)]
use crate::types::*;
use crate::{uprint, uprintln};
use core::fmt::Write;
use rtcc::Rtcc;

pub fn serial_console(
    tx: &mut SerialConsoleOutput,
    cp_state: &CPState,
    sys_ticks: u32,
    ten_ms_counter: u16,
    time: rtcc::NaiveDateTime,
    rtc: &mut Rtc,
    rtc_data: &RTCUpdate,
) {
    const NO_ATTRIB: &str = "\x1B[0m";
    const ERASE_EOL: &str = "\x1B[0K";
    let verbose_console = cp_state.verbose_stats;
    let print_header = ten_ms_counter % 3000 == 0 || cp_state.quiet_to_verbose;
    let print_menu = cp_state.print_menu_request;
    if cp_state.rtc_update {
        set_rtc(tx, rtc, rtc_data, ten_ms_counter);
    } else if verbose_console {
        if print_header {
            print_header_to_serial(tx, verbose_console);
        }

        if ten_ms_counter % 10 <= 3 {
            let mut line = 15;
            for i in cp_state.activity_list.iter() {
                uprintln!(tx, "\x1B[{};3H{}{}\x1B[{};64H|", line, i, ERASE_EOL, line);
                line = line + 1;
            }
        }

        if ten_ms_counter % 50 <= 3 {
            uprintln!(
                tx,
                "\x1B[20HCP Ver: {:X}\x1B[20;20HCP Msg Type: {}\x1B[20;40HVehicle Locked: {}",
                cp_state.app_hash,
                if cp_state.fw_ver == CPVerEnum::Fw2020 {
                    2020
                } else if cp_state.fw_ver == CPVerEnum::Fw2019 {
                    2019
                } else {
                    2018
                },
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
        match cp_state.desired_cp_led_state {
            LEDStateEnum::WhiteBlue => {
                uprintln!(tx, "\x1B[34mBlue Solid{}{}", ERASE_EOL, NO_ATTRIB);
            }
            LEDStateEnum::BlueBlink => {
                uprintln!(tx, "\x1B[5;34mBlue Blinking{}{}", ERASE_EOL, NO_ATTRIB);
            }
            LEDStateEnum::GreenBlink => {
                uprintln!(tx, "\x1B[5;32mGreen Blinking{}{}", ERASE_EOL, NO_ATTRIB);
            }
            LEDStateEnum::GreenSolid => {
                uprintln!(tx, "\x1B[32mGreen Solid{}{}", ERASE_EOL, NO_ATTRIB);
            }
            LEDStateEnum::Rainbow => {
                let res = (ten_ms_counter % 112) >> 2;
                uprintln!(
                    tx,
                    "\x1B[{}mR\x1B[{}mA\x1B[{}mI\x1B[{}mN\x1B[{}mB\x1B[{}mO\x1B[{}mW{}{}",
                    31 + ((res + 6) % 7),
                    31 + ((res + 5) % 7),
                    31 + ((res + 4) % 7),
                    31 + ((res + 3) % 7),
                    31 + ((res + 2) % 7),
                    31 + ((res + 1) % 7),
                    31 + ((res + 0) % 7),
                    ERASE_EOL,
                    NO_ATTRIB
                );
            }
        }

        uprint!(tx, "\x1B[22HState: {}{}", cp_state.charge_state, ERASE_EOL);
        uprintln!(tx, "\x1B[22;30HCAN Loop Status: Probably Fine.");
        uprintln!(tx, "\x1B[23HTime: {}", time);
    } else if ten_ms_counter % 50 <= 3 {
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
    uprintln!(tx, "s - Set the date and time.");
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

pub fn set_rtc(
    tx: &mut SerialConsoleOutput,
    rtc: &mut Rtc,
    rtc_data: &RTCUpdate,
    ten_ms_counter: u16,
) {
    if ten_ms_counter % 10 <= 3 {
        uprintln!(tx, "\x1B[2H1. Year:   {}", rtc.get_year().unwrap());
        uprintln!(tx, "\x1B[3H2. Month:  {}", rtc.get_month().unwrap());
        uprintln!(tx, "\x1B[4H3. Day:    {}", rtc.get_day().unwrap());
        uprintln!(tx, "\x1B[5H4. Hour:   {:?}", rtc.get_hours().unwrap());
        uprintln!(tx, "\x1B[6H5. Minute: {}", rtc.get_minutes().unwrap());
        uprintln!(tx, "\x1B[7H6. Second: {}", rtc.get_seconds().unwrap());
        let date_uip = rtc_data.y_uip || rtc_data.m_uip || rtc_data.d_uip;
        let time_uip = rtc_data.h_uip || rtc_data.min_uip || rtc_data.s_uip;

        if date_uip || time_uip {
            uprint!(tx, "\x1B[8;{}H", rtc_data.temp.len() + 23);
        } else {
            uprint!(
                tx,
                "\x1B[8HEnter Number of item to modify (Enter to exit): "
            );
        }
    }
}
