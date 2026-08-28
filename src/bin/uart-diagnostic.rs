#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, dma, peripherals, usart};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    LPUART1 => usart::InterruptHandler<peripherals::LPUART1>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut uart = Uart::new(
        p.LPUART1,
        p.PA3,
        p.PA2,
        p.DMA1_CH2,
        p.DMA1_CH3,
        Irqs,
        Config::default(),
    )
    .unwrap();

    info!("UART diagnostic started on LPUART1 PA2/PA3 at 115200 8N1");
    loop {
        let _ = uart.write(b"STM32_UART_DIAGNOSTIC\r\n").await;
        Timer::after_secs(1).await;
    }
}
