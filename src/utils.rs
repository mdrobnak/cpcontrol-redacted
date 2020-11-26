#![deny(warnings)]
use crate::types::*;

pub fn checksum_calc(data: &[u8], id: u16, size: u8) -> u8 {
    let mut checksum_calc: u16 = 0;
    for i in 0..size - 1 {
        checksum_calc = checksum_calc + data[i as usize] as u16;
    }
    checksum_calc += id + (id >> 8);
    checksum_calc &= 0x00;
    checksum_calc as u8
}

pub fn send_empty_message(hv_can: &HVCAN, id: u16, size: u8) {
    let mut empty_message_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    empty_message_frame.set_data_length(size.into());
    let empty_message = empty_message_frame.data_as_mut();
    empty_message[0] = 0x00;
    empty_message[1] = 0x00;
    empty_message[2] = 0x00;
    empty_message[3] = 0x00;
    empty_message[4] = 0x00;
    empty_message[5] = 0x00;
    empty_message[6] = 0x00;
    empty_message[7] = 0x00;
    hv_can.transmit(&empty_message_frame.into()).ok();
}
