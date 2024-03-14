/*
frame.rs
This module has helper functions that make using the RRMT protocol easy

To-Do:
- [X] Create frames
- [X] Write frames
- [X] Read frames
- [X] Convert bytes to data
 */

use std::error;
use std::fmt;

use byteorder::{BigEndian, ByteOrder};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use uuid::Uuid;

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
                0x03
            }
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
#[derive(Debug, Clone)]
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

#[derive(PartialEq)]
pub enum RRMTRole {
    Server,
    Client
}

pub enum RRMTFrameDirection {
    ServerToClient,
    ClientToServer,
    BiDirectional
}

#[derive(Debug, Clone)]
pub struct RRMTInvalidRoleError;

impl fmt::Display for RRMTInvalidRoleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Role")
    }
}

impl error::Error for RRMTInvalidRoleError {}

#[derive(Debug, Clone)]
pub enum ReadError {
    ReadFailure,
    LengthMismatch,
    SocketClosed,
    InvalidType
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for ReadError {}

#[derive(Debug, Clone)]
pub enum WriteError {
    WriteFailure,
    LengthOverflow
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for WriteError {}

#[derive(Debug, Clone)]
pub struct RRMTFrame {
    pub frame_type: RRMTFrameType,
    pub data: Option<Vec<u8>>
}


impl RRMTFrame {
    pub fn new_authorize(role: RRMTRole, uuid: Uuid) -> Result<RRMTFrame, RRMTInvalidRoleError> {
        if !Self::check_direction(role, RRMTFrameDirection::ClientToServer) {return Err(RRMTInvalidRoleError)}
        Ok(RRMTFrame {
            frame_type: RRMTFrameType::Authorize,
            data: Some(uuid.as_data())
        })
    }

    pub fn new_denied(role: RRMTRole) -> Result<RRMTFrame, RRMTInvalidRoleError> {
        if !Self::check_direction(role, RRMTFrameDirection::ServerToClient) {return Err(RRMTInvalidRoleError)}

        Ok(RRMTFrame {
            frame_type: RRMTFrameType::Denied,
            data: None
        })
    }

    pub fn new_accepted(role: RRMTRole) -> Result<RRMTFrame, RRMTInvalidRoleError> {
        if !Self::check_direction(role, RRMTFrameDirection::ServerToClient) {return Err(RRMTInvalidRoleError)}

        Ok(RRMTFrame {
            frame_type: RRMTFrameType::Denied,
            data: None
        })
    }

    pub fn new_ping(role: RRMTRole) -> Result<RRMTFrame, RRMTInvalidRoleError> {
        if !Self::check_direction(role, RRMTFrameDirection::ServerToClient) {return Err(RRMTInvalidRoleError)}

        Ok(RRMTFrame {
            frame_type: RRMTFrameType::Ping,
            data: None
        })
    }

    pub fn new_pong(role: RRMTRole) -> Result<RRMTFrame, RRMTInvalidRoleError> {
        if !Self::check_direction(role, RRMTFrameDirection::ClientToServer) {return Err(RRMTInvalidRoleError)}
        
        Ok(RRMTFrame {
            frame_type: RRMTFrameType::Pong,
            data: None
        })
    }
    
    pub fn new_error(role: RRMTRole, error_type: RRMTError) -> Result<RRMTFrame, RRMTInvalidRoleError> {
        if !Self::check_direction(role, RRMTFrameDirection::BiDirectional) {return Err(RRMTInvalidRoleError)}
        
        Ok(RRMTFrame {
            frame_type: RRMTFrameType::Error,
            data: Some(error_type.as_data())
        })
    }
    
    pub fn new_execute(role: RRMTRole, command: String) -> Result<RRMTFrame, RRMTInvalidRoleError> {
        if !Self::check_direction(role, RRMTFrameDirection::ServerToClient) {return Err(RRMTInvalidRoleError) }
        
        Ok(RRMTFrame {
            frame_type: RRMTFrameType::Execute,
            data: Some(command.as_data())
        })
    }
    
    pub fn new_result(role: RRMTRole, result: String) -> Result<RRMTFrame, RRMTInvalidRoleError> {
        if !Self::check_direction(role, RRMTFrameDirection::ClientToServer) {return Err(RRMTInvalidRoleError)}
        
        Ok(RRMTFrame {
            frame_type: RRMTFrameType::Result,
            data: Some(result.as_data())
        })
    }
    
    pub fn new_ack(role: RRMTRole) -> Result<RRMTFrame, RRMTInvalidRoleError> {
        if !Self::check_direction(role, RRMTFrameDirection::BiDirectional) {return Err(RRMTInvalidRoleError)}
        
        Ok(RRMTFrame {
            frame_type: RRMTFrameType::ACK,
            data: None
        })
    }

    fn check_direction(role: RRMTRole, expected: RRMTFrameDirection) -> bool {
        match expected {
            RRMTFrameDirection::ServerToClient => {
                role == RRMTRole::Server
            },
            RRMTFrameDirection::ClientToServer => {
                role == RRMTRole::Client
            },
            RRMTFrameDirection::BiDirectional => true
        }
    }
    
    pub async fn write(&self, write_half: &mut OwnedWriteHalf) -> Result<(), WriteError> {
        let mut frame: Vec<u8> = vec![];
        frame.push(self.frame_type.clone() as u8);
        
        if let Some(mut data) = self.data.clone() {
            if data.len() > u16::MAX as usize {return Err(WriteError::LengthOverflow)}
            
            let mut length = [0; 2]; 
            BigEndian::write_u16(&mut length, data.len() as u16);
            frame.extend_from_slice(&length);
            
            frame.append(&mut data);
        }
        
        if write_half.write_all(&frame).await.is_err() {
            return Err(WriteError::WriteFailure);
        }
        
        Ok(())
    }
    
    pub async fn read(read_half: &mut OwnedReadHalf) -> Result<RRMTFrame, ReadError> {
        let mut peek_buf = [0; 1];
        let check = match read_half.peek(&mut peek_buf).await {
            Ok(check) => check, 
            Err(_) => return Err(ReadError::ReadFailure)
        };
        if check == 0 {return Err(ReadError::SocketClosed)}
        
        let mut header_buf = [0; 3];
        if read_half.read_exact(&mut header_buf).await.is_err() {
            return Err(ReadError::ReadFailure)
        }
        
        let (rrmt_type_buf, rrmt_length_buf) = header_buf.split_at(1);
        
        let rrmt_length = BigEndian::read_u16(rrmt_length_buf) as usize;
        let mut data: Option<Vec<u8>> = None;
        
        if rrmt_length > 0 {
            let mut rrmt_data_buf = vec![0; rrmt_length];
            loop {
                let n = match read_half.read_buf(&mut rrmt_data_buf).await {
                    Ok(n) => n,
                    Err(_) => return Err(ReadError::ReadFailure)
                };
                if n == 0 {
                    break;
                }
            }
            if rrmt_length != rrmt_data_buf.len() { return Err(ReadError::LengthMismatch) }
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
            _ => return Err(ReadError::InvalidType)
        };
        
        Ok(RRMTFrame {
            frame_type: rrmt_type,
            data
        })
    }
}

#[derive(Debug, Clone)]
pub struct ConversionError;

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to convert")
    }
}

impl error::Error for ConversionError {}

pub fn data_to_uuid(data: Vec<u8>) -> Result<Uuid, ConversionError> {
    let bytes: [u8; 16] = match data.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return Err(ConversionError)
    };
    Ok(Uuid::from_bytes(bytes))
}

pub fn data_to_string(data: Vec<u8>) -> Result<String, ConversionError> {
    match String::from_utf8(data) {
        Ok(string) => Ok(string),
        Err(_) => Err(ConversionError)
    }
}

pub fn data_to_error(mut data: Vec<u8>) -> Result<RRMTError, ConversionError> {
    let mut message = String::new();
    let type_int = data[0];
    data.remove(0);
    
    if data.len() > 1 {
        message = match String::from_utf8(data) {
            Ok(msg) => msg,
            Err(_) => return Err(ConversionError)
        };
    }
    
    match type_int {
        0x01 => Ok(RRMTError::LengthMismatch),
        0x02 => Ok(RRMTError::ServerError(message)),
        0x03 => Ok(RRMTError::FormatError),
        0x04 => Ok(RRMTError::ExecuteError(message)),
        _ => Err(ConversionError)
    }
}