use std::io::{Read, Write};

use anyhow::{bail, Result};
use clap::Parser;
use cli::Args;
use log::debug;
use mio::{Events, Interest, Poll, Token};
use mio_serial::{SerialPortBuilderExt, SerialStream};
mod cli;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GsmError {
    #[error("AT command failed: {0}")]
    AtCommandFailed(String),
    #[error("AT command timed out: {0}")]
    AtCommandTimedOut(String),
}
const SERIAL_TOKEN: Token = Token(0);

pub fn at_command(ss: &mut SerialStream, command: &str, timeout_ms: u32) -> Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1);
    poll.registry()
        .register(ss, SERIAL_TOKEN, Interest::READABLE)?;

    let mut buf = vec![0u8; 1024];
    let timeout = Some(std::time::Duration::from_millis(timeout_ms as u64));

    debug!(
        "Sending AT command: {:02X?} -> {}",
        command.as_bytes(),
        command
    );
    ss.write_all(command.as_bytes())?;

    for _ in 0..100 {
        poll.poll(&mut events, timeout)?;
        for event in events.iter() {
            match event.token() {
                SERIAL_TOKEN => {
                    let n = ss.read(&mut buf)?;
                    let response = std::str::from_utf8(&buf[..n])?;
                    debug!("Received {} bytes: {:02X?} -> {}", n, &buf[..n], response);
                    if response.contains("OK") {
                        return Ok(());
                    } else if response.contains("ERROR") {
                        return Err(GsmError::AtCommandFailed(command.to_string()).into());
                    }
                }
                _ => {}
            }
        }
    }
    bail!(GsmError::AtCommandTimedOut(command.to_string()))
}

fn main() -> Result<()> {
    let args = Args::parse();
    let log_level = match args.verbose {
        0 => log::Level::Error,
        1 => log::Level::Info,
        2 => log::Level::Debug,
        _ => log::Level::Trace,
    };
    simple_logger::init_with_level(log_level).unwrap();

    let port = args.port.clone();
    let baud = args.baud;

    let mut ss = mio_serial::new(port, baud).open_native_async().unwrap();
    at_command(&mut ss, "AT\r\n", 1000).unwrap();

    Ok(())
}
