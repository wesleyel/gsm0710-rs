use anyhow::{bail, Ok, Result};
use clap::Parser;
use cli::{Args, ModemType};
use log::info;
use mio_serial::{SerialPortBuilderExt, SerialStream};
use serial::{at_command, openpty};
mod cli;
mod error;
mod serial;
mod buffer;

pub fn init_sam201(ss: &mut SerialStream) -> Result<()> {
    const MUX_CMD: &str = "AT+CMUX=1\r\n";
    const HOLA_CMD: &str = "AT\r\n";

    info!("Initializing SAM-201 modem");
    at_command(ss, HOLA_CMD, 1000)?;
    at_command(ss, MUX_CMD, 1000)?;
    info!("SAM-201 modem initialized");
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

    // TODO: init buffer

    let mut ptys = Vec::new();
    for idx in 0..args.channels {
        ptys.push(openpty(args.clone().pty, idx, args.clone().symlink_prefix)?);
    }
    info!("Opened {} PTYs", ptys.len());
    let mut ss = mio_serial::new(args.clone().port, args.baud)
        .open_native_async()
        .unwrap();
    info!("Opened serial port {}", args.clone().port);

    match args.modem {
        ModemType::Sam201 => init_sam201(&mut ss)?,
        _ => bail!("Unsupported modem type"),
    }

    // TODO: init mux and open control channel and logical channels

    // TODO: Daemon here

    // TODO: main loop for catching events

    Ok(())
}
