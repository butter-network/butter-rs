// Network wide chat example.

use std::io::{stdin};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream, UdpSocket, Ipv4Addr};
use std::sync::{Mutex, Arc};
use std::thread;
use std::time;

use butter::line_codec::LineCodec;
use butter::peer_to_peer::{PeerToPeerNode};
use butter::peer_to_peer;
use std::io::Error;

fn server_behaviour(message: String) -> String {
    message.chars().rev().collect()
}

fn client_behaviour(known_hosts: Arc<Mutex<Vec<SocketAddr>>>) {
    loop {
        println!("Send a message:");

        let mut input = String::new();

        stdin()
            .read_line(&mut input)
            .ok()
            .expect("Couldn't read line");

        let mut lock = known_hosts.lock().unwrap();
        let mut known_hosts_sto = lock.clone();
        drop(lock);

        for i in known_hosts_sto.iter() {
            // let address =  i.to_string() + ":8376";
            println!("{}", i.to_string());
            let stream = TcpStream::connect(i);
            // Don't panic if it fails, some nodes may just be dead, when dead, just remove the node from the list instead of panicking
            let stream = match stream {
                Ok(stream) => {
                    let mut codec = LineCodec::new(stream).unwrap();
                    codec.send_message(&input).unwrap();
                    println!("{}", codec.read_message().unwrap());
                },
                Err(e) => {
                    println!("{}", e);
                    println!("{}", "Couldn't connect to peer");
                    // known_hosts_sto.remove(i);
                }
            };
        }
    }
}



fn main() {
    let p2p: PeerToPeerNode = PeerToPeerNode::new(8376, server_behaviour, client_behaviour);
    p2p.start();
}