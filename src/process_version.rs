#![deny(warnings)]
use crate::types::*;

pub fn init(mut cp_state: &mut CPState, id: u32, data: &[u8]) {
    let fw_2020_versions: [u32; 3] = [0x5860BCDF, 0xBB6B3E7E, 0x4B773D16];
    let fw_2019_versions: [u32; 1] = [0x27CE851A];
    let fw_2018_versions: [u32; 1] = [0xAC5A15C4];
    if id == 0x00E {
        if data[0] == 0x00 {
            let app_crc: u32 = (data[7] as u32) << 24
                | (data[6] as u32) << 16
                | (data[5] as u32) << 8
                | data[4] as u32;
            cp_state.app_hash = app_crc;
        }
        if fw_2020_versions.contains(&cp_state.app_hash) {
            cp_state.fw_ver = CPVerEnum::Fw2020;
        } else if fw_2019_versions.contains(&cp_state.app_hash) {
            cp_state.fw_ver = CPVerEnum::Fw2019;
        } else if fw_2018_versions.contains(&cp_state.app_hash) {
            cp_state.fw_ver = CPVerEnum::Fw2018;
        }
    }
}
