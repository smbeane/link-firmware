#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, dma, peripherals, usart};

use panic_probe as _;

use crate::sdi12::Sdi12;

mod sdi12;
mod serial;

bind_interrupts!(pub struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // TODO: rename usart and sdi12?
    let mut usart = Uart::new(
        p.USART2,
        p.PA3,
        p.PA2,
        p.DMA1_CH2,
        p.DMA1_CH3,
        Irqs,
        Config::default(),
    )
    .unwrap();
    let mut sdi12 = Sdi12::new(p.PC15);

    info!("Starting Program...");

    // TODO: what happens if usart errors out?
    loop {
        info!("Reading!");
        match serial::receive(&mut usart, &mut sdi12).await {
            Ok(()) => {
                info!("Received Command");
            }
            Err(e) => {
                warn!("Error, {:?}", e);
            }
        }
    }
}
