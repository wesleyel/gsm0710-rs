use std::fmt::Display;

use clap::{ArgAction, Parser, ValueEnum};
use serde::Serialize;

#[derive(ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModemType {
    /// Init modem genericly
    #[default]
    Generic,
    /// Init Sam201 modem
    Sam201,
}

impl Display for ModemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModemType::Generic => write!(f, "generic"),
            ModemType::Sam201 => write!(f, "sam201"),
        }
    }
}

/// A gsm0710 protocol MUX implementation
#[derive(Parser, Debug, Clone)]
#[command(version, about, author, long_about = None, arg_required_else_help = true)]
pub struct Args {
    /// Pty device to open
    #[arg(short, long, default_value = "/dev/ptmx")]
    pub pty: String,

    /// Number of channels to create (0-63)
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

    /// Create symlinks for pts. (e.g. /dev/mux)
    #[arg(short, long)]
    pub symlink_prefix: Option<String>,

    /// Disable daemon mode
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub no_daemon: bool,

    /// Auto restart on modem not responding
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub auto_restart: bool,

    /// Verbose mode. (e.g. -v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// Serial port to use
    pub port: String,
}
