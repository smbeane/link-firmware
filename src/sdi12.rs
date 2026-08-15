use embassy_stm32::mode::Async;
use embassy_stm32::usart::{ self, Uart, Config,};
use embassy_stm32::gpio::{ Level, Output, Speed };
use embassy_hal_internal::{ Peri };
use embassy_time::{Timer, with_timeout};
use embassy_stm32::peripherals::{ DMA1_CH4, DMA1_CH5, PA9, USART1 };

use crate::Irqs;

#[derive(Debug, PartialEq)]
pub enum Sdi12Command<'a> {
    Ping,
    Scan { start_addr: char, end_addr: char },
    Raw { sdi12_cmd: &'a str },
    Help,
}

#[derive(Debug, PartialEq, defmt::Format)]
pub enum Sdi12Error {
    Timeout,            // Timeout on the SDI12 bus
    InvalidCommand,     // Command doesn't have something like !
    InvalidResponse,    // Response is not readable
    UartError,           // Something wrong with UART
}


pub struct Sdi12<'a> {
    uart: Peri<'a, USART1>,
    pin:  Peri<'a, PA9>,
    tx_dma: Peri<'a, DMA1_CH4>,
    rx_dma: Peri<'a, DMA1_CH5>,
    irqs: Irqs,    
}

impl <'a> Sdi12<'a> {
    pub fn new(pin: Peri<'a, PA9>, 
           uart: Peri<'a, USART1>, 
           tx_dma: Peri<'a, DMA1_CH4>, 
           rx_dma: Peri<'a, DMA1_CH5>,
           irqs: Irqs
        ) -> Self {
        Sdi12 {
            pin,
            uart,
            tx_dma,
            rx_dma,
            irqs,
        }
    }

    pub async fn query_device(&mut self, cmd: &[u8], rx_buf: &mut [u8]) -> Result<usize, Sdi12Error> {
        {
            let mut break_pin = Output::new(self.pin.reborrow(), Level::High, Speed::Low);

            Timer::after_micros(12000).await;

            break_pin.set_low();
            Timer::after_micros(8330).await;
        }

        let mut config = Config::default();
        
        config.baudrate = 1200;
        config.data_bits = usart::DataBits::DataBits7;
        config.parity = usart::Parity::ParityEven;
        config.invert_rx = true;
        config.invert_tx = true;

        let mut uart_sw = Uart::new_half_duplex(
            self.uart.reborrow(), 
            self.pin.reborrow(), 
            self.tx_dma.reborrow(), 
            self.rx_dma.reborrow(), 
            self.irqs,
            config, 
            usart::HalfDuplexReadback::NoReadback
        ).map_err(|_| Sdi12Error::UartError)?;
        
        uart_sw.write(cmd).await.map_err(|_| Sdi12Error::UartError)?;
        Self::receive_response(&mut uart_sw, rx_buf).await
    }
    
    async fn receive_response(uart: &mut Uart<'_, Async>, rx_buf: &mut [u8]) -> Result<usize, Sdi12Error> {
        let mut character = [0u8; 1];
        let mut index: usize = 0;

        let mut timeout = embassy_time::Duration::from_millis(100);

        loop {
            if index >= rx_buf.len() {
                return Err(Sdi12Error::InvalidResponse); // response too long
            }
            
            match with_timeout(timeout, uart.read(&mut character)).await
                .map_err(|_| Sdi12Error::Timeout)? {
                    Ok(()) => {
                        let no_parity = character[0] & 0x7F;

                        rx_buf[index] = no_parity;
                        index += 1;

                        if no_parity == '\n' as u8 {
                            return Ok(index);
                        }

                        timeout = embassy_time::Duration::from_millis(15);


                    },
                    Err(_e) => {
                        return Err(Sdi12Error::UartError);
                    }
            };
        }
    }
}