#![deny(warnings)]
#![allow(clippy::char_lit_as_u8)]
use crate::types::*;

// Logging
use crate::handle_can_error;
use heapless::consts::U60;
use heapless::String;
use ufmt::uwrite;

pub fn init(
    elapsed: u32,
    mut five_hunded_ms_counter: u8,
    mut cp_state: &mut CPState,
    hv_can: &HVCAN,
) -> u8 {
    vvvm(hv_can).unwrap_or_else(|error| {
        handle_can_error!(vvvm, error, "500ms_0", cp_state, elapsed);
    });
    tatata(hv_can, &mut cp_state).unwrap_or_else(|error| {
        handle_can_error!(tatata, error, "500ms_1", cp_state, elapsed);
    });
    u6(hv_can, five_hunded_ms_counter).unwrap_or_else(|error| {
        handle_can_error!(u6, error, "500ms_2", cp_state, elapsed);
    });
    if five_hunded_ms_counter < 255 {
        five_hunded_ms_counter = five_hunded_ms_counter + 1;
    } else {
        five_hunded_ms_counter = 0;
    }

    five_hunded_ms_counter
}

pub fn vvvm(hv_can: &HVCAN) -> Result<(), CanError> {
    let id: u16 = 0x000;
    let size: u8 = 5;
    let mut vvvm_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    vvvm_frame.set_data_length(size.into());
    let vvvm = vvvm_frame.data_as_mut();
    vvvm[0] = 0x00;
    vvvm[1] = 0x00;
    vvvm[2] = 0x00;
    vvvm[3] = 0x00;
    vvvm[4] = 0x00;
    hv_can.transmit(&vvvm_frame.into())
}

pub fn tatata(hv_can: &HVCAN, cp_state: &mut CPState) -> Result<(), CanError> {
    let id: u16 = 0x000;
    let size: u8 = if cp_state.fw_ver == CPVerEnum::Fw2020 {
        5
    } else {
        4
    };
    let mut tatata_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    tatata_frame.set_data_length(size.into());
    let tatata = tatata_frame.data_as_mut();
    tatata[1] = 0x00;
    tatata[2] = 0x00;
    tatata[3] = 0x00;
    match cp_state.cp_door_state {
        DoorStateEnum::DoorIdle => {
            if cp_state.auto_start {
                tatata[0] = 0x00;
            } else {
                tatata[0] = 0x00;
            }
        }
        DoorStateEnum::DoorOpenRequest => {
            tatata[0] |= 0x00;
        }
        DoorStateEnum::DoorCloseRequest => {
            tatata[0] |= 0x00;
        }
        DoorStateEnum::DoorClosing => {
            tatata[0] &= 0x00;
            let id: u16 = 0x000;
            let mut response_frame = DataFrame::new(ID::BaseID(BaseID::new(id.into())));
            response_frame.set_data_length(8);
            let response = response_frame.data_as_mut();
            response[0] = 0x00;
            response[1] = 0x00;
            response[2] = 0x00;
            response[3] = 0x00;
            response[4] = 0x00;
            response[5] = 0x00;
            response[6] = 0x00;
            response[7] = 0x00;
            hv_can.transmit(&response_frame.into()).ok();
        }
        DoorStateEnum::DoorOpening => {
            tatata[0] &= 0x00;
            let id: u16 = 0x000;
            let mut response_frame = DataFrame::new(ID::BaseID(BaseID::new(id.into())));
            response_frame.set_data_length(8);
            let response = response_frame.data_as_mut();
            response[0] = 0x00;
            response[1] = 0x00;
            response[2] = 0x00;
            response[3] = 0x00;
            response[4] = 0x00;
            response[5] = 0x00;
            response[6] = 0x00;
            response[7] = 0x00;
            hv_can.transmit(&response_frame.into()).ok();
        }
        DoorStateEnum::DoorOpen | DoorStateEnum::DoorClosed => {
            // Do nothing.
        }
    }
    if cp_state.fw_ver == CPVerEnum::Fw2020 {
        tatata[0] |= 0x00;
        tatata[4] = 0x00;
    }
    hv_can.transmit(&tatata_frame.into())
}

pub fn u6(hv_can: &HVCAN, five_hunded_ms_counter: u8) -> Result<(), CanError> {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut u6_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    u6_frame.set_data_length(size.into());
    let u6 = u6_frame.data_as_mut();
    let res = five_hunded_ms_counter % 12;
    match res {
        0 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        1 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        2 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        3 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        4 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        5 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        6 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        7 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        8 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        9 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        10 => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
        11 | _ => {
            u6[0] = 0x00;
            u6[1] = 0x00;
            u6[2] = 0x00;
            u6[3] = 0x00;
            u6[4] = 0x00;
            u6[5] = 0x00;
            u6[6] = 0x00;
            u6[7] = 0x00;
        }
    }
    hv_can.transmit(&u6_frame.into())
}
