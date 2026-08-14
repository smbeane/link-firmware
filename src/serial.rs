use embassy_stm32::mode::Async;
use embassy_stm32::usart::{ Uart, Error };
use crate::sdi12::Sdi12Error;


pub async fn handle_error(usart: &mut Uart<'_, Async>, error: Sdi12Error) -> Result<(), Error> {
    match error {
        Sdi12Error::Timeout => usart.write(b"SDI12 Timeout\r\n").await,
        Sdi12Error::InvalidCommand => usart.write(b"Invalid SDI12 Command\r\n").await,
        Sdi12Error::InvalidResponse => usart.write(b"Invalid SDI12 Response\r\n").await,
        Sdi12Error::UartError => usart.write(b"UART Error\r\n").await,
    }
}
