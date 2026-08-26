use defmt::{debug, warn};
use embassy_stm32::gpio::{Flex, Pull, Speed};
use embassy_stm32::peripherals::PC15;
use embassy_time::{Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum Sdi12Command<'a> {
    Ping,
    Scan { start_addr: char, end_addr: char },
    Raw { sdi12_cmd: &'a str },
    Help,
}

#[derive(Debug, PartialEq, defmt::Format)]
pub enum Sdi12Error {
    Timeout,
    InvalidSDI12Command,
    InvalidSerialCommand,
    InvalidResponse,
}

pub struct Sdi12<'a> {
    pin: Flex<'a>,
}

impl<'a> Sdi12<'a> {
    const BIT_US: u64 = 833;

    pub fn new(pin: embassy_hal_internal::Peri<'a, PC15>) -> Self {
        let mut pin = Flex::new(pin);
        pin.set_low();
        pin.set_as_input(Pull::Down);
        Self { pin }
    }

    pub async fn query_device(
        &mut self,
        cmd: &[u8],
        rx_buf: &mut [u8],
    ) -> Result<usize, Sdi12Error> {
        debug!("SDI-12 transaction start: command={=[u8]}", cmd);

        self.pin.set_high();
        self.pin.set_as_output(Speed::Low);
        Self::wait_until(Instant::now() + Duration::from_millis(12));

        self.pin.set_low();
        Self::wait_until(Instant::now() + Duration::from_micros(8_330));

        for &byte in cmd {
            self.write_byte(byte);
        }

        self.pin.set_low();
        self.pin.set_as_input(Pull::Down);
        let result = self.receive_response(rx_buf);
        match result {
            Ok(size) => debug!(
                "SDI-12 response complete: response={=[u8]}",
                &rx_buf[..size]
            ),
            Err(ref error) => warn!("SDI-12 transaction failed: error={:?}", error),
        }
        result
    }

    fn write_byte(&mut self, byte: u8) {
        let parity = (byte & 0x7f).count_ones() & 1 != 0;
        let bits = [
            false,
            byte & 0x01 != 0,
            byte & 0x02 != 0,
            byte & 0x04 != 0,
            byte & 0x08 != 0,
            byte & 0x10 != 0,
            byte & 0x20 != 0,
            byte & 0x40 != 0,
            parity,
            true,
        ];
        let start = Instant::now();

        for (index, bit) in bits.into_iter().enumerate() {
            self.set_logical_level(bit);
            Self::wait_until(start + Duration::from_micros((index as u64 + 1) * Self::BIT_US));
        }
    }

    fn receive_response(&self, rx_buf: &mut [u8]) -> Result<usize, Sdi12Error> {
        let mut index = 0;
        let mut timeout = Duration::from_millis(100);

        loop {
            if self.wait_for_start(timeout).is_err() {
                if index == 0 {
                    warn!("SDI-12 response timeout waiting for first start bit");
                } else {
                    warn!(
                        "SDI-12 inter-character timeout: received={=usize} partial={=[u8]}",
                        index,
                        &rx_buf[..index]
                    );
                }
                return Err(Sdi12Error::Timeout);
            }

            let start = Instant::now();
            let mut byte = 0u8;
            for bit in 0..7 {
                Self::wait_until(start + Duration::from_micros(Self::BIT_US * (3 + 2 * bit) / 2));
                if self.logical_level() {
                    byte |= 1 << bit;
                }
            }

            Self::wait_until(start + Duration::from_micros(Self::BIT_US * 17 / 2));
            let parity = self.logical_level();
            Self::wait_until(start + Duration::from_micros(Self::BIT_US * 19 / 2));
            let expected_parity = byte.count_ones() & 1 != 0;
            if parity != expected_parity {
                warn!(
                    "SDI-12 parity error: byte={=u8:#04x} received={=bool} expected={=bool}",
                    byte, parity, expected_parity
                );
                return Err(Sdi12Error::InvalidResponse);
            }
            if !self.logical_level() {
                warn!("SDI-12 stop-bit error: byte={=u8:#04x}", byte);
                return Err(Sdi12Error::InvalidResponse);
            }

            if index >= rx_buf.len() {
                warn!(
                    "SDI-12 response buffer full: capacity={=usize}",
                    rx_buf.len()
                );
                return Err(Sdi12Error::InvalidResponse);
            }
            rx_buf[index] = byte;
            index += 1;

            if byte == b'\n' {
                return Ok(index);
            }
            timeout = Duration::from_millis(15);
        }
    }

    fn wait_for_start(&self, timeout: Duration) -> Result<(), Sdi12Error> {
        let deadline = Instant::now() + timeout;
        while self.pin.is_high() {
            if Instant::now() >= deadline {
                warn!("SDI-12 bus stayed SPACE/high; never reached MARK/low");
                return Err(Sdi12Error::Timeout);
            }
        }
        while !self.pin.is_high() {
            if Instant::now() >= deadline {
                warn!("SDI-12 bus stayed MARK/low; no response start bit");
                return Err(Sdi12Error::Timeout);
            }
        }
        Ok(())
    }

    fn set_logical_level(&mut self, high: bool) {
        if high {
            self.pin.set_low();
        } else {
            self.pin.set_high();
        }
    }

    fn logical_level(&self) -> bool {
        self.pin.is_low()
    }

    fn wait_until(deadline: Instant) {
        while Instant::now() < deadline {}
    }
}
