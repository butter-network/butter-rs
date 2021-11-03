use std::io::{stdin};
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::sync::{Mutex};

use butter::line_codec::LineCodec;
use butter::peer_to_peer::PeerToPeer;

// TODO: Look at next videos (object + blockchain videos)
// TODO: Test using local machine and docker container with their respective IP addressed (not the loopback address)

fn server_behaviour(message: String) -> String {
    // let mut codec = LineCodec::new(stream).unwrap();

    // Read & reverse the received message
    // let message: String = codec
    //     .read_message()
    //     // Reverse message
    //     .map(|m| m.chars().rev().collect())
    //     .unwrap();
    //
    // // And use the codec to return it
    // codec.send_message(&message).unwrap();
    message.map(|m| m.chars().rev().collect()).unwrap()
}

fn client_behaviour(known_hosts: &Mutex<Vec<IpAddr>>) {
    loop {
        println!("Send a message:");

        let mut input = String::new();

        stdin()
            .read_line(&mut input)
            .ok()
            .expect("Couldn't read line");

        for i in known_hosts.lock().unwrap().iter() {
            let address = i.to_string() + ":8376";
            let stream = TcpStream::connect(address).unwrap();
            let mut codec = LineCodec::new(stream).unwrap();
            codec.send_message(&input).unwrap();
            println!("{}", codec.read_message().unwrap());
        }
    }
}

fn main() {
    let p2p: PeerToPeer = PeerToPeer::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                                          8376, server_behaviour, client_behaviour);
}
