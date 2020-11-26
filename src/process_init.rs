#![deny(warnings)]
use crate::types::*;

pub fn init(mut cp_state: &mut CPState, elapsed: u32) {
    let ph1_ts = 250;
    let ph2_ts = 5000;

    match cp_state.init_sequence {
        0 => {
            cp_state.init_sequence = 1;
            cp_state.init_ts = elapsed;
            cp_state.cp_comm_timeout = false;
            cp_state.previous_cptod_ts = elapsed;
            cp_state.charge_state = ChargeStateEnum::Init;
        }
        1 => {
            if (elapsed - cp_state.init_ts) >= ph1_ts {
                cp_state.init_ts = elapsed;
                cp_state.init_sequence = 2;
            }
        }
        2 => {
            if (elapsed - cp_state.init_ts) >= ph2_ts && cp_state.cp_init == false {
                cp_state.init_ts = elapsed;
                cp_state.init_sequence = 2;
                cp_state.cp_init = true;
            }
        }
        3 => {
            // Haven't seen an init frame yet.
        }
        _ => {
            // Shouldn't be here.
        }
    }
}
