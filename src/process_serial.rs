#![deny(warnings)]
use crate::types::CPState;
use crate::types::*;
use crate::{uprint, uprintln};
use core::fmt::Write;
use heapless::consts::U60;
use heapless::String;
use rtcc::Rtcc;
use rtcc::Timelike;
use ufmt::uwrite;

pub fn init(
    command: u8,
    elapsed: u32,
    mut cp_state: &mut CPState,
    mut tx: &mut SerialConsoleOutput,
    mut rtc: &mut Rtc,
    mut rtc_data: &mut RTCUpdate,
) {
    if cp_state.rtc_update {
        rtc_input(
            command,
            elapsed,
            &mut cp_state,
            &mut tx,
            &mut rtc,
            &mut rtc_data,
        );
    } else {
        normal_input(command, elapsed, &mut cp_state, &mut tx);
    }
}
pub fn parse_rtc(
    tx: &mut SerialConsoleOutput,
    command: u8,
    max_len: u8,
    rtc_data: &mut RTCUpdate,
) -> bool {
    if command == 0x00{
        rtc_data.temp.pop();
    } else if command > 0x29 && command < 0x3A && rtc_data.temp.len() < max_len.into() {
        uprint!(
            tx,
            "\x1B[8;{}H{}",
            rtc_data.temp.len() + 23,
            command as char
        );
        rtc_data.temp.push(command as char).ok();
    }
    if rtc_data.temp.len() == max_len.into() && command == 0x00{
        return true;
    } else {
        return false;
    }
}

pub fn rtc_input(
    command: u8,
    _elapsed: u32,
    mut cp_state: &mut CPState,
    tx: &mut SerialConsoleOutput,
    rtc: &mut Rtc,
    mut rtc_data: &mut RTCUpdate,
) {
    // Check if there's an update in progress already
    let mut date_uip = rtc_data.y_uip || rtc_data.m_uip || rtc_data.d_uip;
    let mut time_uip = rtc_data.h_uip || rtc_data.min_uip || rtc_data.s_uip;
    if date_uip || time_uip {
        if rtc_data.y_uip {
            if parse_rtc(tx, command, 4, &mut rtc_data) {
                // Update the year
                let year: u16 = rtc_data.temp.parse().unwrap();
                rtc.set_year(year).ok();
                rtc_data.y_uip = false;
            }
        } else if rtc_data.m_uip {
            if parse_rtc(tx, command, 2, &mut rtc_data) {
                // They hit enter or an extraneous character
                // Update the year
                let month: u8 = rtc_data.temp.parse().unwrap();
                rtc.set_month(month).ok();
                rtc_data.m_uip = false;
            }
        } else if rtc_data.d_uip {
            if parse_rtc(tx, command, 2, &mut rtc_data) {
                let day: u8 = rtc_data.temp.parse().unwrap();
                rtc.set_day(day).ok();
                rtc_data.d_uip = false;
            }
        } else if rtc_data.h_uip {
            if parse_rtc(tx, command, 2, &mut rtc_data) {
                let hour: u8 = rtc_data.temp.parse().unwrap();
                rtc.set_hours(rtcc::Hours::H24(hour)).ok();
                rtc_data.h_uip = false;
            }
        } else if rtc_data.min_uip {
            if parse_rtc(tx, command, 2, &mut rtc_data) {
                let minute: u8 = rtc_data.temp.parse().unwrap();
                rtc.set_minutes(minute).ok();
                rtc_data.min_uip = false;
            }
        } else if rtc_data.s_uip {
            if parse_rtc(tx, command, 2, &mut rtc_data) {
                let second: u8 = rtc_data.temp.parse().unwrap();
                rtc.set_seconds(second).ok();
                rtc_data.s_uip = false;
            }
        }

        // If all of the flags are clear,we've updated things.
        date_uip = rtc_data.y_uip || rtc_data.m_uip || rtc_data.d_uip;
        time_uip = rtc_data.h_uip || rtc_data.min_uip || rtc_data.s_uip;
        if !(date_uip || time_uip) {
            rtc_data.temp.clear();
            uprintln!(tx, "\x1B[2J\x1B[HSet Clock: ");
        }
    } else {
        match command {
            // 1
            0x31 => {
                uprint!(
                    tx,
                    "\x1B[8HEnter 4 digit year:   {}\x1B[0K",
                    rtc.get_year().unwrap()
                );
                // Set update in progress flag.
                rtc_data.y_uip = true;
            }

            // 2
            0x32 => {
                uprint!(
                    tx,
                    "\x1B[8HEnter 2 digit month:  {}\x1B[0K",
                    rtc.get_month().unwrap()
                );
                // Set update in progress flag.
                rtc_data.m_uip = true;
            }

            // 3
            0x33 => {
                uprint!(
                    tx,
                    "\x1B[8HEnter 2 digit day:    {}\x1B[0K",
                    rtc.get_day().unwrap()
                );
                // Set update in progress flag.
                rtc_data.d_uip = true;
            }
            // 4
            0x34 => {
                let fulltime = rtc.get_time().unwrap();
                uprint!(
                    tx,
                    "\x1B[8HEnter 2 digit hour:   {}\x1B[0K",
                    fulltime.hour()
                );
                // Set update in progress flag.
                rtc_data.h_uip = true;
            }
            // 5
            0x35 => {
                uprint!(
                    tx,
                    "\x1B[8HEnter 2 digit minute: {}\x1B[0K",
                    rtc.get_minutes().unwrap()
                );
                // Set update in progress flag.
                rtc_data.min_uip = true;
            }
            // 6
            0x36 => {
                uprint!(
                    tx,
                    "\x1B[8HEnter 2 digit second: {}\x1B[0K",
                    rtc.get_seconds().unwrap()
                );
                // Set update in progress flag.
                rtc_data.s_uip = true;
            }

            0xD => {
                let mut s: String<U60> = String::new();
                if (rtc.get_year().unwrap()) == 1985 {
                    uwrite!(s, "Great Scott!").ok();
                    cp_state.activity_list.push_back(s);
                }

                cp_state.rtc_update = false;
                cp_state.quiet_to_verbose = true;
            }
            _ => {}
        }
    }
}
pub fn normal_input(
    command: u8,
    elapsed: u32,
    mut cp_state: &mut CPState,
    tx: &mut SerialConsoleOutput,
) {
    match command {
        // a
        0x61 => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Charge start is auto.", elapsed).ok();
            cp_state.activity_list.push_back(s);
            cp_state.auto_start = true;
        }
        // A
        0x41 => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Charge start is manual.", elapsed).ok();
            cp_state.activity_list.push_back(s);
            cp_state.auto_start = false;
        }
        // c
        0x63 => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Starting Charge!", elapsed).ok();
            cp_state.activity_list.push_back(s);
            if cp_state.charger_type == ChargerTypeEnum::AC {
                cp_state.contactor_request_state = ContactorRequestStateEnum::ContactorACRequest;
            } else if cp_state.charger_type == ChargerTypeEnum::DC {
                cp_state.contactor_request_state = ContactorRequestStateEnum::ContactorDCRequest;
            }
        }
        // C
        0x43 => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Stopping Charge!", elapsed).ok();
            cp_state.activity_list.push_back(s);
            cp_state.charger_relay_enabled = false;
            cp_state.charge_state = ChargeStateEnum::StopCharge;
            cp_state.set_led(LEDStateEnum::WhiteBlue);
        }
        // d
        0x64 => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Opening Door.", elapsed).ok();
            cp_state.activity_list.push_back(s);
            cp_state.cp_door_state = DoorStateEnum::DoorOpenRequest;
        }
        // D
        0x44 => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Closing Door.", elapsed).ok();
            cp_state.activity_list.push_back(s);
            cp_state.cp_door_state = DoorStateEnum::DoorCloseRequest;
        }
        // e
        0x65 => {
            cp_state.quiet_to_verbose = true;
        }
        // l
        0x6C => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Vehicle Locked.", elapsed).ok();
            cp_state.activity_list.push_back(s);
            cp_state.vehicle_locked = true;
        }

        // L
        0x4C => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Vehicle Unlocked.", elapsed).ok();
            cp_state.activity_list.push_back(s);
            cp_state.vehicle_locked = false;
        }

        // m
        0x6D => {
            cp_state.print_menu_request = true;
        }
        // r
        0x72 => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Starting Rainbow mode.", elapsed).ok();
            cp_state.activity_list.push_back(s);
            cp_state.set_led(LEDStateEnum::Rainbow);
        }
        // R
        0x52 => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Stopping Rainbow mode.", elapsed).ok();
            cp_state.activity_list.push_back(s);
            cp_state.set_previous_led();
        }
        // s
        0x73 => {
            // Set the RTC clock
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Enter RTC update mode.", elapsed).ok();
            cp_state.activity_list.push_back(s);
            uprintln!(tx, "\x1B[2J\x1B[HSet Clock: ");
            cp_state.rtc_update = true;
        }
        // v
        0x76 => {
            cp_state.verbose_stats = true;
            cp_state.quiet_to_verbose = true;
        }
        // V
        0x56 => {
            cp_state.verbose_stats = false;
        }
        _ => {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Invalid selection!", elapsed).ok();
            cp_state.activity_list.push_back(s);
        }
    }
}
