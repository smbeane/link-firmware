use embassy_stm32::mode::Async;
use embassy_stm32::usart::{ Uart, UartTx, Error };


use crate::sdi12;
use sdi12::{ Sdi12Error, Sdi12Command };


pub async fn receive(usart: &mut Uart<'_, Async>) -> Result<(), Error> {
    let mut rx_buf = [0u8; 64];
    let mut tx_buf = [0u8; 256];

    let (mut usart_tx, usart_rx) = usart.split_ref();

    match usart_rx.read_until_idle(&mut rx_buf).await {
        Ok(bytes_read) => {
            if let Ok(cmd_str) = core::str::from_utf8(&rx_buf[..bytes_read]) {
                match get_response(cmd_str, &mut tx_buf).await {
                    Ok(size) if size > 0 => {
                        if let Err(e) = usart.write(&tx_buf[..size]).await {
                            Err(e)
                        } else {
                            Ok(())
                        }
                    }
                    Ok(_) => {
                        handle_error(&mut usart_tx, Sdi12Error::InvalidCommand).await
                    }
                    Err(e) => {
                        handle_error(&mut usart_tx, e).await
                    }
                }
            } else {
                // TODO: handle this differently, this would be a parsing error
                Err(Error::Framing)

            }
        },
        Err(e) => {
            Err(e)
        }
    }
}

pub async fn get_response<'a>(cmd_str: &'a str, output: &mut [u8]) -> Result<usize, Sdi12Error> {
    let parsed_cmd: Sdi12Command = parse_cmd(cmd_str)?; 

    let response: &[u8] = match parsed_cmd {
        Sdi12Command::Ping => b"PONG\r\n",
        Sdi12Command::Scan { start_addr, end_addr } => {
            // INSERT SCAN FUNCTION CALL HERE
            // but for now I am just going to return 
            // so that I can test the parser
            b"SCAN handler\r\n"
        },
        Sdi12Command::Raw { sdi12_cmd } => {
            // INSERT RAW FUNCTION CALL HERE
            // again, testing parser
            b"RAW handler\r\n"
        },
        Sdi12Command::Help => b"COMMANDS: PING, SCAN <START>,<END>, RAW <SDI_CMD>\r\n",
    };

    if output.len() < response.len() {
        return Err(Sdi12Error::UartError);
    }

    output[..response.len()].copy_from_slice(response);

    Ok(response.len())
}


pub fn parse_cmd<'a>(cmd: &'a str) -> Result<Sdi12Command<'a>, Sdi12Error>{
    let cmd = cmd.trim();

    if cmd.is_empty() {
        return Err(Sdi12Error::InvalidCommand);
    }

    let (instruction, args) = match cmd.split_once(' ') {
        Some((inst, rest)) => (inst, rest.trim()),
        None => (cmd, ""),
    };
    
    match instruction {
        "PING" => Ok(Sdi12Command::Ping),
        
        "SCAN" => {
            let (start, end) = match args.split_once(',') {
                Some((s, e)) => (s.trim(), e.trim()),
                None => ("", ""),
            };

            let start_addr: char = start.chars().next().unwrap_or(' ');
            let end_addr: char = end.chars().next().unwrap_or(' ');
            Ok(Sdi12Command::Scan{ start_addr, end_addr })
        }
        
        "RAW" => Ok(Sdi12Command::Raw{sdi12_cmd: args}),
        
        
        "HELP" => Ok(Sdi12Command::Help),
        
        _ => Err(Sdi12Error::InvalidCommand),

    }
}

async fn handle_error(tx: &mut UartTx<'_, Async>, error: Sdi12Error) -> Result<(), Error> {
    match error {
        Sdi12Error::Timeout => tx.write(b"SDI12 Timeout\r\n").await,
        Sdi12Error::InvalidCommand => tx.write(b"Invalid SDI12 Command\r\n").await,
        Sdi12Error::InvalidResponse => tx.write(b"Invalid SDI12 Response\r\n").await,
        Sdi12Error::UartError => tx.write(b"UART Error\r\n").await,
    }
}


