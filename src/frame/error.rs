use std::fmt;
use std::error;

#[derive(Debug, Clone)]
pub enum FrameErrorType {
    ReadFailure,
    WriteFailure,
    LengthMismatch,
    SocketClosed, 
    InvalidType,
    LengthOverflow,
    ConversionError,
}

#[derive(Debug, Clone)]
pub struct FrameError {
    pub error_type: FrameErrorType,
    pub message: Option<String>,
}

impl FrameError {
    pub fn new(error_type: FrameErrorType, message: Option<String>) -> FrameError {
        FrameError {
            error_type,
            message
        }
    }
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = &self.message {
            write!(f, "{:?} | {}", self.error_type, msg)
        } else {
            write!(f, "{:?}", self.error_type)
        }
    }
}

impl error::Error for FrameError {}