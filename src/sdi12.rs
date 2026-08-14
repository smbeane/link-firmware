pub enum Sdi12Command<'a> {
    Ping,
    Scan { start_addr: char, end_addr: char },
    Raw { sdi12_cmd: &'a str },
    Help,
}

#[derive(Debug, PartialEq)]
pub enum Sdi12Error {
    Timeout,            // Timeout on the SDI12 bus
    InvalidCommand,     // Command doesn't have something like !
    InvalidResponse,    // Response is not readable
    UartError,           // Something wrong with UART
}

