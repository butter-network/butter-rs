use std::io::{stdin};
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::{thread, time::Duration};
use std::sync::{Mutex};

use lazy_static::lazy_static;

use butter::line_codec::LineCodec;
use butter::server::Server;
// use crate::client::Client;
use butter::threadpool::ThreadPool;
use butter::peer_to_peer::PeerToPeer;

// There are two types of sockets: Active and passive sockets. Active sockets are the ones which
// have a peer connected at the other end and data can be sent and received at this socket. Passive
// socket can just listen to connection requests - it can never talk to clients, send/receive data.

/// Given a TcpStream:
/// - Deserialize the message
/// - Serialize and write the echo message to the stream
fn server_behaviour(stream: TcpStream) {
    let mut codec = LineCodec::new(stream).unwrap();

    // Read & reverse the received message
    let message: String = codec
        .read_message()
        // Reverse message
        .map(|m| m.chars().rev().collect())
        .unwrap();

    // And use the codec to return it
    codec.send_message(&message).unwrap();
}

fn client_behaviour() {
    loop {
        println!("Send a message:");

        let mut input = String::new();

        stdin()
            .read_line(&mut input)
            .ok()
            .expect("Couldn't read line");

        // for i in KNOWN_HOSTS.lock().unwrap().iter() {
        //     let address = i.to_string() + ":8376";
        //     let stream = TcpStream::connect(address).unwrap();
        //     let mut codec = LineCodec::new(stream).unwrap();
        //     codec.send_message(&request).unwrap();
        //     println!("{}", codec.read_message().unwrap());
        // }
        let stream = TcpStream::connect("127.0.0.1:8376").unwrap();
        let mut codec = LineCodec::new(stream).unwrap();
        codec.send_message(&input).unwrap();
        println!("{}", codec.read_message().unwrap());
    }
}

fn main() {
    let p2p: PeerToPeer = PeerToPeer::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                                          8376, server_behaviour, client_behaviour);
}


// The problem is known_host is owned by the main thread, it is then borrowed by the thread running
// the server functionality of the peer. The compiler doesn't know how long the server_functionality()
// function takes to run and thinks that the main thread may get rid of known_hosts while the server thread is still using it.

// We are using a static mutable variable to store the known hosts, this is safe because both
// threads (client and server) run infinite loops hence "closure may outlive the current function"
// is not an issue error. This is a good discussion and solution: https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton