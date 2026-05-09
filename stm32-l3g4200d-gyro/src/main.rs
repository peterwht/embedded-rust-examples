#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::i2c::Mode;
use stm32f1xx_hal::{i2c, pac, prelude::*};

// gyroscope slave address. If SDO is set to 3v3 the LSB changes to 1
const GYRO_SLAVE_ADDRESS: u8 = 0b1101000;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let mut afio = dp.AFIO.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpiob = dp.GPIOB.split();

    let i2c1_scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let i2c1_sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let mode = Mode::standard(100.kHz());

    let i2c = i2c::I2c::i2c1(dp.I2C1, (i2c1_scl, i2c1_sda), &mut afio.mapr, mode, clocks);
    let mut blocking = i2c.blocking_default(clocks);

    // write to control register 1 to enable gyro
    let res = blocking.write(GYRO_SLAVE_ADDRESS, &[0x20, 0xF]);
    if let Err(error) = res {
        rprintln!("Error: {:?}", error);
    }

    // enable MSB reading from continuous registers (auto incrementing)
    let auto_increment = 0x28 | 0x80;

    loop {
        let mut xyz = [0u8; 6];
        let res = blocking.write_read(GYRO_SLAVE_ADDRESS, &[auto_increment], &mut xyz);
        if let Err(error) = res {
            rprintln!("Error: {:?}", error);
        }
        let x = i16::from_le_bytes([xyz[0], xyz[1]]);
        let y = i16::from_le_bytes([xyz[2], xyz[3]]);
        let z = i16::from_le_bytes([xyz[4], xyz[5]]);

        // CTRL_REG4 FS1-FS0 default sensitivty is 250 degress per second (DPS).
        // Per table 4 for the l3g4200D gyroscope, for 250 DPS (full scale)
        // the typical specification is 8.75 (milli-DPS / digit)
        let x_dps = x as f32 * 8.75f32 / 1000.0;
        let y_dps = y as f32 * 8.75f32 / 1000.0;
        let z_dps = z as f32 * 8.75f32 / 1000.0;

        rprintln!(
            "(x_dps,y_dps,z_dps): ({:?}, {:?}, {:?})",
            x_dps,
            y_dps,
            z_dps
        );

        for _ in 0..50_000 {
            cortex_m::asm::nop();
        }
    }
}
