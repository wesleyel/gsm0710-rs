use std::{
    io::{Read, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
use buffer::{GSM0710Buffer, GSM0710_BUFFER_CAPACITY};
use clap::Parser;
use cli::{Args, ModemType};
use error::GsmError;
use log::{debug, info};
use mio::{Events, Poll, Token};
use mio_serial::{SerialPortBuilderExt, SerialStream};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use serial::{at_command, openpty, PtyStream, PtyWriteFrame};
use types::{AddressImpl, ControlImpl, Frame, FrameType, CR, C_CLD};
mod buffer;
mod cli;
mod error;
mod serial;
mod types;

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
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;

    let mut buffer = AllocRingBuffer::<u8>::new(GSM0710_BUFFER_CAPACITY);
    info!("Initialized buffer with capacity {}", buffer.capacity());

    let mut ptys = Vec::new();
    for idx in 0..args.channels {
        let pty = openpty(args.clone().pty, idx, args.clone().symlink_prefix)?;
        ptys.push(PtyStream::new(pty));
    }
    info!("Opened {} PTYs", ptys.len());

    let mut ss = mio_serial::new(args.clone().port, args.baud)
        .open_native_async()
        .unwrap();
    info!("Opened serial port {}", args.clone().port);

    match args.modem {
        ModemType::Sam201 => init_sam201(&mut ss)?,
        _ => return Err(GsmError::UnsupportedModemType(args.modem.to_string()).into()),
    }
    info!("Modem {} initialized", args.modem);

    let addr = 0u8.with_cr(true).with_ea(true).with_dlci(0);
    let ctrl = 0u8.with_pf(true).with_frame(FrameType::SABM);
    let mut frame = Frame::new(addr, ctrl, 0, vec![0]);
    ptys.iter_mut().enumerate().for_each(|(idx, pty)| {
        debug!("Sending SABM frame to PTY {}", idx);
        if idx == 0 {
            frame.address.set_dlci(idx as u8);
            pty.write_frame(frame.clone()).unwrap();
        } else {
            frame.address.set_dlci(idx as u8);
            pty.write_frame(frame.clone()).unwrap();
        }
    });
    info!("Sent SABM frames to all PTYs");

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(ptys.len() + 1);

    // Register the serial port and all PTYs with the poller
    poll.registry()
        .register(&mut ss, Token(0), mio::Interest::READABLE)?;
    for (idx, pty) in ptys.iter_mut().enumerate() {
        poll.registry()
            .register(pty, Token(idx + 1), mio::Interest::READABLE)?;
    }

    while !term.load(Ordering::Relaxed) {
        poll.poll(&mut events, Some(Duration::from_secs(1)))?;
        for event in events.iter() {
            match event.token() {
                Token(0) => {
                    let mut buf = vec![0u8; 1024];
                    let n = ss.read(&mut buf)?;
                    debug!(
                        "Received {} bytes: {:02X?} from {}",
                        n,
                        &buf[..n],
                        args.clone().port
                    );
                    buffer.push_vec((&buf[..n]).to_vec());
                    loop {
                        match buffer.pop_frame1() {
                            Some(frame) => {
                                todo!("Handle frame: {:?}", frame);
                            }
                            None => break,
                        }
                    }
                }
                Token(idx) => {
                    let mut buf = vec![0u8; 1024];
                    let n = ptys[idx - 1].inner.read(&mut buf)?;
                    debug!(
                        "Received {} bytes from PTY {}: {:02X?}",
                        n,
                        idx - 1,
                        &buf[..n]
                    );

                    let frame = Frame::new(
                        addr.with_dlci((idx + 1) as u8),
                        ctrl.with_frame(FrameType::UIH),
                        n as u16,
                        buf[..n].to_vec(),
                    );
                    let data = frame.try_to_bytes()?;
                    match ss.write(&data) {
                        Ok(_) => debug!("Sent {} bytes to serial port: {:02X?}", data.len(), &data),
                        Err(e) => {
                            info!("Error sending data to serial port: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    }
    info!("Closing logical channels");
    ptys.iter_mut().enumerate().for_each(|(idx, pty)| {
        debug!("Sending DISC frame to PTY {}", idx);
        if idx != 0 {
            let frame = Frame::new(
                addr.with_dlci(idx as u8),
                ctrl.with_frame(FrameType::DISC),
                0,
                vec![0],
            );
            pty.write_frame(frame.clone()).unwrap();
        }
    });
    info!("Closing control channel");
    ptys.iter_mut().enumerate().for_each(|(idx, pty)| {
        debug!("Sending DISC frame to PTY {}", idx);
        if idx == 0 {
            let frame = Frame::new(
                addr.with_dlci(idx as u8),
                ctrl.with_frame(FrameType::UIH),
                2,
                vec![C_CLD | CR, 1],
            );
            pty.write_frame(frame.clone()).unwrap();
        }
    });
    Ok(())
}
