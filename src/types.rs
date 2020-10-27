#![deny(warnings)]
#[cfg(feature = "nucleof767zi")]
extern crate stm32f7xx_hal as hal;

#[cfg(any(
    feature = "nucleof446re",
    feature = "production",
    feature = "twentyfour",
))]
extern crate stm32f4xx_hal as hal;

use arraydeque::{ArrayDeque, Wrapping};
use core::fmt::Display;
use heapless::consts::*;
use heapless::String;

pub enum DoorStateEnum {
    DoorIdle = 0,
    DoorOpen = 1,
    DoorOpenRequest = 2,
    DoorOpening = 3,
    DoorClosed = 4,
    DoorCloseRequest = 5,
    DoorClosing = 6,
}

#[derive(Copy, Clone)]
pub enum LEDStateEnum {
    WhiteBlue,
    BlueBlink,
    GreenBlink,
    GreenSolid,
    Rainbow,
}

impl Display for ChargeStateEnum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        match self {
            ChargeStateEnum::TimeOut => write!(f, "Timeout"),
            ChargeStateEnum::ChargePortError => write!(f, "Charge Port Error"),
            ChargeStateEnum::ChargeIdle => write!(f, "Idle"),
            ChargeStateEnum::ACBlocked => write!(f, "AC Blocked"),
            ChargeStateEnum::WaitForComms => write!(f, "Wait Comms"),
            ChargeStateEnum::ContactorWaitRequest => write!(f, "Standby"),
            ChargeStateEnum::ContactorRequest => write!(f, "Contactor Request"),
            ChargeStateEnum::ContactorFixed => write!(f, "Charge Enabled"),
            ChargeStateEnum::StopCharge => write!(f, "Stop Charge"),
        }
    }
}
pub enum ChargeStateEnum {
    TimeOut = 0,
    ChargePortError = 1,
    ChargeIdle = 2,
    ACBlocked = 3,
    WaitForComms = 4,
    ContactorWaitRequest = 5,
    ContactorRequest = 6,
    ContactorFixed = 7,
    StopCharge = 8,
}
//pub enum charge_stateText[9][20] =
//{ "Timeout", "Charge Port Error", "Proximity Idle", "AC Blocked",
//"Wait Comms", "Standby", "Contactor Request", "Charge Enabled", "Stop Charge"}

#[derive(PartialEq, Eq)]
pub enum ContactorRequestStateEnum {
    ContactorNone = 0,
    ContactorACRequest = 1,
    ContactorACEnable = 2,
    ContactorDCRequest = 3,
    ContactorDCEnable = 4,
}

#[derive(PartialEq, Eq)]
pub enum ChargerTypeEnum {
    None = 0,
    AC = 1,
    DC = 2,
}

impl Display for ChargerTypeEnum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        match self {
            ChargerTypeEnum::None => write!(f, "None"),
            ChargerTypeEnum::AC => write!(f, "AC  "),
            ChargerTypeEnum::DC => write!(f, "DC  "),
        }
    }
}

pub struct CPState {
    pub activity_list: ArrayDeque<[String<U60>; 4], Wrapping>,
    pub auto_start: bool,
    pub charger_relay_enabled: bool,
    pub charger_type: ChargerTypeEnum,
    pub charge_state: ChargeStateEnum,
    pub cbtxva_request: bool,
    pub contactor_request_state: ContactorRequestStateEnum,
    pub cp_comm_timeout: bool,
    pub cp_door_state: DoorStateEnum,
    pub cp_init: bool,
    pub desired_cp_led_state: LEDStateEnum,
    pub evse_request: bool,
    pub latch_enabled: bool,
    previous_desired_cp_led_state: LEDStateEnum,
    pub print_menu_request: bool,
    pub previous_cptod_ts: u32,
    pub quiet_to_verbose: bool,
    pub tcgz: u8,
    pub vehicle_locked: bool,
    pub verbose_stats: bool,
}

impl CPState {
    pub fn set_led(&mut self, desired: LEDStateEnum) {
        self.previous_desired_cp_led_state = self.desired_cp_led_state;
        self.desired_cp_led_state = desired;
    }
    pub fn set_previous_led(&mut self) {
        self.desired_cp_led_state = self.previous_desired_cp_led_state;
        self.previous_desired_cp_led_state = LEDStateEnum::Rainbow;
    }
    // pub fn new() -> CPState { // we create a method to instantiate `Foo`
    //        CPState {
    pub fn new() -> Self {
        // we create a method to instantiate `Foo`
        Self {
            activity_list: ArrayDeque::new(),
            auto_start: false,
            charger_relay_enabled: false,
            charger_type: ChargerTypeEnum::None,
            charge_state: ChargeStateEnum::StopCharge,
            cbtxva_request: false,
            contactor_request_state: ContactorRequestStateEnum::ContactorNone,
            cp_comm_timeout: true,
            cp_door_state: DoorStateEnum::DoorIdle,
            cp_init: false,
            desired_cp_led_state: LEDStateEnum::WhiteBlue,
            evse_request: false,
            latch_enabled: true,
            previous_desired_cp_led_state: LEDStateEnum::WhiteBlue,
            print_menu_request: false,
            previous_cptod_ts: 0,
            quiet_to_verbose: false,
            tcgz: 0x60,
            vehicle_locked: false,
            verbose_stats: false,
        }
    }
}

// Generic type abstractions
// Why? Remove reference to hal, so that it does not need to be included in many spots with
// conditional code around it.
pub type BaseID = hal::can::BaseID;
pub type CanFrame = hal::can::CanFrame;
pub type DataFrame = hal::can::DataFrame;
pub type ID = hal::can::ID;

// HW specific type abstractions
#[cfg(feature = "nucleof767zi")]
mod abstractions {
    extern crate stm32f7xx_hal as hal;
    use hal::can::Can;
    use hal::gpio::gpiod::{PD0, PD1};
    use hal::gpio::gpiog::{PG2, PG3};
    use hal::gpio::AF9;
    use hal::gpio::{Alternate, Floating, Input, Output, PushPull};
    use hal::pac::CAN1;
    pub type HVCAN = Can<CAN1, (PD1<Alternate<AF9>>, PD0<Alternate<AF9>>)>;
    pub type SerialConsoleOutput = hal::serial::Tx<hal::pac::USART3>;
    pub type FaultLinePin = PG2<Input<Floating>>;
    pub type LatchOutPin = PG3<Output<PushPull>>;
}

#[cfg(feature = "nucleof446re")]
mod abstractions {
    extern crate stm32f4xx_hal as hal;
    use hal::can::Can;
    use hal::gpio::gpiob::{PB3, PB5, PB8, PB9};
    use hal::gpio::AF9;
    use hal::gpio::{Alternate, Floating, Input, Output, PushPull};
    use hal::pac::CAN1;
    pub type HVCAN = Can<CAN1, (PB9<Alternate<AF9>>, PB8<Alternate<AF9>>)>;
    pub type SerialConsoleOutput = hal::serial::Tx<hal::pac::USART2>;
    pub type FaultLinePin = PB3<Input<Floating>>;
    pub type LatchOutPin = PB5<Output<PushPull>>;
}

#[cfg(any(feature = "production", feature = "twentyfour",))]
mod abstractions {
    // Simply a clone of the nucleof446re for the moment.
    extern crate stm32f4xx_hal as hal;
    use hal::can::Can;
    use hal::gpio::gpiob::{PB3, PB5, PB8, PB9};
    use hal::gpio::AF9;
    use hal::gpio::{Alternate, Floating, Input, Output, PushPull};
    use hal::pac::CAN1;
    pub type HVCAN = Can<CAN1, (PB9<Alternate<AF9>>, PB8<Alternate<AF9>>)>;
    pub type SerialConsoleOutput = hal::serial::Tx<hal::pac::USART2>;
    pub type FaultLinePin = PB3<Input<Floating>>;
    pub type LatchOutPin = PB5<Output<PushPull>>;
}

pub type HVCAN = abstractions::HVCAN;
pub type SerialConsoleOutput = abstractions::SerialConsoleOutput;
pub type FaultLinePin = abstractions::FaultLinePin;
pub type LatchOutPin = abstractions::LatchOutPin;
