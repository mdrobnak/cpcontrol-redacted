#![deny(warnings)]
use crate::types::{BaseID, CPState, DataFrame, HVCAN, ID};

pub fn init(mut thousand_ms_counter: u8, mut cp_state: &mut CPState, hv_can: &HVCAN) -> u8 {
    usx(hv_can);
    tto(hv_can);
    rss(hv_can);
    u5(hv_can);
    tcgd(&mut cp_state, hv_can);

    // PG2 for read?
    // PG3 for write?
    /*
    faultLineVoltage = analogRead(IN1);
    if ((faultLineVoltage <= 6) and (cp_init == true))
    {
    initCount++;
    }
    else if (initCount > 8)
    {
    cp_init = false;
    initCount = 0;
    }
    return faultLineVoltage;
    */
    if thousand_ms_counter < 255 {
        thousand_ms_counter = thousand_ms_counter + 1;
    } else {
        thousand_ms_counter = 0;
    }
    if cp_state.tcgz < 255 {
        cp_state.tcgz = cp_state.tcgz + 1;
    } else {
        cp_state.tcgz = 0x60;
    }

    thousand_ms_counter
}
pub fn usx(hv_can: &HVCAN) {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut usx_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    usx_frame.set_data_length(size.into());
    let usx = usx_frame.data_as_mut();
    usx[0] = 0x00;
    usx[1] = 0x00;
    usx[2] = 0x00;
    usx[3] = 0x00;
    usx[4] = 0x00;
    usx[5] = 0x00;
    usx[6] = 0x00;
    usx[7] = 0x00;
    hv_can.transmit(&usx_frame.into()).ok();
}

pub fn tto(hv_can: &HVCAN) {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut tto_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    tto_frame.set_data_length(size.into());
    let tto = tto_frame.data_as_mut();
    tto[0] = 0x00;
    tto[1] = 0x00;
    tto[2] = 0x00;
    tto[3] = 0x00;
    tto[4] = 0x00;
    tto[5] = 0x00;
    tto[6] = 0x00;
    tto[7] = 0x00;
    hv_can.transmit(&tto_frame.into()).ok();
}

pub fn rss(hv_can: &HVCAN) {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut rss_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    rss_frame.set_data_length(size.into());
    let rss = rss_frame.data_as_mut();
    rss[0] = 0x00;
    rss[1] = 0x00;
    rss[2] = 0x00;
    rss[3] = 0x00;
    rss[4] = 0x00;
    rss[5] = 0x00;
    rss[6] = 0x00;
    rss[7] = 0x00;
    hv_can.transmit(&rss_frame.into()).ok();
}

pub fn u5(hv_can: &HVCAN) {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut u5_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    u5_frame.set_data_length(size.into());
    let u5 = u5_frame.data_as_mut();
    u5[0] = 0x00;
    u5[1] = 0x00;
    u5[2] = 0x00;
    u5[3] = 0x00;
    u5[4] = 0x00;
    u5[5] = 0x00;
    u5[6] = 0x00;
    u5[7] = 0x00;
    hv_can.transmit(&u5_frame.into()).ok();
}

pub fn tcgd(cp_state: &mut CPState, hv_can: &HVCAN) {
    let id: u16 = 0x000;
    let size: u8 = 8;
    let mut tcgd_frame = DataFrame::new(ID::BaseID(BaseID::new(id)));
    tcgd_frame.set_data_length(size.into());
    let tcgd = tcgd_frame.data_as_mut();
    tcgd[0] = cp_state.tcgz;
    tcgd[1] = 0x00;
    tcgd[2] = 0x00;
    tcgd[3] = 0x00;
    tcgd[4] = 0x00;
    tcgd[5] = 0x00;
    tcgd[6] = 0x00;
    tcgd[7] = 0x00;
    hv_can.transmit(&tcgd_frame.into()).ok();
}
