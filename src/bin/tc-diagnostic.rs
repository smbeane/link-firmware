#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_stm32::spi::{Config as SpiConfig, MODE_1, Spi};
use embassy_time::Timer;
use panic_probe as _;

#[path = "../max31856.rs"]
mod max31856;

use max31856::Max31856;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut spi_config = SpiConfig::default();
    spi_config.mode = MODE_1;
    let spi = Spi::new_blocking(p.SPI1, p.PA1, p.PA7, p.PA6, spi_config);
    let mut thermocouples = Max31856::new(spi, p.PA4, p.PA5).unwrap();

    info!("Thermocouple diagnostic started");
    loop {
        for channel in 0..2 {
            match thermocouples.read_temperature(channel) {
                Ok((raw, fault)) => {
                    let negative = raw < 0;
                    let magnitude = i64::from(raw).abs();
                    let whole = magnitude / 128;
                    let fraction = (magnitude % 128) * 10_000 / 128;
                    info!(
                        "TC{} temperature_c={}{=i64}.{=u64:04} fault=0x{=u8:02x}",
                        channel,
                        if negative { "-" } else { "" },
                        whole,
                        fraction as u64,
                        fault
                    );
                }
                Err(_) => error!("TC{} SPI error", channel),
            }
        }
        Timer::after_secs(1).await;
    }
}
