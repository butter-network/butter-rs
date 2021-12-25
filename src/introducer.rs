use std::net::SocketAddr;
use std::sync::Arc;

use crate::tcp_listener::Listener;
use crate::line_codec::LineCodec;
use crate::threadpool::ThreadPool;
use crate::utils;


// NAT IDEA - https://docs.libp2p.io/concepts/nat/
// The network would look like a collection of subnetworks with no interconnections.
// This is why we need NAT
// Router IP acts as the IP for the entire network - the difficult thing is determining what node on the local network needs to process the responce
// The solution to that is port forwarding i.e. assign a port of the router to point to a specific peer on the local network
// This can be configure in an automated way by using upnp.


// This is a server (yes this is a server and as it is a listener but not a peer) that
// has a queue of people that want to make friends and allows friends to be made across sub-networks
// have this running on a cheap server e.g. aws/droplet - anyone can run an introduction server
// An introducer puts into contact a new lonely peer with a peer that wants to make a friend
struct Introducer {
    listener: Listener,
    lonely_peers: Vec<SocketAddr>,
    pool: ThreadPool,
}

impl Introducer {
    pub fn new() -> Introducer {
        let listener: Listener = Listener::new(utils::get_local_ip().unwrap(), 0);
        let pool = ThreadPool::new(4);
        let lonely_peers: Vec<SocketAddr> = Vec::new();
        Introducer {
            listener,
            lonely_peers,
            pool,
        }
    }

    pub fn run(&self) {
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            let lonely_peers = Arc::clone(&self.lonely_peers); // This might cause a big overhead? Maybe make known hosts static?
            self.pool.execute(move || {
                // PROBLEM: the UDP multicast caller is being added to the known hosts but it should
                // not be! Instead we need to add the TCP soccer of the calling node not it's caller UDP.
                let peer_address = stream.peer_addr().unwrap();
                println!("\tNew connection from: {}", peer_address);
                let mut codec = LineCodec::new(stream).unwrap();
                if codec.read_message().unwrap() == "I'm a new lonely peer" {
                    // add him to my queue of lonely peers
                    lonely_peers.lock().push(peer_address);
                } else {
                    // I'm just lonely and I want to make a friend (i.e. I want
                    // pop from the queue of lonely peers
                    // check if the ip address indicates that it is part of a different sub-network (an we are not making friend with nodes that are already on the same network)
                    lonely_peers.lock().get(0)
                    // and introduce them by sending socket address to one of the peer
                }
            });
        }
    }
}