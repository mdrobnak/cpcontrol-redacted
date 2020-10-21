#![deny(warnings)]
use crate::serial_console::serial_console;
extern crate stm32f7xx_hal as hal;
use crate::types::*;
use hal::can::{BaseID, DataFrame, ID};

pub fn init(
    mut tx: &mut SerialConsoleOutput,
    mut cp_state: &mut CPState,
    elapsed: u32,
    mut ten_ms_counter: u16,
    hv_can: &HVCAN,
) -> u16 {
    u1(ten_ms_counter, hv_can);
    ccaa(hv_can, &mut cp_state);
    cbtxva(hv_can, &mut cp_state);
    cbtxt(hv_can, &mut cp_state);
    cs(hv_can, &mut cp_state);

    // Check for timeout with CP ECU
    if (elapsed - cp_state.previous_cptod_ts) > 1000 {
        cp_state.cp_comm_timeout = true;
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
    );
    if ten_ms_counter < 65535 {
        ten_ms_counter = ten_ms_counter + 1;
    } else {
        ten_ms_counter = 0;
    }

    ten_ms_counter
}

pub fn u1(ten_ms_counter: u16, hv_can: &HVCAN) {

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
    hv_can.transmit(&u1_frame.into()).ok();
}

pub fn ccaa(hv_can: &HVCAN, cp_state: &mut CPState) {
    let id: u16 = 0x000;
    let size: u8 = 6;
    let mut ccaa_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    ccaa_frame.set_data_length(size.into());

    if cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACRequest {
        // Do stuff to make the contactors correct.
        if cp_state.cbtxva_request {
            cp_state.contactor_request_state = ContactorRequestStateEnum::ContactorACEnable;
            cp_state.charge_state = ChargeStateEnum::ContactorRequest;
        }
    }

    let ccaa = ccaa_frame.data_as_mut();
    ccaa[0] = 0x00;
    ccaa[1] = 0x00;
    ccaa[2] = 0x00;
    ccaa[3] = 0x00;
    ccaa[4] = 0x00;
    ccaa[5] = 0x00;

    if cp_state.contactor_request_state == ContactorRequestStateEnum::ContactorACEnable {
        // Do stuff?
        ccaa[5] = 0x00;
    }
    hv_can.transmit(&ccaa_frame.into()).ok();
}

pub fn cbtxva(hv_can: &HVCAN, cp_state: &mut CPState) {


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
    hv_can.transmit(&cbtxva_frame.into()).ok();
}

pub fn cbtxt(hv_can: &HVCAN, cp_state: &mut CPState) {








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
    hv_can.transmit(&cbtxt_frame.into()).ok();
}

pub fn cs(hv_can: &HVCAN, cp_state: &mut CPState) {
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

    hv_can.transmit(&cs_frame.into()).ok();
}
