use std::io::{stdin};
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::sync::{Mutex, Arc};
use std::mem;

use butter::line_codec::LineCodec;
use butter::peer_to_peer::{PeerToPeer};
use butter::peer_to_peer;

// TODO: Look at next videos (object + blockchain videos)
// TODO: Test using local machine and docker container with their respective IP addressed (not the loopback address)

fn server_behaviour(message: String) -> String {
    // let mut codec = LineCodec::new(stream).unwrap();
    //
    // let mut message = codec.read_message().unwrap();
    //
    // let mut answer = String::new();
    // if message.eq("/known_hosts") {
    //     // answer = peer_to_peer::get_known_hosts();
    // } else {
    //     answer = message.chars().rev().collect();
    // }

    // Read & reverse the received message


    // And use the codec to return it
    message.chars().rev().collect()
}

fn client_behaviour(known_hosts: Vec<IpAddr>) {
    loop {
        println!("Send a message:");

        let mut input = String::new();

        stdin()
            .read_line(&mut input)
            .ok()
            .expect("Couldn't read line");

        // let lock = known_hosts.lock().unwrap();

        for i in known_hosts.iter() {
            let address = i.to_string() + ":8376";
            let stream = TcpStream::connect(address).unwrap();
            let mut codec = LineCodec::new(stream).unwrap();
            codec.send_message(&input).unwrap();
            println!("{}", codec.read_message().unwrap());
        }
        // drop(lock);
    }
}

fn main() {
    let p2p: PeerToPeer = PeerToPeer::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                                          8376, server_behaviour, client_behaviour);
    p2p.start();
}
