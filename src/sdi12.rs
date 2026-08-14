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

pub async fn handle_uart<'a>(cmd_str: &'a str, output: &mut [u8]) -> Result<usize, Sdi12Error> {
    let parsed_cmd: Sdi12Command = parse_serial_cmd(cmd_str)?; 

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

pub fn parse_serial_cmd<'a>(cmd: &'a str) -> Result<Sdi12Command<'a>, Sdi12Error>{
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