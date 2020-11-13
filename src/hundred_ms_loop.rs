#![deny(warnings)]
use crate::types::*;
use crate::utils::checksum_calc;

pub fn init(mut hundred_ms_counter: u8, mut cp_state: &mut CPState, hv_can: &HVCAN) -> u8 {
    u2(hv_can);
    u3(hv_can, &mut cp_state);
    ctfa(hv_can);
    scss(hv_can);
    da(hv_can, &mut cp_state);
    u4(hv_can, &mut cp_state);
    vvvsv(hv_can, hundred_ms_counter);
    counter(hv_can, hundred_ms_counter);
    msx(hv_can);
    vvvv(hv_can, hundred_ms_counter);
    if hundred_ms_counter < 255 {
        hundred_ms_counter = hundred_ms_counter + 1;
    } else {
        hundred_ms_counter = 0;
    }
    hundred_ms_counter
}

pub fn u2(hv_can: &HVCAN) {
    let id: u16 = 0x000;
    let size: u8 = 1;
    let mut u2_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    u2_frame.set_data_length(size.into());
    let u2 = u2_frame.data_as_mut();
    u2[0] = 0x00;
    hv_can.transmit(&u2_frame.into()).ok();
}

pub fn u3(hv_can: &HVCAN, cp_state: &mut CPState) {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut u3_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    u3_frame.set_data_length(size.into());
    let u3 = u3_frame.data_as_mut();
    u3[0] = 0x00;
    u3[1] = 0x00;
    u3[2] = 0x00;
    u3[3] = 0x00;
    u3[4] = 0x00;
    u3[5] = 0x00;
    u3[6] = 0x00;
    u3[7] = 0x00;
    if cp_state.evse_request {
        u3[0] += 0x00;
        u3[7] += 0x00;
        cp_state.set_led(LEDStateEnum::GreenBlink);
    }
    hv_can.transmit(&u3_frame.into()).ok();
}

pub fn ctfa(hv_can: &HVCAN) {
    let id: u16 = 0x000;
    let size: u8 = 1;
    let mut ctfa_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    ctfa_frame.set_data_length(size.into());
    let ctfa = ctfa_frame.data_as_mut();
    ctfa[0] = 0x00;
    hv_can.transmit(&ctfa_frame.into()).ok();
}

pub fn scss(hv_can: &HVCAN) {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut scss_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    scss_frame.set_data_length(size.into());
    let scss = scss_frame.data_as_mut();
    scss[0] = 0x00;
    scss[1] = 0x00;
    scss[2] = 0x00;
    scss[3] = 0x00;
    scss[4] = 0x00;
    scss[5] = 0x00;
    scss[6] = 0x00;
    scss[7] = 0x00;
    hv_can
        .transmit(&scss_frame.into())
        .ok();
}

pub fn da(hv_can: &HVCAN, cp_state: &mut CPState) {
    let id: u16 = 0x000;
    let size: u8 = 2;
    let mut da_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    da_frame.set_data_length(size.into());
    let da = da_frame.data_as_mut();
    if cp_state.cp_init {
        da[0] = 0x00;
    } else {
        da[0] = 0x00;
    }

    da[1] = 0x00;

    hv_can.transmit(&da_frame.into()).ok();
}

pub fn u4(hv_can: &HVCAN, cp_state: &mut CPState) {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut u4_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    u4_frame.set_data_length(size.into());
    let u4 = u4_frame.data_as_mut();
    u4[0] = 0x00;
    u4[1] = 0x00;
    u4[2] = 0x00;
    u4[3] = 0x00;
    u4[4] = 0x00;
    u4[5] = 0x00;
    u4[6] = 0x00;
    u4[7] = 0x00;
    if cp_state.vehicle_locked {
        u4[5] += 0x00;
        u4[6] += 0x00;
    }
    hv_can.transmit(&u4_frame.into()).ok();
}

pub fn vvvsv(hv_can: &HVCAN, hundred_ms_counter: u8) {
    // Checksum verified to be correct.
    let id: u16 = 0x000;
    let size: u8 = 8;
    let res = (hundred_ms_counter % 16) as u8;
    let mut vvvsv_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    vvvsv_frame.set_data_length(size.into());
    let vvvsv = vvvsv_frame.data_as_mut();
    vvvsv[0] = 0x00;
    vvvsv[1] = 0x00;
    vvvsv[2] = 0x00;
    vvvsv[3] = 0x00;
    vvvsv[4] = 0x00;
    vvvsv[5] = 0x00;
    vvvsv[6] = 0x00;
    vvvsv[6] |= res << 4;
    vvvsv[7] = checksum_calc(&vvvsv, id, size);
    hv_can.transmit(&vvvsv_frame.into()).ok();
}

pub fn counter(hv_can: &HVCAN, hundred_ms_counter: u8) {
    let id: u16 = 0x000;
    let size: u8 = 2;
    let mut counter_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    counter_frame.set_data_length(size.into());
    let data = counter_frame.data_as_mut();
    let res = (hundred_ms_counter % 16) as u8;
    data[0] = res + res;
    data[1] = 0;

    hv_can.transmit(&counter_frame.into()).ok();
}

pub fn msx(hv_can: &HVCAN) {
    let id: u16 = 0x000;
    let size: u8 = 6;
    let mut msx_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    msx_frame.set_data_length(size.into());
    let msx = msx_frame.data_as_mut();
    msx[0] = 0x00;
    msx[1] = 0x00;
    msx[2] = 0x00;
    msx[3] = 0x00;
    msx[4] = 0x00;
    msx[5] = 0x00;
    hv_can.transmit(&msx_frame.into()).ok();
}

pub fn vvvv(hv_can: &HVCAN, hundred_ms_counter: u8) {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let res = (hundred_ms_counter % 4) as u8;
    let mut vvvv_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    vvvv_frame.set_data_length(size.into());
    let vvvv = vvvv_frame.data_as_mut();
    match res {
        0 => {
            vvvv[0] = 0x00;
            vvvv[1] = 0x00;
            vvvv[2] = 0x00;
            vvvv[3] = 0x00;
            vvvv[4] = 0x00;
            vvvv[5] = 0x00;
            vvvv[6] = 0x00;
            vvvv[7] = 0x00;
        }
        1 => {
            vvvv[0] = 0x00;
            vvvv[1] = 0x00;
            vvvv[2] = 0x00;
            vvvv[3] = 0x00;
            vvvv[4] = 0x00;
            vvvv[5] = 0x00;
            vvvv[6] = 0x00;
            vvvv[7] = 0x00;
        }
        2 => {
            vvvv[0] = 0x00;
            vvvv[1] = 0x00;
            vvvv[2] = 0x00;
            vvvv[3] = 0x00;
            vvvv[4] = 0x00;
            vvvv[5] = 0x00;
            vvvv[6] = 0x00;
            vvvv[7] = 0x00;
        }
        3 | _ => {
            vvvv[0] = 0x00;
            vvvv[1] = 0x00;
            vvvv[2] = 0x00;
            vvvv[3] = 0x00;
            vvvv[4] = 0x00;
            vvvv[5] = 0x00;
            vvvv[6] = 0x00;
            vvvv[7] = 0x00;
        }
    }
    hv_can.transmit(&vvvv_frame.into()).ok();
}
