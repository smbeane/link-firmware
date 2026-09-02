#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::spi::{Config as SpiConfig, MODE_1, Spi};
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::{bind_interrupts, dma, peripherals, usart};
use embassy_time::Timer;

use panic_probe as _;

use crate::max31856::Max31856;
use crate::sdi12::Sdi12Bitbang;

mod max31856;
mod sdi12;
mod serial;


// interrupts for serial bus
bind_interrupts!(pub struct Irqs {
    USART1=> usart::InterruptHandler<peripherals::USART1>;
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
    DMA1_CH4_5_DMAMUX1_OVR => dma::InterruptHandler<peripherals::DMA1_CH4>, dma::InterruptHandler<peripherals::DMA1_CH5>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut watchdog = IndependentWatchdog::new(p.IWDG, 15_000_000);
    watchdog.unleash();
    spawner.spawn(feed_watchdog(watchdog).unwrap());

    let mut serial_bus = Uart::new(
        p.USART2,
        p.PA3,
        p.PA2,
        p.DMA1_CH2,
        p.DMA1_CH3,
        Irqs,
        Config::default(),
    )
    .unwrap();

    let mut sdi12_bus = Sdi12Bitbang::new(p.PC15.into());
    
    let mut spi_config = SpiConfig::default();
    spi_config.mode = MODE_1;
    let spi = Spi::new_blocking(p.SPI1, p.PA1, p.PA7, p.PA6, spi_config);
    let mut thermocouples = Max31856::new(spi, p.PA4, p.PA5).unwrap();
    Timer::after_millis(250).await;

    info!("Starting Program...");

    // TODO: what happens if usart errors out?
    loop {
        info!("Reading!");
        match serial::receive(&mut serial_bus, &mut sdi12_bus, &mut thermocouples).await {
            Ok(()) => {
                info!("Received Command");
            }
            Err(e) => {
                warn!("Error, {:?}", e);
            }
        }
    }
}

#[embassy_executor::task]
async fn feed_watchdog(mut watchdog: IndependentWatchdog<'static, peripherals::IWDG>) -> ! {
    loop {
        watchdog.pet();
        Timer::after_secs(5).await;
    }
}
