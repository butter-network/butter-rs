// Network wide chat example.

use std::io::{stdin};
use std::net::TcpStream;
use butter::codec::LineCodec;
use butter::peer::Node;

fn server_behaviour(node: Node, incoming_message: String) -> String {
    incoming_message.chars().rev().collect()
}

fn client_behaviour(node: Node) {
    loop {
        println!("Send a message:");

        let mut input = String::new();

        stdin()
            .read_line(&mut input)
            .ok()
            .expect("Couldn't read line");

        let lock = node.known_hosts.lock().unwrap();
        let known_hosts_sto = lock.clone();
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
    let node: Node = Node::new(None);
    node.start(server_behaviour, client_behaviour);
}