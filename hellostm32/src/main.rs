#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

// Memory-mapped register addresses
const RCC_APB2ENR: *mut u32 = 0x40021018 as *mut u32; // RCC APB2 enable register
const GPIOA_CRL: *mut u32 = 0x40010800 as *mut u32;   // GPIOA configuration low
const GPIOA_ODR: *mut u32 = 0x4001080C as *mut u32;   // GPIOA output data

use stm32f1::stm32f103::Peripherals;
use stm32f1xx_hal::{pac, prelude::*};
use stm32f1xx_hal as hal;
#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let mut gpioa = p.GPIOA.split();
    let mut gpioc = p.GPIOC.split();
    let mut led = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);
    let mut button = gpioc.pc13.into_pull_down_input(&mut gpioc.crh);

    loop {
        if button.is_low() {
            led.set_high();
        } else {
            led.set_low();
        }

        for _ in 0..5000 {
            cortex_m::asm::nop();
        }
    }
}