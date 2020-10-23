//#![deny(warnings)]
#[cfg(feature = "nucleo767zi")]
extern crate stm32f7xx_hal as hal;

#[cfg(feature = "f4board")]
extern crate stm32f4xx_hal as hal;

use cortex_m::peripheral::NVIC;
use hal::{interrupt, pac};

pub fn init() {
    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);
    }
    #[cfg(feature = "nucleo767zi")]
    unsafe {
        NVIC::unmask::<interrupt>(interrupt::EXTI2);
    }
    #[cfg(feature = "f4board")]
    unsafe {
        NVIC::unmask::<interrupt>(interrupt::EXTI3);
    }
}
