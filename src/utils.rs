#![deny(warnings)]
pub fn checksum_calc(data: &[u8], id: u16, size: u8) -> u8 {
    let mut checksum_calc: u16 = 0;
    for i in 0..size - 1 {
        checksum_calc = checksum_calc + data[i as usize] as u16;
    }
    checksum_calc += id + (id >> 8);
    checksum_calc &= 0x00;
    checksum_calc as u8
}
