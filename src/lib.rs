extern crate core;

pub mod flow_handler {
    use std::cmp::PartialEq;

    use tokio::net::TcpStream;
    use tokio::sync::mpsc;

    /*
          Flow Handler System

          Any task can use the mpsc sender to make the following requests:
              RequestFlowByte - Reserve a flow byte for your use.
              Transmit - Send a composed transmission across the TCP stream
              ReturnFlowByte - Return the flow byte to the pool

          The flow handler does not authenticate that the flow bytes in the messages are the correct ones.
          It will need to be implemented at a higher level.

          The enum HandlerCmd includes the data that is required by each command type
        */

    pub mod writer_task {
        /*
           Writer Task

           This task manages the flow states available and when given a transmit command will pass
           on the reply oneshot to the reading task
        */
        use std::collections::HashMap;
        use std::error;
        use std::fmt;

        use chrono::{TimeDelta, Utc};
        use tokio::io::AsyncWriteExt;
        use tokio::net::tcp::OwnedWriteHalf;
        use tokio::sync::{mpsc, oneshot};

        use crate::flow_handler::{ConnectionType, reader_task};

        #[derive(Debug)]
        pub struct RequestFlowByteError;

        impl fmt::Display for RequestFlowByteError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Flow byte not available")
            }
        }

        impl error::Error for RequestFlowByteError {}

        pub type RequestFlowByteReply = Result<u8, RequestFlowByteError>;

        #[derive(Debug)]
        pub struct TransmitError;

        impl fmt::Display for TransmitError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Failed to transmit")
            }
        }

        impl error::Error for TransmitError {}

        pub type TransmitReply = Result<Vec<u8>, TransmitError>;

        type ReplySender<T> = oneshot::Sender<T>;

        pub enum Cmd {
            RequestFlowByte(ReplySender<RequestFlowByteReply>),
            Transmit(ReplySender<TransmitReply>, u8, Vec<u8>),
            ReturnFlowByte(u8),
        }

        pub async fn create_writer_task(
            connection_type: ConnectionType,
            mut writer_cmd_rx: mpsc::Receiver<Cmd>,
            reader_cmd_tx: mpsc::Sender<reader_task::Cmd>,
            mut write: OwnedWriteHalf,
        ) {
            tokio::spawn(async move {
                // Setup Flow Tracker
                let mut flow_tracker: HashMap<u8, bool> = HashMap::new();

                if connection_type == ConnectionType::Client {
                    // Assign client flows
                    for flow_byte in 0x01..0x7F {
                        flow_tracker.insert(flow_byte, false);
                    }
                } else {
                    // Assign server flows
                    for flow_byte in 0x80..0xFF {
                        flow_tracker.insert(flow_byte, false);
                    }
                }

                // RX Loop
                loop {
                    if let Some(recv) = writer_cmd_rx.recv().await {
                        match recv {
                            Cmd::RequestFlowByte(reply_sender) => {
                                // Find the next available flow and reply
                                if let Some((next_available_flow, _)) =
                                    flow_tracker.iter().find(|(_, state)| !(**state))
                                {
                                    let _ = reply_sender.send(Ok(*next_available_flow));
                                } else {
                                    let _ = reply_sender.send(Err(RequestFlowByteError));
                                }
                            }
                            Cmd::Transmit(reply_sender, flow, data) => {
                                let _ = write.write(&data).await;

                                // Send reply_sender to reading thread
                                let (timeout_time, _) = Utc::now()
                                    .time()
                                    .overflowing_add_signed(TimeDelta::minutes(1));
                                let _ = reader_cmd_tx
                                    .send(reader_task::Cmd {
                                        flow,
                                        reply_channel: Some(reply_sender),
                                        timeout_time,
                                    })
                                    .await;
                            }
                            Cmd::ReturnFlowByte(return_flow) => {
                                // Assume the byte given is a valid and in use flow.
                                flow_tracker.insert(return_flow, false);
                            }
                        }
                    }
                }
            });
        }
    }

    pub mod reader_task {
        /*
           Reader Task

           This task will constantly check for new transmissions. It will ONLY validate the flow byte
           in the header of the transmission. Reservations are made for flow 0x00 for the heartbeat cycle.
        */
        use byteorder::{BigEndian, ByteOrder};
        use chrono::{NaiveTime, Utc};
        use tokio::io;
        use tokio::io::AsyncReadExt;
        use tokio::net::tcp::OwnedReadHalf;
        use tokio::sync::{mpsc, oneshot};

        use crate::flow_handler::ConnectionType;
        use crate::flow_handler::writer_task::{TransmitError, TransmitReply};

        pub struct Cmd {
            pub flow: u8,
            pub reply_channel: Option<oneshot::Sender<TransmitReply>>,
            pub timeout_time: NaiveTime,
        }

        pub async fn create_reader_task(
            connection_type: ConnectionType,
            mut reader_cmd_rx: mpsc::Receiver<Cmd>,
            heartbeat_cmd_tx: mpsc::Sender<()>,
            mut read: OwnedReadHalf,
        ) {
            tokio::spawn(async move {
                let mut recv_backlog: Vec<Cmd> = Vec::new();

                loop {
                    let mut header_buf: [u8; 5] = [0; 5];
                    match read.try_read(&mut header_buf) {
                        Ok(bytes_received) => {
                            if bytes_received != 5 {
                                eprintln!(
                                    "Malformed Transmission Header: {}, {:02X?}. Dropping",
                                    bytes_received, header_buf
                                );
                                continue;
                            }

                            let transmission_flow = header_buf[2];

                            if transmission_flow == 0x00
                                && connection_type == ConnectionType::Server
                            {
                                // Heartbeat Cycle
                                let _ = heartbeat_cmd_tx.send(()).await;
                            } else {
                                let data_length = BigEndian::read_u16(&header_buf[3..5]) as usize;

                                let mut combined_buf = vec![];
                                combined_buf.extend_from_slice(&header_buf);

                                if data_length > 0 {
                                    let mut data_buf = vec![0; data_length];

                                    let _ = read.read_exact(&mut data_buf).await;

                                    combined_buf.append(&mut data_buf);
                                }

                                if let Some(cmd) = recv_backlog
                                    .iter_mut()
                                    .find(|cmd| cmd.flow == transmission_flow)
                                {
                                    if let Some(reply) = cmd.reply_channel.take() {
                                        let _ = reply.send(Ok(combined_buf));
                                    }
                                }
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // Remove old received mpsc messages
                            for cmd in recv_backlog.iter_mut() {
                                if Utc::now().time() > cmd.timeout_time {
                                    if let Some(sender) = cmd.reply_channel.take() {
                                        let _ = sender.send(Err(TransmitError));
                                    }
                                }
                            }

                            // Look for new received on mpsc
                            if let Ok(cmd) = reader_cmd_rx.try_recv() {
                                recv_backlog.push(cmd);
                            }
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            });
        }
    }

    pub mod heartbeat_task {
        /*
           Heartbeat Task
        */

        use std::time::Duration;

        use tokio::sync::{mpsc, oneshot};
        use tokio::time::sleep;

        use crate::flow_handler::{ConnectionType, writer_task};

        pub async fn create_heartbeat_task(
            connection_type: ConnectionType,
            mut heartbeat_cmd_rx: mpsc::Receiver<()>,
            writer_cmd_tx: mpsc::Sender<writer_task::Cmd>,
        ) {
            tokio::spawn(async move {
                if connection_type == ConnectionType::Client {
                    loop {
                        let (tx, rx) = oneshot::channel::<writer_task::TransmitReply>();
                        let _ = writer_cmd_tx
                            .send(writer_task::Cmd::Transmit(tx, 0x00, vec![0x00; 5]))
                            .await;
                        if let Ok(Err(err)) = rx.await {
                            eprintln!("{:?}", err);
                            break;
                        }
                        sleep(Duration::from_secs(10)).await;
                    }
                } else {
                    loop {
                        if heartbeat_cmd_rx.try_recv().is_ok() {
                            let (tx, rx) = oneshot::channel::<writer_task::TransmitReply>();
                            let _ = writer_cmd_tx
                                .send(writer_task::Cmd::Transmit(tx, 0x00, vec![0x00; 5]))
                                .await;
                            if let Ok(Err(err)) = rx.await {
                                eprintln!("{:?}", err);
                                break;
                            }
                        }
                    }
                }
            });
        }
    }

    #[derive(PartialEq, Copy, Clone)]
    pub enum ConnectionType {
        Server,
        Client,
    }

    pub struct FlowHandler {
        pub handler_cmd: mpsc::Sender<writer_task::Cmd>,
    }

    impl FlowHandler {
        pub async fn new(socket: TcpStream, connection_type: ConnectionType) -> FlowHandler {
            let (read, write) = socket.into_split();

            let (writer_cmd_tx, writer_cmd_rx) = mpsc::channel::<writer_task::Cmd>(32);
            let (reader_cmd_tx, reader_cmd_rx) = mpsc::channel::<reader_task::Cmd>(32);
            let (heartbeat_cmd_tx, heartbeat_cmd_rx) = mpsc::channel::<()>(32);

            writer_task::create_writer_task(connection_type, writer_cmd_rx, reader_cmd_tx, write)
                .await;
            reader_task::create_reader_task(connection_type, reader_cmd_rx, heartbeat_cmd_tx, read)
                .await;
            heartbeat_task::create_heartbeat_task(
                connection_type,
                heartbeat_cmd_rx,
                writer_cmd_tx.clone(),
            )
            .await;

            FlowHandler {
                handler_cmd: writer_cmd_tx,
            }
        }
    }
}

pub mod transmission {}
