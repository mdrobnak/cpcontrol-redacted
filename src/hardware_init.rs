#![deny(warnings)]
use cortex_m::peripheral::NVIC;
extern crate stm32f7xx_hal as hal;
use hal::{interrupt, pac};

pub fn init() {
    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);
    }
    unsafe {
        NVIC::unmask::<interrupt>(interrupt::EXTI2);
    }
}
