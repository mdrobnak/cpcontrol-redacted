#![deny(warnings)]
extern crate stm32f7xx_hal as hal;
use crate::types::HVCAN;
use hal::can::{BaseID, DataFrame, ID};
use heapless::consts::*;
use heapless::String;

pub fn init(mut two_fifty_ms_counter: u8, hv_can: &HVCAN) -> u8 {
    vin405(hv_can, two_fifty_ms_counter);
    if two_fifty_ms_counter < 255 {
        two_fifty_ms_counter = two_fifty_ms_counter + 1;
    } else {
        two_fifty_ms_counter = 0;
    }

    two_fifty_ms_counter
}

pub fn vin405(hv_can: &HVCAN, two_fifty_ms_counter: u8) {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut vin405_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    vin405_frame.set_data_length(size.into());
    let vin405 = vin405_frame.data_as_mut();
    let res = two_fifty_ms_counter % 3;

    let vin: String<U18> = String::from("5Y123456789012345");

    match res {
        0 => {
            vin405[0] = 0x00;
            vin405[1] = 0;
            vin405[2] = 0;
            vin405[3] = 0;
            vin405[4] = 0;
            vin405[5] = vin.bytes().nth(0).unwrap();
            vin405[6] = vin.bytes().nth(1).unwrap();
            vin405[7] = vin.bytes().nth(2).unwrap();
        }
        1 => {
            vin405[0] = 0x00;
            vin405[1] = vin.bytes().nth(3).unwrap();
            vin405[2] = vin.bytes().nth(4).unwrap();
            vin405[3] = vin.bytes().nth(5).unwrap();
            vin405[4] = vin.bytes().nth(6).unwrap();
            vin405[5] = vin.bytes().nth(7).unwrap();
            vin405[6] = vin.bytes().nth(8).unwrap();
            vin405[7] = vin.bytes().nth(9).unwrap();
        }
        2 | _ => {
            vin405[0] = 0x00;
            vin405[1] = vin.bytes().nth(10).unwrap();
            vin405[2] = vin.bytes().nth(11).unwrap();
            vin405[3] = vin.bytes().nth(12).unwrap();
            vin405[4] = vin.bytes().nth(13).unwrap();
            vin405[5] = vin.bytes().nth(14).unwrap();
            vin405[6] = vin.bytes().nth(15).unwrap();
            vin405[7] = vin.bytes().nth(16).unwrap();
        }
    }

    hv_can.transmit(&vin405_frame.into()).ok();
}
