use std::io::{stdin};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream, UdpSocket, Ipv4Addr};
use std::sync::{Mutex, Arc};
use std::thread;
use std::time;

use butter::line_codec::LineCodec;
use butter::peer_to_peer::{PeerToPeer};
use butter::peer_to_peer;
use butter::multicast::test;
use std::io::Error;

// TODO: Look at next videos (object + blockchain videos)
// TODO: Test using local machine and docker container with their respective IP addressed (not the loopback address)
fn server_behaviour(message: String) -> String {
    message.chars().rev().collect()
}

fn client_behaviour(known_hosts: Arc<Mutex<Vec<IpAddr>>>) {
    loop {
        println!("Send a message:");

        let mut input = String::new();

        stdin()
            .read_line(&mut input)
            .ok()
            .expect("Couldn't read line");

        let mut lock = known_hosts.lock().unwrap();
        let known_hosts_sto = lock.clone();
        drop(lock);

        for i in known_hosts_sto.iter() {
            let address =  i.to_string() + ":8376";
            let stream = TcpStream::connect("127.0.0.1:8376").unwrap();
            let mut codec = LineCodec::new(stream).unwrap();
            codec.send_message(&input).unwrap();
            println!("{}", codec.read_message().unwrap());
        }
    }
}

fn main() {
    // let p2p: PeerToPeer = PeerToPeer::new(8376, server_behaviour, client_behaviour);
    // p2p.start();
    test();
}