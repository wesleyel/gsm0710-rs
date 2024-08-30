use clap::{ArgAction, Parser, ValueEnum};
use serde::Serialize;

#[derive(ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModemType {
    /// Generic modem
    #[default]
    Generic,
    /// Sam201 modem
    Sam201,
}

/// A gsm0710 protocol MUX implementation
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Pty device to open
    #[arg(short, long, default_value = "/dev/ptmx")]
    pub pty: String,

    /// Number of channels to create.
    #[arg(short, long, default_value = "7")]
    pub channels: u8,

    /// Baud rate to use
    #[arg(short, long, default_value = "115200")]
    pub baud: u32,

    /// Maximum frame size
    #[arg(short, long, default_value = "32")]
    pub frame_size: u32,

    /// Modem type
    #[arg(short, long, default_value = "generic")]
    pub modem: ModemType,

    /// Prefix for the symlinks (e.g. /dev/mux)
    #[arg(short, long)]
    pub symlink_prefix: Option<String>,

    /// Disable daemon mode
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub no_daemon: bool,

    /// Auto restart on modem not responding
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub auto_restart: bool,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// Serial port to use
    pub port: String,
}
