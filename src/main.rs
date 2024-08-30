use anyhow::{Ok, Result};
use clap::Parser;
use cli::Args;
use mio_serial::SerialPortBuilderExt;
use serial::at_command;
mod cli;
mod error;
mod serial;

pub fn init_sam201() -> Result<()> {
    Ok(())
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
