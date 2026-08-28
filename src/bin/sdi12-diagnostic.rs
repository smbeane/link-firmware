#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_time::Timer;
use panic_probe as _;

#[path = "../sdi12.rs"]
mod sdi12;

use sdi12::{Sdi12, Sdi12Error};

fn error_name(error: Sdi12Error) -> &'static str {
    match error {
        Sdi12Error::Timeout => "timeout",
        Sdi12Error::InvalidSDI12Command => "invalid command",
        Sdi12Error::InvalidSerialCommand => "invalid serial command",
        Sdi12Error::InvalidResponse => "invalid/colliding response",
        Sdi12Error::SpiError => "SPI error",
    }
}

async fn query(sdi12: &mut Sdi12<'_>, command: &[u8]) {
    let mut response = [0u8; 64];
    match sdi12.query_device(command, &mut response).await {
        Ok(length) => info!("command={=[u8]} response={=[u8]}", command, &response[..length]),
        Err(error) => warn!("command={=[u8]} error={=str}", command, error_name(error)),
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut sdi12 = Sdi12::new(p.PC15);
    Timer::after_millis(500).await;

    info!("Read-only SDI-12 diagnostic started; no addresses will be changed");
    query(&mut sdi12, b"?!").await;
    for address in b'0'..=b'9' {
        query(&mut sdi12, &[address, b'!']).await;
        query(&mut sdi12, &[address, b'R', b'0', b'!']).await;
    }
    info!("SDI-12 diagnostic complete");

    loop {
        Timer::after_secs(60).await;
    }
}
