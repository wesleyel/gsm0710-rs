use std::{
    io::{Read, Write},
    os::fd::AsRawFd,
};

use crate::{error::GsmError, types::Frame};
use anyhow::{bail, Result};
use log::debug;
use mio::{event::Source, unix::SourceFd, Events, Interest, Poll, Token};
use mio_serial::SerialStream;
use nix::{
    fcntl::OFlag,
    pty::PtyMaster,
    sys::{
        stat::Mode,
        termios::{tcgetattr, tcsetattr, InputFlags, LocalFlags, OutputFlags, SetArg},
    },
};

/// PtyStream
#[derive(Debug)]
pub struct PtyStream {
    pub inner: PtyMaster,
}

impl PtyStream {
    pub fn new(inner: PtyMaster) -> Self {
        Self { inner }
    }
}

impl Source for PtyStream {
    fn register(
        &mut self,
        registry: &mio::Registry,
        token: Token,
        interests: Interest,
    ) -> std::io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &mio::Registry,
        token: Token,
        interests: Interest,
    ) -> std::io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &mio::Registry) -> std::io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).deregister(registry)
    }
}

pub trait PtyWriteFrame {
    fn write_frame(&mut self, frame: Frame) -> Result<()>;
}

impl PtyWriteFrame for PtyStream {
    fn write_frame(&mut self, frame: Frame) -> Result<()> {
        let buf = frame.try_to_bytes()?;
        self.inner.write_all(&buf)?;
        Ok(())
    }
}

pub const SERIAL_TOKEN: Token = Token(0);
/// Send an AT command to the modem and wait for a response.
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

pub fn openpty(
    ptmx: String,
    channel_index: u8,
    symlink_prefix: Option<String>,
) -> Result<PtyMaster> {
    let fd = nix::pty::posix_openpt(OFlag::O_RDWR | OFlag::O_NONBLOCK)?;
    if let Some(prefix) = symlink_prefix {
        let symlink = format!("{}{}", prefix, channel_index);
        let sym_path = unsafe { nix::pty::ptsname(&fd)? };

        // Remove the symlink if it already exists
        if let Err(err) = nix::unistd::unlink(sym_path.as_str()) {
            debug!("Failed to remove symlink: {}", err);
        }
        // Create a new symlink from the slave pty to the symlink path
        debug!("Creating symlink: {} -> {}", sym_path, symlink);
        nix::unistd::symlinkat(sym_path.as_str(), None, symlink.as_str())?;

        // grant access to the slave pty
        if ptmx.contains("/dev/ptmx") {
            nix::pty::grantpt(&fd)?;
            nix::pty::unlockpt(&fd)?;
            nix::sys::stat::fchmodat(
                None,
                symlink.as_str(),
                Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP,
                nix::sys::stat::FchmodatFlags::FollowSymlink,
            )?;
        }
    }

    // Set the slave pty terminal settings
    let mut termios = tcgetattr(&fd)?;
    termios.input_flags =
        termios.input_flags & !(InputFlags::INLCR | InputFlags::ICRNL | InputFlags::IGNCR);
    termios.local_flags = termios.local_flags
        & !(LocalFlags::ICANON | LocalFlags::ECHO | LocalFlags::ECHOE | LocalFlags::ISIG);
    termios.output_flags = termios.output_flags
        & !(OutputFlags::OPOST
            | OutputFlags::OLCUC
            | OutputFlags::ONLRET
            | OutputFlags::ONOCR
            | OutputFlags::OCRNL);
    tcsetattr(&fd, SetArg::TCSANOW, &termios)?;
    Ok(fd)
}
