#![deny(warnings)]
use crate::types::{BaseID, CPState, CanError, DataFrame, HVCAN, ID};
use crate::utils::checksum_calc;

// Logging
use crate::handle_can_error;
use heapless::consts::U60;
use heapless::String;
use ufmt::uwrite;

pub fn init(elapsed: u32, fifty_ms_ptr: &mut u8, cp_state: &mut CPState, hv_can: &HVCAN) {
    let fifty_ms_counter: u8 = *fifty_ms_ptr;
    let fifty_ms_checksum_count: u8 = fifty_ms_counter % 16;
    p2(hv_can, fifty_ms_checksum_count, cp_state).unwrap_or_else(|error| {
        handle_can_error!(p2, error, "50ms_0", cp_state, elapsed);
    });
    u7(hv_can, fifty_ms_checksum_count).unwrap_or_else(|error| {
        handle_can_error!(u7, error, "50ms_1", cp_state, elapsed);
    });
    if fifty_ms_counter < 255 {
        *fifty_ms_ptr = fifty_ms_counter + 1;
    } else {
        *fifty_ms_ptr = 0;
    }
}

pub fn p2(
    hv_can: &HVCAN,
    fifty_ms_checksum_count: u8,
    cp_state: &mut CPState,
) -> Result<(), CanError> {






    // Checksum correct.
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut p2_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    p2_frame.set_data_length(size.into());
    let p2 = p2_frame.data_as_mut();
    let res = fifty_ms_checksum_count % 2;
    match res {
        0 => {
            p2[0] = 0x00;
            p2[1] = 0x00;
            p2[2] = 0x00;
            p2[3] = 0x00;
            p2[4] = 0x00;
            p2[5] = 0x00;
            p2[6] = 0x00;
            p2[6] |= (fifty_ms_checksum_count + 1) << 4;
        }
        1 | _ => {
            p2[0] = 0x00;
            if cp_state.cp_init {
                p2[1] = 0x00;
            } else {
                p2[1] = 0x00;
            }
            p2[2] = 0x00;
            p2[3] = 0x00;
            p2[4] = 0x00;
            p2[5] = 0x00;
            p2[6] = 0x00;
            p2[6] |= (fifty_ms_checksum_count + 1) << 4;
        }
    }
    p2[7] = 0;
    p2[7] = checksum_calc(p2, id, size);
    hv_can.transmit(&p2_frame.into())
}

pub fn u7(hv_can: &HVCAN, fifty_ms_checksum_count: u8) -> Result<(), CanError> {
    // Checksum correct.
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut u7_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    u7_frame.set_data_length(size.into());
    let u7 = u7_frame.data_as_mut();
    let res = fifty_ms_checksum_count % 2;
    match res {
        0 => {
            u7[0] = 0x00;
            u7[1] = 0x00;
            u7[2] = 0x00;
            u7[3] = 0x00;
            u7[4] = 0x00;
            u7[5] = 0x00;
            u7[6] = 0x00;
            u7[6] |= fifty_ms_checksum_count << 4;
        }
        1 | _ => {
            u7[0] = 0x00;
            u7[1] = 0x00;
            u7[2] = 0x00;
            u7[3] = 0x00;
            u7[4] = 0x00;
            u7[5] = 0x00;
            u7[6] = 0x00;
            u7[6] |= fifty_ms_checksum_count << 4;
        }
    }
    u7[7] = 0;
    u7[7] = checksum_calc(u7, id, size);
    hv_can.transmit(&u7_frame.into())
}
