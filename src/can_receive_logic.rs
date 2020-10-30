#![deny(warnings)]
use crate::process_cp::init as process_cp;
use crate::process_door::init as process_door;
use crate::types::*;

pub fn init(can_frame: &CanFrame, elapsed: u32, mut cp_state: &mut CPState) {
    if let CanFrame::DataFrame(ref frame) = can_frame {
        let id: u32 = frame.id().into();
        let data = frame.data();

        // Can only say you've gotten a frame, not
        // that you _haven't_ gotten a frame.
        // Timeout needs to be set somewhere else.
        if id == 0x00F {
            cp_state.previous_cptod_ts = elapsed;
            cp_state.cp_comm_timeout = false;
        }

        // Do something about the door
        if id == 0x00F {
            process_door(&mut cp_state, id, data);
        }

        // Main state machine for charge state here
        if id == 0x00F || id == 0x00F {
            process_cp(&mut cp_state, id, data);
        }
    }
}
