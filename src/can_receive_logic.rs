#![deny(warnings)]
use crate::types::*;


pub fn init(can_frame: &CanFrame, elapsed: u32, mut cp_state: CPState) -> CPState {
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

        // Main state machine for charge state here
        match cp_state.charge_state {
            ChargeStateEnum::ChargePortError => {
                // States allowed - ChargeIdle, idle
                if (id == 0x00F) && (data[0] == 0x00) {
                    cp_state.charge_state = ChargeStateEnum::ChargeIdle;
                } else if id == 0x00F {
                    // Do nothing.
                } else {

                }
            }
            ChargeStateEnum::ChargeIdle => {
                // States allowed - ACBlocked, Standby (? - if we have comms already?), Error state
                // if we lose ground.
                if (id == 0x00F) && (data[6] == 0x00) && (data[4] == 0x00) {
                    cp_state.charge_state = ChargeStateEnum::ACBlocked;
                } else if (id == 0x00F) && (data[4] == 0x00)

                {

                } else if (id == 0x00F) && (data[6] == 0x00)

                {


                } else if (id == 0x00F) && (data[0] == 0x00)

                {
                    cp_state.charge_state = ChargeStateEnum::ChargePortError;
                }
            }
            ChargeStateEnum::ACBlocked => {
                // States allowed - wait for comms, proxdetect, idle
                if (id == 0x00F) && (data[4] == 0x00) {

                    cp_state.charge_state = ChargeStateEnum::WaitForComms;
                } else if (id == 0x00F) && (data[0] & 0x00) == 0x00 {

                    cp_state.charge_state = ChargeStateEnum::WaitForComms;
                } else if (id == 0x00F) && data[0] == 0x00 {
                    cp_state.charge_state = ChargeStateEnum::ChargeIdle;
                } else if (id == 0x00F) && data[0] == 0x00 {
                    cp_state.charge_state = ChargeStateEnum::ChargePortError;
                }
                if (id == 0x00F) && ((data[0] & 0x00) == 0x00)

                {

                }
            }
            ChargeStateEnum::WaitForComms => {
                // Comms here can be either SWCAN, or standard charging.
                // Standard charging will sit in Wait Request until current is non-zero.
                // SWCAN will start nearly immediately if auto start...
                if (id == 0x00F) && ((data[0] & 0x00) == 0x00)

                {

                }
                if (id == 0x00F) && ((data[4] & 0x00) == 0x00) {

                    cp_state.charge_state = ChargeStateEnum::ContactorWaitRequest;
                } else if (id == 0x00F) && data[3] != 0x00 {

                    cp_state.charge_state = ChargeStateEnum::ContactorWaitRequest;
                } else if (id == 0x00F) && data[0] == 0x00 {
                    cp_state.charge_state = ChargeStateEnum::ChargeIdle;
                } else if (id == 0x00F) && data[0] == 0x00 {
                    cp_state.charge_state = ChargeStateEnum::ChargePortError;
                }
            }
            ChargeStateEnum::ContactorWaitRequest => {







                // FIXME : Handle DC better.
                // AC is AC here at this point, it seems.
                if (id == 0x00F) && ((data[4] & 0x00) == 0x00) && (data[1] != 0x00)
                // Charge current set, make contactor request
                {
                    cp_state.contactor_request_state =
                        ContactorRequestStateEnum::ContactorACRequest;
                    cp_state.charger_type = ChargerTypeEnum::AC;


                }
                if (id == 0x00F) && ((data[4] & 0x00) == 0x00) && (data[1] != 0x00)
                // Charge current set, make contactor request
                {
                    cp_state.contactor_request_state =
                        ContactorRequestStateEnum::ContactorDCRequest;
                    cp_state.charger_type = ChargerTypeEnum::DC;

                }
            }
            ChargeStateEnum::ContactorRequest => {

                if (id == 0x00F)
                    && (data[1] != 0x00)
                    && (cp_state.contactor_request_state
                        == ContactorRequestStateEnum::ContactorACEnable)
                // Charge current set, make contactor request
                {

                    cp_state.charge_state = ChargeStateEnum::ContactorFixed;
                    if cp_state.auto_start {
                        cp_state.charger_relay_enabled = true;
                    }
                } else if (id == 0x00F)
                    && (data[1] == 0x00)
                    && (cp_state.contactor_request_state
                        == ContactorRequestStateEnum::ContactorDCEnable)
                // Charge current set, make contactor request
                {

                    cp_state.charge_state = ChargeStateEnum::ContactorFixed;
                }
            }
            ChargeStateEnum::ContactorFixed => {
                // If EVSE Request, enable the relay.
                if (id == 0x00F) && ((data[0] & 0x00) == 0x00) {
                    if cp_state.charger_type == ChargerTypeEnum::AC {
                        cp_state.charger_relay_enabled = true;
                        cp_state.desired_cp_led_state = LEDStateEnum::GreenBlink;
                    } else if cp_state.charger_type == ChargerTypeEnum::DC {
                    }
                }
                if (id == 0x00F) && ((data[0] & 0x00) == 0x00) {
                    // Disable...
                    cp_state.charger_relay_enabled = false;
                    cp_state.charge_state = ChargeStateEnum::StopCharge;
                }
            }
            ChargeStateEnum::StopCharge => {
                // States allowed - ChargeIdle (if cable removed), Standby
                if cp_state.contactor_request_state != ContactorRequestStateEnum::ContactorNone {
                    cp_state.contactor_request_state = ContactorRequestStateEnum::ContactorNone;
                    cp_state.cbtxva_request = false;
                    cp_state.desired_cp_led_state = LEDStateEnum::WhiteBlue;
                    cp_state.charger_type = ChargerTypeEnum::None;
                }
                if (id == 0x00F) && (data[0] == 0x00) {
                    cp_state.charge_state = ChargeStateEnum::ChargeIdle;
                }
                // FIXME: Shouldn't go to proximity idle - should be able to restart charge
                // or remove cable.
            }
            ChargeStateEnum::TimeOut => {
                // After these messages...
                // States allowed - ChargeIdle, Standby?, ChargePortError
                if (id == 0x00F) && (data[0] == 0x00) {
                    cp_state.charge_state = ChargeStateEnum::ChargeIdle;
                } else if id == 0x00F {
                    cp_state.charge_state = ChargeStateEnum::ChargePortError;
                }
            }
        }
    }
    cp_state
}
