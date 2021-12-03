# Butter
> The network that spreads :sunglasses:!

Butter is a platform/library that supports the development of efficient decentralised applications.

## Project overview

How am I buidling my decentralised platform/framework

## The vocabulary - detaching oursels from thinking in a clinet/server way
a server = a listener

a client = a caller

a peer = a node on the network that interacts both as a listener and caller (it behaves as both a client and a server)

what nodes exists on a butter network
- peers
- introducers (allows two new-subnetworks to be introduces to each other, once introduce they interact direccly)

## Design

<!--Add the design stuff I took out of the report-->

**The problems**

1. Peer discovery

- Finish this sort out how to make both nodes aware of existence and stop broadcasting

3. NAT traversal

- this is a problem for listing to incoming connections
- either just port forwarding (but that means having a specific port assigned to the app) or use UPNP (i think i'm gonna go for upnp)

Peer discovery + NAT traversal protocol

```pseudocode
(not a requirment of the butter network is that you have a public IP that does not chnage too frequently + you know how to port forward)
1. Use a port for the butter network e.g. 2000 (this does allow several computers on the same network to exist as they have different local IPs)
2. A user can port forward all the different local ips to a different port of the router (think of it as habing many servers all with the server port being 2000 but the router port being whatevers available)
3. Peer discovery
	a. multicast for local discovery
	b. annouce yourself to very basic introduction server (not used for data but just to allow nodes to meet and then they can connect directy)
		i. add your address to the queue on the server
		ii. another computer who's looking for a more complete known host list asks if he can be introduced to someone new
```



5. Choosing who to be friends with (first degree connection) - who to have in my `known_hosts`

   Devise a heuristic for peers which determines the maintenance of the known hosts list - call it optimising for good friends :smile: (someone who's always there for you vs. someone who you see from time to time but that relies on you)

   develop a mathetmatica model for a good group of friends --> diverse

   - Each peer has an uptime score --> used to put them into brackets of: "friend that's always there for you", "friend that sometimes there for you" and "flaky friend" (a friend you can be there for - hard but can be rewarding)
   - chose size of known host list to be dynamic based on user preference/hardware ability
   - if you feel lonely multicast (local friend-making) and go through introducer (oversees friend-making) 

6. Getting the information from peers - assume that's DHT?

   1. Breaking down information into chunks
   2. Distributing chunks
   3. Maintaining data consistency (even upon peer failure)

   ? devise a heuristic for "who knows what" i.e. hash information in such a way that the hashes infer how similar the information is so you look through your known hosts and see if his hash is a little similar and then ask his known hosts etc until you find information... (this has implications in information distribution too)

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


Resources
https://jsantell.com/p2p-peer-discovery/

https://docs.libp2p.io/concepts/nat/
