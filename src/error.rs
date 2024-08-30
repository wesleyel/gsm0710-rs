use thiserror::Error;

#[derive(Error, Debug)]
pub enum GsmError {
    #[error("AT command failed: {0}")]
    AtCommandFailed(String),
    #[error("AT command timed out: {0}")]
    AtCommandTimedOut(String),
}
