# Rust Remote Management Toolkit (RRMT)
The goal of RRMT is to create a service that will provide a tool that 
administrators can use to preform remote work on a system even if 
that system crosses firewalls.

To accomplish this each remote host maintains a constant websocket 
connection to the management server. When a administrator wants to connect
they will use the cli to request a shell with the management server. 
The management server utilizes the websocket to instruct the remote host
to use a tool like netcat to open a shell and forward it to the 
management server, which will in turn forward that again to the Administrator

See diagram:  
![System Diagram](static/diagram.png?)
