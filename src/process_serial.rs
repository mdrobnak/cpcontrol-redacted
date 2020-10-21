#![deny(warnings)]
use crate::types::CPState;
use crate::types::*;
use heapless::consts::U60;
use heapless::String;
use ufmt::uwrite;

pub fn init(command: u8, elapsed: u32, mut cp_state: CPState) -> CPState {
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
            cp_state.charger_relay_enabled = true;
            cp_state.set_led(LEDStateEnum::GreenBlink);
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
    cp_state
}
