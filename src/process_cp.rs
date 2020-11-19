#![deny(warnings)]
use crate::types::*;

pub fn init(mut cp_state: &mut CPState, id: u32, data: &[u8]) {
    // Declare some tests which are useful later on in the code.
    let charge_port_error: bool = (id == 0x00F) && (data[0] == 0x00);
    let charge_idle: bool = (id == 0x00F) && data[0] == 0x00;

    // Main state machine for charge state here
    match cp_state.charge_state {
        ChargeStateEnum::ChargePortError => {
            // States allowed - ChargeIdle, idle
            if charge_idle {
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
            } else if (id == 0x00F) && (data[0] == 0x00) {
                // FIXME: This is probably a bad hack and should be undone.
                cp_state.charge_state = ChargeStateEnum::ACBlocked;
            } else if (id == 0x00F) && (data[4] == 0x00)

            {

            } else if (id == 0x00F) && (data[6] == 0x00)

            {


            } else if charge_port_error

            {
                cp_state.charge_state = ChargeStateEnum::ChargePortError;
            }
        }
        ChargeStateEnum::ACBlocked => {
            let sw_can: bool = id == 0x00F && data[4] == 0x00;
            let j1772: bool = id == 0x00F && (data[0] & 0x00) == 0x00;
            let eu_ac: bool = id == 0x00F && (data[0] & 0x00) == 0x00;

            // States allowed - wait for comms, idle, error
            if sw_can || j1772 || eu_ac {
                cp_state.charge_state = ChargeStateEnum::WaitForComms;
            } else if charge_idle {
                cp_state.charge_state = ChargeStateEnum::ChargeIdle;
            } else if charge_port_error {
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
            } else if charge_idle {
                cp_state.charge_state = ChargeStateEnum::ChargeIdle;
            } else if charge_port_error {
                cp_state.charge_state = ChargeStateEnum::ChargePortError;
            }
        }
        ChargeStateEnum::ContactorWaitRequest => {







            // FIXME : Handle DC better.
            // AC is AC here at this point, it seems.
            if (id == 0x00F) && ((data[4] & 0x00) == 0x00) && (data[1] != 0x00)
            // Charge current set, make contactor request
            {
                if cp_state.auto_start {
                    cp_state.contactor_request_state =
                        ContactorRequestStateEnum::ContactorACRequest;
                }
                cp_state.charger_type = ChargerTypeEnum::AC;


            }
            if (id == 0x00F) && ((data[4] & 0x00) == 0x00) && (data[1] != 0x00)
            // Charge current set, make contactor request
            {
                cp_state.contactor_request_state = ContactorRequestStateEnum::ContactorDCRequest;
                cp_state.charger_type = ChargerTypeEnum::DC;

            }
            if (id == 0x00F) && ((data[0] & 0x00) == 0x00) {
                // Disable...
                cp_state.charger_relay_enabled = false;
                cp_state.charge_state = ChargeStateEnum::StopCharge;
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
            // If EVSE Request or Accept, enable the relay.

            let evse_request: bool = (id == 0x00F) && ((data[0] & 0x00) == 0x00);
            let evse_accept: bool = (id == 0x00F) && ((data[0] & 0x00) == 0x00);
            if evse_request || evse_accept {
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
            }
            cp_state.desired_cp_led_state = LEDStateEnum::WhiteBlue;
            cp_state.charger_type = ChargerTypeEnum::None;
            cp_state.charge_state = ChargeStateEnum::ContactorWaitRequest;

            if charge_idle {
                cp_state.charge_state = ChargeStateEnum::ChargeIdle;
            }
            // FIXME: Shouldn't go to proximity idle - should be able to restart charge
            // or remove cable.
        }
        ChargeStateEnum::TimeOut => {
            // After these messages...
            // States allowed - ChargeIdle, Standby?, ChargePortError
            if charge_idle {
                cp_state.charge_state = ChargeStateEnum::ChargeIdle;
            } else if charge_port_error {
                cp_state.charge_state = ChargeStateEnum::ChargePortError;
            } else if id == 0x00F && !(charge_idle && charge_port_error) {
                cp_state.charge_state = ChargeStateEnum::ContactorWaitRequest;
            }
        }
    }
}
