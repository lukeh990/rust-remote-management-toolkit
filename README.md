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

### Frame Fields

#### Header

- Type (1 Byte)
- Length (2 Bytes) (Big Endian)

#### Data

- Data (0 - 65,536 Bytes)

### Frame Types

The message type is encoded in a single byte. The following message types are
im use:

- `0x1` - ACK (Server <-> Client)
- `0x2` - Authorize (Server <-- Client)
- `0x3` - Revoke (Server --> Client)
- `0x4` - Provision (Server --> Client)
- `0x5` - Ping (Server --> Client)
- `0x6` - Pong (Server <-- Client)
- `0x7` - Execute (Server --> Client)
- `0x8` - Result (Server <-- Client)
- `0x9` - Reauthorize (Server <-- Client)
- `0xA` - Denied (Server --> Client)
- `0xB` - Error (Server <-> Client)

### Authorization Stage

This is the first step any host must take is requesting authorization using
a token. The token will be in the form of a RFC4122 v4 UUID that will be
known by the server beforehand. The payload will look something like this:

```
02 | 00 10 | A1 B6 85 8E 25 CC 43 54 A0 FF 06 FC A3 0B 11 09
```

- `0x02` - Type (Authorize)
- `0x0010` - Length (16)
- `0xA1...09` - Data (UUID V4 in binary)

If the server validates the token, response will contain a type of provision (`0x04`)
with information. If the server is unable to validate the token the response
will be of type Denied (`0xA`)