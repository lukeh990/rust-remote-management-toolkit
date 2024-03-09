# Rust Remote Management Toolkit (RRMT)
The goal of RRMT is to create a service that will provide a tool that 
administrators can use to preform remote work on a system even if 
that system crosses firewalls.

To accomplish this each remote host maintains a constant websocket 
connection to the management server. When an administrator wants to connect
they will use the cli to request a shell with the management server. 
The management server utilizes the websocket to instruct the remote host
to use a tool like netcat to open a shell and forward it to the 
management server, which will in turn forward that again to the Administrator

See diagram:  
![System Diagram](static/diagram.png?)

## Progress
- [ ] Design RRMT Protocol
- [ ] Implement RRMT Protocol
- [ ] Authentication
- [ ] Provisioning
- [ ] Maintaining Connection
- [ ] Remote Execution
- [ ] Get bash shell on cli client
- [ ] Implement TLS
- [ ] Real World Testing
- [ ] Minimum Viable Product
- [ ] RDP/VLC support
- [ ] Web UI

## RRMT Protocol
The RRMT Protocol is a simple binary protocol designed specifically to maintain 
an open connection with the Management Server and to instruct the remote clients 
on how to start the remote shell. 

### Packet Makeup

- Type (1 Byte)
- Length (2 Bytes)
- Data (0 - 65,536 Bytes)

### Payload Type
The message type is encoded in a single byte. The following message types are 
im use:
- `0x01` - ACK
- `0x02` - Authorize
- `0x03` - Revoke
- `0x04` - Provision
- `0x05` - Ping
- `0x06` - Pong
- `0x07` - Execute
- `0x08` - Result
- `0x09` - Reauthorize
- `0x0A` - Error

### Authorization Stage
This is the first step any host must take is requesting authorization using 
a token. The token will be in the form of a RFC4122 v4 UUID that will be
known by the server beforehand. The payload will look something like this:
```
02 | 00 10 | A1 B6 85 8E 25 CC 43 54 A0 FF 06 FC A3 0B 11 09
```
- `0x02` - Type (Authorize)
- `0x0010` - Length (16)
- `0xA1...09` - Payload (UUID V4 in binary)

If the server validates the token, response will contain a type of provision (`0x04`) 
with information. If the server is unable to validate the token the response 
will be of type Error (`0x0A`)