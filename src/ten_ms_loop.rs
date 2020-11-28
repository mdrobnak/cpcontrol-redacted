#![deny(warnings)]
use crate::process_init::init as process_init;
use crate::serial_console::serial_console;
use crate::types::*;

// Logging
use crate::handle_can_error;
use heapless::consts::U60;
use heapless::String;
use ufmt::uwrite;

pub fn init(
    mut tx: &mut SerialConsoleOutput,
    mut cp_state: &mut CPState,
    elapsed: u32,
    mut ten_ms_counter: u16,
    hv_can: &HVCAN,
    time: rtcc::NaiveDateTime,
    mut rtc: &mut Rtc,
    rtc_data: &RTCUpdate,
) -> u16 {
    process_init(&mut cp_state, elapsed);
    u1(ten_ms_counter, hv_can).unwrap_or_else(|error| {
        handle_can_error!(u1, error, "10ms_0", cp_state, elapsed);
    });
    ccaa(hv_can, &mut cp_state).unwrap_or_else(|error| {
        handle_can_error!(ccaa, error, "10ms_1", cp_state, elapsed);
    });
    cbtxva(hv_can, &mut cp_state).unwrap_or_else(|error| {
        handle_can_error!(cbtxva, error, "10ms_2", cp_state, elapsed);
    });
    cbtxt(hv_can, &mut cp_state).unwrap_or_else(|error| {
        handle_can_error!(cbtxt, error, "10ms_3", cp_state, elapsed);
    });
    sid845(hv_can, &mut cp_state).unwrap_or_else(|error| {
        handle_can_error!(sid845, error, "10ms_4", cp_state, elapsed);
    });
    cs(hv_can, &mut cp_state).unwrap_or_else(|error| {
        handle_can_error!(cs, error, "10ms_5", cp_state, elapsed);
    });

    // Check for timeout with CP ECU
    // FIXME: Temporarily move from 1 to 3.5 second timeout.
    if (elapsed - cp_state.previous_cptod_ts) > 3500 {
        // Only log a message if we've transitioned to an OK state before.
        if !cp_state.cp_comm_timeout {
            let mut s: String<U60> = String::new();
            uwrite!(s, "{} - Timeout", elapsed,).ok();
            cp_state.activity_list.push_back(s);
        }

        cp_state.cp_comm_timeout = true;
        cp_state.cp_init = false;
        cp_state.init_sequence = 3;
        cp_state.charge_state = ChargeStateEnum::TimeOut;
    }

    serial_console(
        &mut tx,
        &cp_state,
        elapsed,
        ten_ms_counter,
        cp_state.verbose_stats,
        ten_ms_counter % 3000 == 0 || cp_state.quiet_to_verbose,
        cp_state.print_menu_request,
        time,
        &mut rtc,
        &rtc_data,
    );
    if ten_ms_counter < 65535 {
        ten_ms_counter = ten_ms_counter + 1;
    } else {
        ten_ms_counter = 0;
    }

    ten_ms_counter
}

pub fn u1(ten_ms_counter: u16, hv_can: &HVCAN) -> Result<(), CanError> {

    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut u1_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    u1_frame.set_data_length(size.into());

    let u1 = u1_frame.data_as_mut();
    let res = ten_ms_counter % 6;

    match res {
        0 => {
            u1[0] = 0x00;
            u1[1] = 0x00;
            u1[2] = 0x00;
            u1[3] = 0x00;
            u1[4] = 0x00;
            u1[5] = 0x00;
            u1[6] = 0x00;
            u1[7] = 0x00;
        }
        1 => {
            u1[0] = 0x00;
            u1[1] = 0x00;
            u1[2] = 0x00;
            u1[3] = 0x00;
            u1[4] = 0x00;
            u1[5] = 0x00;
            u1[6] = 0x00;
            u1[7] = 0x00;
        }
        2 => {
            u1[0] = 0x00;
            u1[1] = 0x00;
            u1[2] = 0x00;
            u1[3] = 0x00;
            u1[4] = 0x00;
            u1[5] = 0x00;
            u1[6] = 0x00;
            u1[7] = 0x00;
        }
        3 => {
            u1[0] = 0x00;
            u1[1] = 0x00;
            u1[2] = 0x00;
            u1[3] = 0x00;
            u1[4] = 0x00;
            u1[5] = 0x00;
            u1[6] = 0x00;
            u1[7] = 0x00;
        }
        4 => {
            u1[0] = 0x00;
            u1[1] = 0x00;
            u1[2] = 0x00;
            u1[3] = 0x00;
            u1[4] = 0x00;
            u1[5] = 0x00;
            u1[6] = 0x00;
            u1[7] = 0x00;
        }
        5 => {
            u1[0] = 0x00;
            u1[1] = 0x00;
            u1[2] = 0x00;
            u1[3] = 0x00;
            u1[4] = 0x00;
            u1[5] = 0x00;
            u1[6] = 0x00;
            u1[7] = 0x00;
        }
        _ => {
            u1[0] = 0x00;
            u1[1] = 0x00;
            u1[2] = 0x00;
            u1[3] = 0x00;
            u1[4] = 0x00;
            u1[5] = 0x00;
            u1[6] = 0x00;
            u1[7] = 0x00;
        }
    }
    hv_can.transmit(&u1_frame.into())
}

pub fn ccaa(hv_can: &HVCAN, cp_state: &mut CPState) -> Result<(), CanError> {
    let id: u16 = 0x000;
    let size: u8 = 6;
    let mut ccaa_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    ccaa_frame.set_data_length(size.into());

    if cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACRequest
        && cp_state.cbtxva_request
    {
        cp_state.contactor_request_state = ContactorRequestStateEnum::ContactorACEnable;
        cp_state.charge_state = ChargeStateEnum::ContactorRequest;
    } else if cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorDCRequest
        && cp_state.cbtxva_request
    {
        cp_state.contactor_request_state = ContactorRequestStateEnum::ContactorDCEnable;
        cp_state.charge_state = ChargeStateEnum::ContactorRequest;
    }

    let ccaa = ccaa_frame.data_as_mut();
    ccaa[0] = 0x00;
    ccaa[1] = 0x00;
    ccaa[2] = 0x00;
    ccaa[3] = 0x00;
    ccaa[4] = 0x00;
    ccaa[5] = 0x00;

    if cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACEnable {
        ccaa[5] = 0x00;
    } else if cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorDCEnable {
        ccaa[5] = 0x00;
    }
    hv_can.transmit(&ccaa_frame.into())
}

pub fn cbtxva(hv_can: &HVCAN, cp_state: &mut CPState) -> Result<(), CanError> {



    let id: u16 = 0x000;
    let size: u8 = 4;

    let mut cbtxva_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    cbtxva_frame.set_data_length(size.into());

    let cbtxva = cbtxva_frame.data_as_mut();
    cbtxva[0] = 0x00;
    cbtxva[1] = 0x00;
    cbtxva[2] = 0x00;
    if cp_state.cbtxva_request {
        cbtxva[2] += 0x00;
    }
    cbtxva[3] = 0x00;
    hv_can.transmit(&cbtxva_frame.into())
}

pub fn cbtxt(hv_can: &HVCAN, cp_state: &mut CPState) -> Result<(), CanError> {








    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut cbtxt_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    cbtxt_frame.set_data_length(size.into());

    let cbtxt = cbtxt_frame.data_as_mut();

    cbtxt[0] = 0x00;
    cbtxt[1] = 0x00;
    if (cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACRequest)
        || (cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACEnable)
    {
        if cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACRequest {
            cp_state.set_led(LEDStateEnum::BlueBlink);
        }
        cbtxt[0] -= 0x00;
        cbtxt[1] += 0x00;
        cp_state.cbtxva_request = true;
    }
    cbtxt[2] = 0x00;
    cbtxt[3] = 0x00;
    cbtxt[4] = 0x00;
    cbtxt[5] = 0x00;
    cbtxt[6] = 0x00;
    cbtxt[7] = 0x00;
    hv_can.transmit(&cbtxt_frame.into())
}

pub fn sid845(hv_can: &HVCAN, cp_state: &mut CPState) -> Result<(), CanError> {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut cs_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    cs_frame.set_data_length(size.into());

    let cs = cs_frame.data_as_mut();

    cs[0] = 0x00;
    cs[1] = 0x00;
    cs[2] = 0x00;
    cs[3] = 0x00;
    cs[4] = 0x00;

    if cp_state.charger_relay_enabled {
        cs[3] += 0x00;
    } else if cp_state.cbtxva_request
        && (cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACRequest
            || cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACEnable)
    {
        cs[3] += 0x00;
    }
    match cp_state.desired_cp_led_state {
        LEDStateEnum::WhiteBlue => {}

        LEDStateEnum::BlueBlink => {
            cs[3] += 0x00;
        }

        LEDStateEnum::GreenBlink => {
            cs[3] += 0x00;
        }

        LEDStateEnum::GreenSolid => {
            cs[3] += 0x00;
        }

        LEDStateEnum::Rainbow => {
            cs[3] += 0x00;
        }
    }

    if cp_state.charger_relay_enabled {
        cs[4] += 0x00;
    }
    cs[5] = 0x00;
    cs[6] = 0x00;
    cs[7] = 0x00;

    hv_can.transmit(&cs_frame.into())
}

pub fn cs(hv_can: &HVCAN, cp_state: &mut CPState) -> Result<(), CanError> {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut cs_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    cs_frame.set_data_length(size.into());

    let cs = cs_frame.data_as_mut();







    cs[0] = 0x00;
    cs[1] = 0x00;
    cs[2] = 0x00;
    cs[3] = 0x00;
    cs[4] = 0x00;

    if cp_state.charger_relay_enabled {
        cs[4] += 0x00;
    } else if cp_state.cbtxva_request
        && (cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACRequest
            || cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACEnable)
    {
        cs[4] += 0x00;
    } else if cp_state.charge_state == ChargeStateEnum::StopCharge {
        cs[4] = 0x00;
    }
    match cp_state.desired_cp_led_state {
        LEDStateEnum::WhiteBlue => {}

        LEDStateEnum::BlueBlink => {
            cs[4] += 0x00;
        }

        LEDStateEnum::GreenBlink => {
            cs[4] += 0x00;
        }

        LEDStateEnum::GreenSolid => {
            cs[4] += 0x00;
        }

        LEDStateEnum::Rainbow => {
            cs[4] += 0x00;
        }
    }
    cs[5] = 0x00;
    cs[6] = 0x00;
    cs[7] = 0x00;

    hv_can.transmit(&cs_frame.into())
}
