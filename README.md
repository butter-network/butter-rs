# Butter
> The network that spreads :sunglasses:!

Butter is a platform/library that supports the development of efficient decentralised applications.

## Design

<!--Add the design stuff I took out of the report-->

The peer to peer objects generates both a server and a client with specified server and client behaviours which have to take the form...

The server run in its own thread in an infinite loop, listening for streams and acting upon them with the specified behaviour - it is worth noting that the server itself handles connections within a threadpool

the client and server functionality 4run in different threads which allow both operations to happen at the same time

Sharing the known hosts list is tricky:
1. Nodes could have function that allows them to ask for list of known hosts from other nodes
2. Nodes could periodically broadcast their list out for everyone to listen

W have to commandeer the client function if we need to ask for a list
Having the list only shared when it's asked for will take a lot less network traffic than broadcasting the list periodically to the whole network, this might also leed to expoential list growwth 

the server specifies a rout by which it will server you something

## Workflow
We want to be testing with different IPs (at least on the local network). We can use containers to have several IPs and test the system.

To open an interactive shell 
```bash
docker-compose run node bash
```

Very good resource for Kademlia implementation in rust: https://github.com/f0lg0/kademlia-dht