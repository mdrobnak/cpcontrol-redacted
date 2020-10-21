#![deny(warnings)]
extern crate stm32f7xx_hal as hal;
use crate::types::*;
use hal::can::{BaseID, DataFrame, ID};

pub fn init(mut five_hunded_ms_counter: u8, mut cp_state: &mut CPState, hv_can: &HVCAN) -> u8 {
    vvvm(hv_can);
    tatata(hv_can, &mut cp_state);
    u6(hv_can, five_hunded_ms_counter);
    if five_hunded_ms_counter < 255 {
        five_hunded_ms_counter = five_hunded_ms_counter + 1;
    } else {
        five_hunded_ms_counter = 0;
    }

    five_hunded_ms_counter
}

pub fn vvvm(hv_can: &HVCAN) {
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
    hv_can.transmit(&vvvm_frame.into()).ok();
}

pub fn tatata(hv_can: &HVCAN, cp_state: &mut CPState) {
    let id: u16 = 0x000;
    let size: u8 = 4;
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
        DoorStateEnum::DoorOpening | DoorStateEnum::DoorClosing => {
            tatata[0] &= 0x00;
        }
        DoorStateEnum::DoorOpen | DoorStateEnum::DoorClosed => {
            // Do nothing.
        }
    }
    hv_can.transmit(&tatata_frame.into()).ok();
}

pub fn u6(hv_can: &HVCAN, five_hunded_ms_counter: u8) {
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
    hv_can.transmit(&u6_frame.into()).ok();
}
