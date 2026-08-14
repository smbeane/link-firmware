#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, dma, peripherals, usart};
use panic_probe as _;

mod sdi12;
mod serial;

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<peripherals::DMA1_CH2>, dma::InterruptHandler<peripherals::DMA1_CH3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut usart = Uart::new(p.USART2, p.PA3, p.PA2, p.DMA1_CH2, p.DMA1_CH3, Irqs, Config::default()).unwrap();

    usart.write(b"Hello Embassy World!\r\n").await.unwrap();
    info!("wrote Hello, starting echo");

    let mut rx_buf = [0u8; 64];
    let mut tx_buf = [0u8; 256];

    loop {
        match usart.read_until_idle(&mut rx_buf).await {
            Ok(bytes_read) => {
                // Attempt to parse the incoming bytes as a UTF-8 string
                if let Ok(cmd_str) = core::str::from_utf8(&rx_buf[..bytes_read]) {
                    info!("Host sent command: {}", cmd_str.trim());

                    // Parse the command and let it populate tx_buf
                    match sdi12::handle_uart(cmd_str, &mut tx_buf).await {
                        Ok(size) if size > 0 => {
                            // Success: write the populated buffer back to the host
                            if let Err(e) = usart.write(&tx_buf[..size]).await {
                                warn!("UART TX error: {:?}", e);
                            }
                        }
                        Ok(_) => {
                            // Size was 0, meaning nothing needs to be sent back
                        }
                        Err(e) => {
                            // Pass the usart directly and await the error handler
                            if let Err(e) = serial::handle_error(&mut usart, e).await {
                                warn!("UART Error Reporting failed: {:?}", e);
                            }
                        }
                    }
                } else {
                    warn!("Received non-UTF-8 payload");
                }
            }
            Err(e) => {
                warn!("UART RX error, {:?}", e);
            }
        }
    }
}