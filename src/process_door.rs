#![deny(warnings)]
use crate::types::*;

pub fn init(mut cp_state: &mut CPState, id: u32, data: &[u8]) {
    if id == 0x00F {
        match cp_state.cp_door_state {
            DoorStateEnum::DoorOpenRequest => {
                if (data[1] & 0x00) == 0x00 {
                    cp_state.cp_door_state = DoorStateEnum::DoorOpening;
                }
                // If we power on with mostly open door already, or completely open, this may be
                // the case.
                else if (data[1] & 0x00) == 0x00 {
                    cp_state.cp_door_state = DoorStateEnum::DoorOpen;
                } else if (data[2] & 0x00) == 0x00 {
                    cp_state.cp_door_state = DoorStateEnum::DoorOpen;
                }
            }
            DoorStateEnum::DoorOpening => {
                if (data[1] & 0x00) == 0x00 {
                    cp_state.cp_door_state = DoorStateEnum::DoorOpen;
                }
            }
            DoorStateEnum::DoorCloseRequest => {
                if (data[1] & 0x00) == 0x00 {
                    cp_state.cp_door_state = DoorStateEnum::DoorClosing;
                }
            }
            DoorStateEnum::DoorClosing => {
                if (data[1] & 0x00) == 0x00 {
                    cp_state.cp_door_state = DoorStateEnum::DoorClosed;
                } else if (data[2] & 0x00) == 0x00 {
                    cp_state.cp_door_state = DoorStateEnum::DoorClosed;
                }
            }
            DoorStateEnum::DoorOpen | DoorStateEnum::DoorClosed | DoorStateEnum::DoorIdle => {
                // Do nothing.
            }
        }
    }
}
