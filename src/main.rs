use std::{
    collections::HashMap,
    io::{Read, Write},
    time::Duration,
};

use anyhow::Result;
use buffer::{GSM0710Buffer, GSM0710_BUFFER_CAPACITY};
use clap::Parser;
use cli::{Args, ModemType};
use error::GsmError;
use log::{debug, error, info};
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
    at_command(ss, HOLA_CMD, 100)?;
    at_command(ss, MUX_CMD, 100)?;
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

    let mut buffer = AllocRingBuffer::<u8>::new(GSM0710_BUFFER_CAPACITY);
    info!("Initialized buffer with capacity {}", buffer.capacity());

    let mut ptys = HashMap::<u8, PtyStream>::new();
    for idx in 0..args.channels {
        let pty = openpty(args.clone().pty, idx, args.clone().symlink_prefix)?;
        ptys.insert(idx, PtyStream { inner: pty });
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
    let ctrl = 0u8.with_pf(true).with_frame_type(FrameType::SABM);
    let mut frame = Frame::new(addr, ctrl, 0, vec![0]);
    ptys.iter_mut().for_each(|(idx, pty)| {
        debug!("Sending SABM frame to PTY {}", idx);
        if *idx == 0 {
            frame.address.set_dlci(*idx);
            pty.write_frame(frame.clone()).unwrap();
        } else {
            frame.address.set_dlci(*idx);
            pty.write_frame(frame.clone()).unwrap();
        }
    });
    info!("Sent SABM frames to all PTYs");

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(ptys.len() + 1);

    // Register the serial port and all PTYs with the poller
    poll.registry()
        .register(&mut ss, Token(0), mio::Interest::READABLE)?;
    for (idx, pty) in ptys.iter_mut() {
        poll.registry()
            .register(pty, Token((idx + 1).into()), mio::Interest::READABLE)?;
    }

    'outer: loop {
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
                        let frame = match buffer.pop_frame1() {
                            Some(frame) => frame,
                            None => break,
                        };
                        match frame.address.get_frame_type() {
                            Err(e) => {
                                error!("Error parsing frame type: {}", e);
                                continue;
                            }
                            Ok(ft) => match ft {
                                FrameType::UIH | FrameType::UI => {
                                    let pty = ptys.get_mut(&frame.address.get_dlci()).unwrap();
                                    pty.inner.write(&frame.content)?;
                                }
                                _ => {}
                            },
                        }
                    }
                }
                Token(idx) => {
                    let idx_real = (idx - 1) as u8;
                    let pty = ptys.get_mut(&idx_real).unwrap();
                    let mut buf = vec![0u8; 1024];
                    let n = match pty.inner.read(&mut buf) {
                        Ok(n) => n,
                        Err(e) => {
                            error!("Error reading from PTY {}: {}", idx_real, e);
                            break;
                        }
                    };
                    debug!(
                        "Received {} bytes from PTY {}: {:02X?}",
                        n,
                        idx_real,
                        &buf[..n]
                    );

                    let frame = Frame::new(
                        addr.with_dlci(idx_real),
                        ctrl.with_frame_type(FrameType::UIH),
                        n as u16,
                        buf[..n].to_vec(),
                    );
                    let data = frame.try_to_bytes()?;
                    match ss.write(&data) {
                        Ok(_) => debug!("Sent {} bytes to serial port: {:02X?}", data.len(), &data),
                        Err(e) => {
                            error!("Error sending data to serial port: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    }

    info!("Closing logical channels");
    ptys.iter_mut().for_each(|(idx, pty)| {
        debug!("Sending DISC frame to PTY {}", idx);
        if *idx != 0 {
            let frame = Frame::new(
                addr.with_dlci(*idx),
                ctrl.with_frame_type(FrameType::DISC),
                0,
                vec![0],
            );
            pty.write_frame(frame.clone()).unwrap();
        }
    });
    info!("Closing control channel");
    let frame = Frame::new(
        addr.with_dlci(0),
        ctrl.with_frame_type(FrameType::UIH),
        2,
        vec![C_CLD | CR, 1],
    );
    ptys.get_mut(&0).unwrap().write_frame(frame)?;

    Ok(())
}
