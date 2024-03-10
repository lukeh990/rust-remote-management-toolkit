pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub mod frame {
    use byteorder::{BigEndian, ByteOrder};
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpStream;
    use uuid::Uuid;

    use crate::Result;

    // Refer to README.md#Frame Types
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

    pub async fn read_frame(mut socket: TcpStream) -> Result<RRMTFrame> {
        let mut header_buf = [0; 3];
        socket.read_exact(&mut header_buf).await?;

        let (rrmt_type_buf, rrmt_length_buf) = header_buf.split_at(1);

        let rrmt_length = BigEndian::read_u16(rrmt_length_buf) as usize;

        let mut rrmt_data_buf = vec![0; rrmt_length];
        socket.read_exact(&mut rrmt_data_buf).await?;

        println!("{:X?}", rrmt_data_buf);

        match rrmt_type_buf[0] {
            0x01 => Ok(RRMTFrame::ACK),
            0x02 => Ok(RRMTFrame::Authorize(uuid_from_buf(rrmt_data_buf)?)),
            0x03 => Ok(RRMTFrame::Revoke),
            0x04 => Ok(RRMTFrame::Provision(uuid_from_buf(rrmt_data_buf)?)),
            0x05 => Ok(RRMTFrame::Ping),
            0x06 => Ok(RRMTFrame::Pong),
            0x07 => Ok(RRMTFrame::Execute(String::from_utf8(rrmt_data_buf)?)),
            0x08 => Ok(RRMTFrame::Result(String::from_utf8(rrmt_data_buf)?)),
            0x09 => Ok(RRMTFrame::Reauthorize(uuid_from_buf(rrmt_data_buf)?)),
            0x0A => Ok(RRMTFrame::Denied(String::from_utf8(rrmt_data_buf)?)),
            0x0B => Ok(RRMTFrame::Error(String::from_utf8(rrmt_data_buf)?)),
            _ => Err("Invalid frame type".into())
        }
    }

    fn uuid_from_buf(rrmt_data_buf: Vec<u8>) -> Result<Uuid> {
        let bytes: [u8; 16] = match rrmt_data_buf.try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err("Invalid length".into())
        };
        Ok(Uuid::from_bytes(bytes))
    }
}