#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_time::Timer;
use panic_probe as _;

#[path = "../sdi12.rs"]
mod sdi12;

use sdi12::Sdi12;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut sdi12 = Sdi12::new(p.PC15);
    let mut response = [0u8; 16];
    Timer::after_millis(500).await;

    info!("DANGER: changing SDI-12 address 0 to 1; only one soil sensor may be connected");
    match sdi12.query_device(b"0A1!", &mut response).await {
        Ok(length) => info!("address-change response={=[u8]}", &response[..length]),
        Err(_) => error!("address-change command failed"),
    }

    Timer::after_secs(1).await;
    match sdi12.query_device(b"1!", &mut response).await {
        Ok(length) => info!("verification response={=[u8]}", &response[..length]),
        Err(_) => error!("new address 1 did not respond"),
    }

    loop {
        Timer::after_secs(60).await;
    }
}
