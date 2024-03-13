/*
frame.rs
This module has helper functions that make using the RRMT protocol easy

To-Do:
- [X] read & write frame functions
 */

use std::error;
use std::fmt;

use byteorder::{BigEndian, ByteOrder};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;

// Refer to README.md#Error Types
#[derive(Debug)]
pub enum ErrorType {
    //0x01
    LengthMismatch,
    //0x02
    ServerError,
    //0x03,
    FormatError,
    //0x04
    ExecuteError,
}

impl ErrorType {
    pub fn as_byte(&self) -> u8 {
        match self {
            ErrorType::LengthMismatch => 0x01,
            ErrorType::ServerError => 0x02,
            ErrorType::FormatError => 0x03,
            ErrorType::ExecuteError => 0x04,
        }
    }
}

// Refer to README.md#Frame Types
#[derive(Debug)]
pub enum RRMTFrame {
    // 0x01
    Authorize(Uuid),
    // 0x02
    Denied,
    // 0x03
    Accepted,
    // 0x04
    Ping,
    // 0x05
    Pong,
    // 0x06
    Error(ErrorType, String),
    // 0x07
    Execute(String),
    // 0x08
    Result(String),
    // 0x09
    ACK,
}

#[derive(Debug, Clone)]
pub enum ReadError {
    ConnectionError(&'static str),
    ClientError(&'static str),
    SocketClosed,
    LengthMismatch,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for ReadError {}

pub async fn read_frame(stream: &mut TcpStream) -> Result<RRMTFrame, ReadError> {
    let mut check_buf = [0; 1];
    let check = match stream.peek(&mut check_buf).await {
        Ok(check) => check,
        Err(_) => {
            return Err(ReadError::ConnectionError("Peek Check"));
        }
    };
    if check == 0 {
        return Err(ReadError::SocketClosed);
    }

    let mut header_buf = [0; 3];
    if stream.read_exact(&mut header_buf).await.is_err() {
        return Err(ReadError::ConnectionError("Read Buf"));
    }

    let (rrmt_type_buf, rrmt_length_buf) = header_buf.split_at(1);

    let rrmt_length = BigEndian::read_u16(rrmt_length_buf) as usize;

    let mut rrmt_data_buf = vec![0; rrmt_length];
    if stream.read_exact(&mut rrmt_data_buf).await.is_err() {
        return Err(ReadError::LengthMismatch);
    }

    match rrmt_type_buf[0] {
        0x01 => Ok(RRMTFrame::Authorize(uuid_from_buf(rrmt_data_buf).await?)),
        0x02 => Ok(RRMTFrame::Denied),
        0x03 => Ok(RRMTFrame::Accepted),
        0x04 => Ok(RRMTFrame::Ping),
        0x05 => Ok(RRMTFrame::Pong),
        0x06 => Ok(RRMTFrame::Error(
            error_type_from_buf(&mut rrmt_data_buf).await?,
            string_from_buf(rrmt_data_buf).await?,
        )),
        0x07 => Ok(RRMTFrame::Execute(string_from_buf(rrmt_data_buf).await?)),
        0x08 => Ok(RRMTFrame::Result(string_from_buf(rrmt_data_buf).await?)),
        0x09 => Ok(RRMTFrame::ACK),
        _ => Err(ReadError::ClientError("Header Not Supported")),
    }
}

#[derive(Debug, Clone)]
pub enum WriteError {
    FrameTooBig,
    ConnectionFailure,
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for WriteError {}

pub async fn write_frame(stream: &mut TcpStream, frame: RRMTFrame) -> Result<(), WriteError> {
    let header: u8;
    let mut data: Vec<u8> = vec![];

    match frame {
        RRMTFrame::Authorize(uuid) => {
            header = 0x01;
            data.extend_from_slice(uuid.as_bytes());
        }

        RRMTFrame::Denied => header = 0x02,

        RRMTFrame::Accepted => header = 0x03,

        RRMTFrame::Ping => header = 0x04,

        RRMTFrame::Pong => header = 0x05,

        RRMTFrame::Error(error_type, message) => {
            header = 0x06;
            data.push(error_type.as_byte());
            data.extend_from_slice(message.as_bytes());
        }

        RRMTFrame::Execute(string) => {
            header = 0x07;
            data.extend_from_slice(string.as_bytes());
        }

        RRMTFrame::Result(string) => {
            header = 0x08;
            data.extend_from_slice(string.as_bytes());
        }

        RRMTFrame::ACK => header = 0x09,
    }

    let length = data.len();

    if length > u16::MAX as usize {
        return Err(WriteError::FrameTooBig);
    }

    let length = length as u16;

    let mut frame: Vec<u8> = vec![];

    frame.push(header);

    let mut data_len = [0; 2];
    BigEndian::write_u16(&mut data_len, length);
    frame.extend_from_slice(&data_len);

    frame.append(&mut data);

    if stream.write_all(&frame).await.is_err() {
        return Err(WriteError::ConnectionFailure);
    }

    Ok(())
}

async fn uuid_from_buf(rrmt_data_buf: Vec<u8>) -> Result<Uuid, ReadError> {
    let bytes: [u8; 16] = match rrmt_data_buf.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return Err(ReadError::ClientError("Bad UUID format")),
    };
    Ok(Uuid::from_bytes(bytes))
}

async fn string_from_buf(rrmt_data_buf: Vec<u8>) -> Result<String, ReadError> {
    match String::from_utf8(rrmt_data_buf) {
        Ok(string) => Ok(string),
        Err(_) => Err(ReadError::ClientError("Bad String")),
    }
}

async fn error_type_from_buf(rrmt_data_buf: &mut Vec<u8>) -> Result<ErrorType, ReadError> {
    let error_type = match rrmt_data_buf[0] {
        0x01 => ErrorType::LengthMismatch,
        0x02 => ErrorType::ServerError,
        0x03 => ErrorType::FormatError,
        0x04 => ErrorType::ExecuteError,
        _ => return Err(ReadError::ClientError("Invalid error type")),
    };
    rrmt_data_buf.remove(0);
    Ok(error_type)
}
