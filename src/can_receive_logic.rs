#![deny(warnings)]
use crate::process_cp::init as process_cp;
use crate::process_door::init as process_door;
use crate::process_init::init as process_init;
use crate::process_version::init as process_version;
use crate::types::*;

// Logging
use heapless::consts::U60;
use heapless::String;
use ufmt::uwrite;

pub fn init(can_frame: &CanFrame, elapsed: u32, mut cp_state: &mut CPState) {
    if let CanFrame::DataFrame(ref frame) = can_frame {
        let id: u32 = frame.id().into();
        let data = frame.data();

        // Can only say you've gotten a frame, not
        // that you _haven't_ gotten a frame.
        // Timeout needs to be set somewhere else.
        if id == 0x00E || id == 0x00F {
            if cp_state.cp_comm_timeout {
                let mut s: String<U60> = String::new();
                uwrite!(
                    s,
                    "{} - Timeout recovery after {}",
                    elapsed,
                    (elapsed - cp_state.previous_cptod_ts)
                )
                .ok();
                cp_state.activity_list.push_back(s);
            }

            cp_state.previous_cptod_ts = elapsed;
            cp_state.cp_comm_timeout = false;
        }

        // Handle init.
        if id == 0x00E || (id == 0x00F && cp_state.cp_init == false && cp_state.init_sequence == 3)
        {
            // Handle a hello frame or attempt to pick up from a happy CP ECU.
            cp_state.init_sequence = 0;
            process_init(&mut cp_state, elapsed);
        }

        // Handle version.
        if id == 0x00E {
            process_version(&mut cp_state, id, data);
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
