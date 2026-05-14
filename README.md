# embedded-rust-examples

Bare-metal and `no_std` Rust projects for embedded systems — STM32F103 hardware examples and portable library crates.

## STM32F103 examples

| Crate | Description |
|---|---|
| [`hellostm32`](hellostm32/) | Blinky — GPIO output, clock setup |
| [`stm32-interrupt`](stm32-interrupt/) | External interrupt on GPIO pin |
| [`stm32-adc`](stm32-adc/) | ADC read with DMA |
| [`stm32-pwm`](stm32-pwm/) | PWM output via timer |
| [`stm32-l3g4200d-gyro`](stm32-l3g4200d-gyro/) | I2C gyroscope driver |

## Library crates

| Crate | Description |
|---|---|
| [`cobs`](cobs/) | COBS encoder/decoder — `no_std`, zero-alloc, full test suite |
| [`w25qxx`](w25qxx/) | W25Q flash simulator — hardware-accurate write/erase constraints |
