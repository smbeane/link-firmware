#![no_std]
#![no_main]

use defmt::{info, warn};
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
    let mut response = [0u8; 64];
    Timer::after_millis(500).await;

    warn!("BEST-EFFORT ONLY: shared-address responses may collide, corrupt, or represent only one sensor");
    loop {
        for address in b'0'..=b'2' {
            let command = [address, b'R', b'0', b'!'];
            match sdi12.query_device(&command, &mut response).await {
                Ok(length) => warn!(
                    "UNVERIFIED address={=u8} raw={=[u8]}",
                    address - b'0',
                    &response[..length]
                ),
                Err(_) => info!("address={=u8} no valid response", address - b'0'),
            }
            Timer::after_millis(500).await;
        }
        Timer::after_secs(10).await;
    }
}
