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

use crate::Result;

// Refer to README.md#Frame Types
#[derive(Debug)]
pub enum RRMTFrame {
    // 0x01
    ACK,
    // 0x02
    Authorize(Uuid),
    // 0x03
    Revoke,
    // 0x04
    Provision(Uuid),
    // 0x05
    Ping,
    // 0x06
    Pong,
    // 0x07
    Execute(String),
    // 0x08
    Result(String),
    // 0x09
    Reauthorize(Uuid),
    // 0x0A
    Denied(String),
    // 0x0B
    Error(String),
}

#[derive(Debug, Clone)]
pub enum ReadErrorType {
    ConnectionError,
    ClientError,
}

#[derive(Debug, Clone)]
pub struct ReadError {
    pub error_type: ReadErrorType,
    pub msg: &'static str,
}

impl ReadError {
    fn new(error_type: ReadErrorType, msg: &'static str) -> ReadError {
        ReadError { error_type, msg }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} | {}", self.error_type, self.msg)
    }
}

impl error::Error for ReadError {}

pub async fn read_frame(stream: &mut TcpStream) -> std::result::Result<RRMTFrame, ReadError> {
    let mut check_buf = [0; 1];
    let check = match stream.peek(&mut check_buf).await {
        Ok(check) => check,
        Err(_) => {
            return Err(ReadError::new(
                crate::frame::ReadErrorType::ConnectionError,
                "Failed to check connection",
            ));
        }
    };
    if check == 0 {
        return Err(ReadError::new(crate::frame::ReadErrorType::ConnectionError, "Connection Closed"));
    }

    let mut header_buf = [0; 3];
    if stream.read_exact(&mut header_buf).await.is_err() {
        return Err(ReadError::new(
            crate::frame::ReadErrorType::ConnectionError,
            "Failed to read header bytes",
        ));
    }

    let (rrmt_type_buf, rrmt_length_buf) = header_buf.split_at(1);

    let rrmt_length = BigEndian::read_u16(rrmt_length_buf) as usize;

    let mut rrmt_data_buf = vec![0; rrmt_length];
    if stream.read_exact(&mut rrmt_data_buf).await.is_err() {
        return Err(ReadError::new(crate::frame::ReadErrorType::ConnectionError, "Failed to read payload"));
    }

    match rrmt_type_buf[0] {
        0x01 => Ok(RRMTFrame::ACK),
        0x02 => Ok(RRMTFrame::Authorize(uuid_from_buf(rrmt_data_buf).await?)),
        0x03 => Ok(RRMTFrame::Revoke),
        0x04 => Ok(RRMTFrame::Provision(uuid_from_buf(rrmt_data_buf).await?)),
        0x05 => Ok(RRMTFrame::Ping),
        0x06 => Ok(RRMTFrame::Pong),
        0x07 => Ok(RRMTFrame::Execute(string_from_buf(rrmt_data_buf).await?)),
        0x08 => Ok(RRMTFrame::Result(string_from_buf(rrmt_data_buf).await?)),
        0x09 => Ok(RRMTFrame::Reauthorize(uuid_from_buf(rrmt_data_buf).await?)),
        0x0A => Ok(RRMTFrame::Denied(string_from_buf(rrmt_data_buf).await?)),
        0x0B => Ok(RRMTFrame::Error(string_from_buf(rrmt_data_buf).await?)),
        _ => Err(ReadError::new(crate::frame::ReadErrorType::ClientError, "Invalid Connection Type")),
    }
}

pub async fn write_frame(stream: &mut TcpStream, frame: RRMTFrame) -> Result<()> {
    let header: u8;
    let mut data: Vec<u8> = vec![];

    match frame {
        RRMTFrame::ACK => header = 0x01,

        RRMTFrame::Authorize(uuid) => {
            header = 0x02;
            data.extend_from_slice(uuid.as_bytes());
        }

        RRMTFrame::Revoke => header = 0x03,

        RRMTFrame::Provision(uuid) => {
            header = 0x04;
            data.extend_from_slice(uuid.as_bytes());
        }

        RRMTFrame::Ping => header = 0x05,

        RRMTFrame::Pong => header = 0x06,

        RRMTFrame::Execute(string) => {
            header = 0x07;
            data.extend_from_slice(string.as_bytes());
        }

        RRMTFrame::Result(string) => {
            header = 0x08;
            data.extend_from_slice(string.as_bytes());
        }

        RRMTFrame::Reauthorize(uuid) => {
            header = 0x09;
            data.extend_from_slice(uuid.as_bytes());
        }

        RRMTFrame::Denied(string) => {
            header = 0x0A;
            data.extend_from_slice(string.as_bytes());
        }

        RRMTFrame::Error(string) => {
            header = 0x0B;
            data.extend_from_slice(string.as_bytes())
        }
    }

    let length = data.len();

    if length > u16::MAX as usize {
        return Err(format!("Frame exceeds size limit. Tried to send {} bytes", length).into());
    }

    let length = length as u16;

    let mut frame: Vec<u8> = vec![];

    frame.push(header);

    let mut data_len = [0; 2];
    BigEndian::write_u16(&mut data_len, length);
    frame.extend_from_slice(&data_len);

    frame.append(&mut data);

    stream.write_all(&frame).await?;

    Ok(())
}

async fn uuid_from_buf(rrmt_data_buf: Vec<u8>) -> std::result::Result<Uuid, ReadError> {
    let bytes: [u8; 16] = match rrmt_data_buf.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return Err(ReadError::new(crate::frame::ReadErrorType::ClientError, "Invalid UUID Length")),
    };
    Ok(Uuid::from_bytes(bytes))
}

async fn string_from_buf(rrmt_data_buf: Vec<u8>) -> std::result::Result<String, ReadError> {
    match String::from_utf8(rrmt_data_buf) {
        Ok(string) => Ok(string),
        Err(_) => Err(ReadError::new(crate::frame::ReadErrorType::ClientError, "Failed to convert to string")),
    }
}
