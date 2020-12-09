#![deny(warnings)]
use crate::types::*;
use crate::utils::checksum_calc;

// Logging
use crate::handle_can_error;
use heapless::consts::U60;
use heapless::String;
use ufmt::uwrite;

// Semaphore
use core::cell::Cell;
use cortex_m::interrupt::{free, Mutex};

pub fn init(
    elapsed: u32,
    hundred_ms_ptr: &mut u8,
    mut cp_state: &mut CPState,
    hv_can: &HVCAN,
    semaphore: &Mutex<Cell<bool>>,
) {
    free(|cs| {
        if semaphore.borrow(cs).get() == false {
            // If this is low, then the GPIO is low.
            cp_state.cp_init = false;
        } else {
            cp_state.cp_init = true;
        }
    });

    let hundred_ms_counter: u8 = *hundred_ms_ptr;
    ltbxtxf(hv_can).unwrap_or_else(|error| {
        handle_can_error!(ltbxtxf, error, "100ms_0", cp_state, elapsed);
    });
    u2(hv_can).unwrap_or_else(|error| {
        handle_can_error!(u2, error, "100ms_1", cp_state, elapsed);
    });
    u3(hv_can, &mut cp_state).unwrap_or_else(|error| {
        handle_can_error!(u3, error, "100ms_2", cp_state, elapsed);
    });
    ctfa(hv_can, cp_state, false).unwrap_or_else(|error| {
        handle_can_error!(ctfa, error, "100ms_3", cp_state, elapsed);
    });
    scss(hv_can).unwrap_or_else(|error| {
        handle_can_error!(
            scss,
            error,
            "100ms_4",
            cp_state,
            elapsed
        );
    });
    da(hv_can, &mut cp_state).unwrap_or_else(|error| {
        handle_can_error!(da, error, "100ms_5", cp_state, elapsed);
    });
    u4(hv_can, &mut cp_state).unwrap_or_else(|error| {
        handle_can_error!(u4, error, "100ms_6", cp_state, elapsed);
    });
    vvvsv(hv_can, hundred_ms_counter).unwrap_or_else(|error| {
        handle_can_error!(vvvsv, error, "100ms_7", cp_state, elapsed);
    });
    counter(hv_can, hundred_ms_counter).unwrap_or_else(|error| {
        handle_can_error!(counter, error, "100ms_8", cp_state, elapsed);
    });
    msx(hv_can).unwrap_or_else(|error| {
        handle_can_error!(msx, error, "100ms_9", cp_state, elapsed);
    });
    vvvv(hv_can, hundred_ms_counter).unwrap_or_else(|error| {
        handle_can_error!(vvvv, error, "100ms_10", cp_state, elapsed);
    });
    if hundred_ms_counter < 255 {
        *hundred_ms_ptr = hundred_ms_counter + 1;
    } else {
        *hundred_ms_ptr = 0;
    }
}

pub fn ltbxtxf(hv_can: &HVCAN) -> Result<(), CanError> {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut ltbxtxf_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    ltbxtxf_frame.set_data_length(size.into());
    let ltbxtxf = ltbxtxf_frame.data_as_mut();
    ltbxtxf[0] = 0x00;
    ltbxtxf[1] = 0x00;
    ltbxtxf[2] = 0x00;
    ltbxtxf[3] = 0x00;
    ltbxtxf[4] = 0x00;
    ltbxtxf[5] = 0x00;
    ltbxtxf[6] = 0x00;
    ltbxtxf[7] = 0x00;
    hv_can.transmit(&ltbxtxf_frame.into())
}

pub fn u2(hv_can: &HVCAN) -> Result<(), CanError> {
    let id: u16 = 0x000;
    let size: u8 = 1;
    let mut u2_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    u2_frame.set_data_length(size.into());
    let u2 = u2_frame.data_as_mut();
    u2[0] = 0x00;
    hv_can.transmit(&u2_frame.into())
}

pub fn u3(hv_can: &HVCAN, cp_state: &mut CPState) -> Result<(), CanError> {
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
    hv_can.transmit(&u3_frame.into())
}

pub fn ctfa(hv_can: &HVCAN, cp_state: &mut CPState, fault_set: bool) -> Result<(), CanError> {
    // TODO: Fix fault_set state.
    let id: u16 = 0x000;
    let size: u8 = 1;
    let mut ctfa_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    ctfa_frame.set_data_length(size.into());
    let ctfa = ctfa_frame.data_as_mut();
    ctfa[0] = 0x00;
    if fault_set {
        ctfa[0] += 0x00;
    }
    if cp_state.charger_relay_enabled {
        ctfa[0] -= 0x00;
    }
    hv_can.transmit(&ctfa_frame.into())
}

pub fn scss(hv_can: &HVCAN) -> Result<(), CanError> {
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
    hv_can.transmit(&scss_frame.into())
}

pub fn da(hv_can: &HVCAN, cp_state: &mut CPState) -> Result<(), CanError> {
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

    hv_can.transmit(&da_frame.into())
}

pub fn u4(hv_can: &HVCAN, cp_state: &mut CPState) -> Result<(), CanError> {
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
    hv_can.transmit(&u4_frame.into())
}

pub fn vvvsv(hv_can: &HVCAN, hundred_ms_counter: u8) -> Result<(), CanError> {
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
    hv_can.transmit(&vvvsv_frame.into())
}

pub fn counter(hv_can: &HVCAN, hundred_ms_counter: u8) -> Result<(), CanError> {
    let id: u16 = 0x000;
    let size: u8 = 2;
    let mut counter_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    counter_frame.set_data_length(size.into());
    let data = counter_frame.data_as_mut();
    let res = (hundred_ms_counter % 16) as u8;
    data[0] = res + res;
    data[1] = 0;

    hv_can.transmit(&counter_frame.into())
}

pub fn msx(hv_can: &HVCAN) -> Result<(), CanError> {
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
    hv_can.transmit(&msx_frame.into())
}

pub fn vvvv(hv_can: &HVCAN, hundred_ms_counter: u8) -> Result<(), CanError> {
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
    hv_can.transmit(&vvvv_frame.into())
}
