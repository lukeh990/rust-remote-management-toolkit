# Rust Remote Management Toolkit (RRMT)

The goal of RRMT is to create a service that will provide a tool that
administrators can use to preform remote work on a system even if
that system crosses firewalls.

To accomplish this each remote host maintains a constantly maintained
connection to the management server. When an administrator wants to connect
they will use the cli to request a session with the management server.
The management server utilizes the maintained connection with the remote host to instruct the remote host
to use a tool like netcat to open a shell and forward it to the
management server, which will in turn forward that again to the Administrator

See diagram:  
![System Diagram](static/diagram.png?)

## Project Status

**2024-06-24**

Got a basic ping/pong cycle working with a framework for multithreaded communication.

## Tasks

- [/] Design RRMT Protocol
- [/] Implement RRMT Protocol
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

### Basics

#### Header

- Version (1 Byte)
- Type (1 Byte)
- Flow (1 Byte)
- Data Length (2 Bytes) (Big Endian)

##### Types

- 0x00 - Heartbeat

#### Data

Data can range from 0 bytes to 65,535.

As soon as the 5 bytes that make up the header have been sent the data follows. The length of the data must be exactly
as
defined by the length attribute of the header. If it is not an exact match the receiver must treat the transmission as
malformed.

### Dealing with Multiple Threads

As the server and client are designed to handle multiple asynchronous tasks a problem arises when data arrives for
different
tasks at the same time. The solution is to assign one thread to for each stream to record the flow byte on outgoing
transmissions and to ensure returning data is delivered to the right task.

The flow byte of 0 is reserved for the Ping/Pong loop.

The remaining are to be divided as follows:
01 - 7F | Remote Client
80 - FF | Server

This is to prevent an accidental collision of the flow bytes.

### Heartbeat Cycle

In order to keep the TCP connection open and to ensure availability the remote system will send a heartbeat transmission
with no data to the server if the connection does not experience any activity for 10 seconds. The server will reply with
an identical transmission. If either party fails to reply the connection will be considered dead and closed.