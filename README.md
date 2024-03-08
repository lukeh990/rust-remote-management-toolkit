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

## RRMT Protocol
The RRMT Protocol is a simple binary protocol designed specifically to maintain 
an open connection with the Management Server and to instruct the remote clients 
on how to start the remote shell. 

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

