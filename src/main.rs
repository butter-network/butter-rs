use std::io::{stdin};
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::{thread, time::Duration};
use std::collections::LinkedList;
use std::sync::{Mutex, Arc};

mod linecodec;
mod server;
// mod client;

use crate::linecodec::LineCodec;
use crate::server::Server;
// use crate::client::Client;

// There are two types of sockets: Active and passive sockets. Active sockets are the ones which
// have a peer connected at the other end and data can be sent and received at this socket. Passive
// socket can just listen to connection requests - it can never talk to clients, send/receive data.

/// Given a TcpStream:
/// - Deserialize the message
/// - Serialize and write the echo message to the stream
fn handle_client(stream: TcpStream) {
    // let mut data = [0 as u8; 50]; // using 50 byte buffer
    // while match stream.read(&mut data) {
    //     Ok(size) => {
    //         // echo everything!
    //         stream.write(&data[0..size]).unwrap();
    //         true
    //     }
    //     Err(_) => {
    //         println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
    //         stream.shutdown(Shutdown::Both).unwrap();
    //         false
    //     }
    // } {}
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

fn server_functionality(known_hosts: &mut LinkedList<IpAddr>) -> () {
    let server: Server = Server::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8376);
    for stream in server.listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("\tNew connection from: {}", stream.peer_addr().unwrap());
                if !known_hosts.contains(&stream.peer_addr().unwrap().ip()) {
                    known_hosts.push_back(stream.peer_addr().unwrap().ip());
                }
                handle_client(stream);
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
}

fn client_functionality(request: &str) {
    // Establish a TCP connection with the farend
    let stream = TcpStream::connect("127.0.0.1:8376").unwrap();

    // Codec is our interface for reading/writing messages.
    // No need to handle reading/writing directly
    let mut codec = LineCodec::new(stream).unwrap();

    // Serializing & Sending is now just one line
    codec.send_message(&request).unwrap();

    // And same with receiving the response!
    println!("{}", codec.read_message().unwrap());
    // Ok(())

    // TODO: Look at using the stream.shutdown() method...

    // let client: Client = Client::new(0,0,0,0,0);
    // match TcpStream::connect("127.0.0.1:8376") {
    //     Ok(mut stream) => {
    //         println!("Successfully connected to server.");
    //
    //         // let msg = b"Hello!";
    //
    //         stream.write(request.as_bytes()).unwrap();
    //         println!("Sent request, awaiting reply...");
    //
    //         let mut data = [0 as u8; 6]; // using 6 byte buffer
    //         match stream.read_exact(&mut data) {
    //             Ok(_) => {
    //                 if &data == request.as_bytes() {
    //                     println!("Reply is ok!");
    //                 } else {
    //                     let text = from_utf8(&data).unwrap();
    //                     println!("Unexpected reply: {}", text);
    //                 }
    //             }
    //             Err(e) => {
    //                 println!("Failed to receive data: {}", e);
    //             }
    //         }
    //     }
    //     Err(e) => {
    //         println!("Failed to connect: {}", e);
    //     }
    // }
}

fn main() {
    let entrypoint = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let mut known_hosts: LinkedList<IpAddr> = LinkedList::new();
    // let known_hosts_mutex = Arc::new(Mutex::new(known_hosts));
    // known_hosts.push_back(entrypoint);
    // let known_hosts_x = Arc::clone(&known_hosts_mutex);
    thread::spawn(|| {
        // let mut jhgjgf = known_hosts_x.lock().unwrap();
        server_functionality(&mut known_hosts);
    });
    thread::sleep(Duration::from_secs(2));
    println!("Send a message:");

    let mut input = String::new();

    stdin()
        .read_line(&mut input)
        .ok()
        .expect("Couldn't read line");

    // println!("hello {}", input);
    // let known_hosts_y = Arc::clone(&known_hosts_mutex);
    client_functionality(&input);

    println!("Terminated.");
}


// The problem is known_host is owned by the main thread, it is then borrowed by the thread running
// the server functionality of the peer. The compiler doesn't know how long the server_functionality()
// function takes to run and thinks that the main thread may get rid of known_hosts while the server thread is still using it.
