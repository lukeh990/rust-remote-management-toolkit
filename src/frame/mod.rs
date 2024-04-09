/*
frame.rs
This module has helper functions that make using the RRMT protocol easy

To-Do:
- [X] Create frames
- [X] Write frames
- [X] Read frames
- [X] Convert bytes to data
 */


use std::fmt;
use byteorder::{BigEndian, ByteOrder};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use uuid::Uuid;
use crate::frame::error::{FrameError, FrameErrorType};

pub mod error;

pub trait AsData {
    fn as_data(&self) -> Vec<u8>;
}

// Refer to README.md#Error Types
#[derive(Debug, Clone)]
pub enum RRMTError {
    //0x01
    LengthMismatch,
    //0x02
    ServerError(String),
    //0x03,
    FormatError,
    //0x04
    ExecuteError(String),
    //0x05
    NotExpected,
}

impl AsData for RRMTError {
    fn as_data(&self) -> Vec<u8> {
        let mut message: Option<String> = None;

        let byte = match self {
            RRMTError::LengthMismatch => 0x01,
            RRMTError::ServerError(msg) => {
                message = Some(msg.clone());
                0x02
            },
            RRMTError::FormatError => 0x03,
            RRMTError::ExecuteError(msg) => {
                message = Some(msg.clone());
                0x04
            },
            RRMTError::NotExpected => 0x05,
        };

        let mut vec = vec![byte];

        if let Some(msg) = message {
            vec.extend_from_slice(msg.as_bytes());
        }

        vec
    }
}

impl AsData for Uuid {
    fn as_data(&self) -> Vec<u8> {
        Vec::from(self.as_bytes())
    }
}

impl AsData for String {
    fn as_data(&self) -> Vec<u8> {
        Vec::from(self.as_bytes())
    }
}

// Refer to README.md#Frame Types
#[derive(Debug, Clone, PartialEq)]
pub enum RRMTFrameType {
    // 0x01
    Authorize,
    // 0x02
    Denied,
    // 0x03
    Accepted,
    // 0x04
    Ping,
    // 0x05
    Pong,
    // 0x06
    Error,
    // 0x07
    Execute,
    // 0x08
    Result,
    // 0x09
    ACK,
}

#[derive(Debug, Clone)]
pub struct RRMTInvalidRoleError;

impl fmt::Display for RRMTInvalidRoleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Role")
    }
}

impl std::error::Error for RRMTInvalidRoleError {}

#[derive(Debug, Clone)]
pub struct RRMTFrame {
    pub frame_type: RRMTFrameType,
    pub data: Option<Vec<u8>>,
    pub conversation: u8
}


impl RRMTFrame {
    pub fn new(frame_type: RRMTFrameType, data: Option<Vec<u8>>, conversation: u8) -> RRMTFrame {
        RRMTFrame {
            frame_type,
            data,
            conversation
        }
    }
    
    pub async fn write(&self, write_half: &mut OwnedWriteHalf) -> Result<(), FrameError> {
        let mut frame: Vec<u8> = vec![];
        frame.push(self.frame_type.clone() as u8);
        
        if let Some(data) = &self.data {
            if data.len() > u16::MAX as usize {return Err(FrameError::new(FrameErrorType::LengthOverflow, None))}
            
            let mut length = [0; 2]; 
            BigEndian::write_u16(&mut length, data.len() as u16);
            frame.extend_from_slice(&length);
        } else {
            frame.extend_from_slice(&[0x00, 0x00]);
        }
        
        frame.push(self.conversation);
        
        if let Some(data) = &self.data {
            frame.extend(data);
        }
        
        if write_half.write_all(&frame).await.is_err() {
            return Err(FrameError::new(FrameErrorType::WriteFailure, None));
        }
        
        Ok(())
    }
    
    pub async fn read(read_half: &mut OwnedReadHalf) -> Result<RRMTFrame, FrameError> {
        let mut peek_buf = [0; 1];
        let check = match read_half.peek(&mut peek_buf).await {
            Ok(check) => check, 
            Err(_) => return Err(FrameError::new(FrameErrorType::ReadFailure, None))
        };
        if check == 0 {return Err(FrameError::new(FrameErrorType::SocketClosed, None))}
        
        let mut header_buf = [0; 4];
        if read_half.read_exact(&mut header_buf).await.is_err() {
            return Err(FrameError::new(FrameErrorType::ReadFailure, None))
        }
        
        let (rrmt_type_buf, rrmt_header_remain) = header_buf.split_at(1);
        let (rrmt_length_buf, rrmt_conversation_buf) = rrmt_header_remain.split_at(2);
        
        let rrmt_length = BigEndian::read_u16(rrmt_length_buf) as usize;
        let mut data: Option<Vec<u8>> = None;
        
        if rrmt_length > 0 {
            let mut rrmt_data_buf = vec![0; rrmt_length];
            loop {
                let n = match read_half.read_buf(&mut rrmt_data_buf).await {
                    Ok(n) => n,
                    Err(_) => return Err(FrameError::new(FrameErrorType::ReadFailure, None))
                };
                if n == 0 {
                    break;
                }
            }
            if rrmt_length != rrmt_data_buf.len() { return Err(FrameError::new(FrameErrorType::LengthMismatch, None)) }
            data = Some(rrmt_data_buf);
        }
        
        let rrmt_type = match rrmt_type_buf[0] {
            0x01 => RRMTFrameType::Authorize,
            0x02 => RRMTFrameType::Denied,
            0x03 => RRMTFrameType::Accepted,
            0x04 => RRMTFrameType::Ping,
            0x05 => RRMTFrameType::Pong,
            0x06 => RRMTFrameType::Error,
            0x07 => RRMTFrameType::Execute,
            0x08 => RRMTFrameType::Result,
            0x09 => RRMTFrameType::ACK,
            _ => return Err(FrameError::new(FrameErrorType::InvalidType, None))
        };
        
        Ok(RRMTFrame {
            frame_type: rrmt_type,
            data,
            conversation: rrmt_conversation_buf[0]
        })
    }
}

pub fn data_to_uuid(data: Vec<u8>) -> Result<Uuid, FrameError> {
    let bytes: [u8; 16] = match data.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return Err(FrameError::new(FrameErrorType::ConversionError, None))
    };
    Ok(Uuid::from_bytes(bytes))
}

pub fn data_to_string(data: Vec<u8>) -> Result<String, FrameError> {
    match String::from_utf8(data) {
        Ok(string) => Ok(string),
        Err(_) => Err(FrameError::new(FrameErrorType::ConversionError, None))
    }
}

pub fn data_to_error(mut data: Vec<u8>) -> Result<RRMTError, FrameError> {
    let mut message = String::new();
    let type_int = data[0];
    data.remove(0);
    
    if data.len() > 1 {
        message = match String::from_utf8(data) {
            Ok(msg) => msg,
            Err(_) => return Err(FrameError::new(FrameErrorType::ConversionError, None))
        };
    }
    
    match type_int {
        0x01 => Ok(RRMTError::LengthMismatch),
        0x02 => Ok(RRMTError::ServerError(message)),
        0x03 => Ok(RRMTError::FormatError),
        0x04 => Ok(RRMTError::ExecuteError(message)),
        0x05 => Ok(RRMTError::NotExpected),
        _ => Err(FrameError::new(FrameErrorType::ConversionError, None))
    }
}