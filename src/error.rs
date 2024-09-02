use thiserror::Error;

#[derive(Error, Debug)]
pub enum GsmError {
    #[error("AT command failed: {0}")]
    AtCommandFailed(String),
    #[error("AT command timed out: {0}")]
    AtCommandTimedOut(String),

    #[error("Parse frame error: {0}")]
    ParseFrameError(String),

    #[error("Unsupported frame type: {0}")]
    UnsupportedFrameType(String),
    #[error("Unsupported modem type: {0}")]
    UnsupportedModemType(String),
}
