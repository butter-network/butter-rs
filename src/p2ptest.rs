// use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
// use std::str::FromStr;
// use std::thread;
// use std::time::Duration;
// use crate::server::{TcpServer};
// use crate::threadpool::ThreadPool;
//
// use std::sync::{Arc, Mutex};
// use crate::line_codec::LineCodec;
//
// use crate::discover;
// use crate::utils;
//
//
// // A node is simply aa thing - it is inert, it simply exists and has certain properties hence it is described as a struct
// // A node becomes a peer when it has traits (i.e. functionality) that let it interact with the world
//
//
// pub struct Node {
//     ip_address: IpAddr,
//     port: u16,
//     server: TcpServer,
//     known_hosts: Arc<Mutex<Vec<SocketAddr>>>,
// }
//
// impl Peer for Node {
//     fn client_behaviour(&self) {
//         todo!()
//     }
//
//     fn server_behaviour(&self) {
//         todo!()
//     }
// }
//
//
// pub trait Peer {
//     // Default constructor for a node with peer traits
//     fn new(port: Option<u16>) -> Node {
//         // Get computer's IPV4 address on the local network
//         let ip_address = utils::get_local_ip().unwrap();
//
//         // The size of the known host list could be determines upon instantiation so maybe I can make this not dynamic
//         let known_hosts = Arc::new(Mutex::new(Vec::new()));
//
//         // If specified use the desired port, else let OS assign a port
//         let server: TcpServer = TcpServer::new(ip_address, port.unwrap_or(0));
//
//         println!("TCP server address: {}", server.listener.local_addr().unwrap());
//
//        Node {
//            ip_address,
//            port: server.listener.local_addr().unwrap().port(),
//            server,
//            known_hosts,
//        }
//     }
//     fn run(&self) {
//         let pool = ThreadPool::new(4);
//
//         // i don't need to move entire self into the thread scope - I just need to move the client server
//         // now client behaviour owns self.client behaviour - right?
//         // this is why rust is good! I create the object and then move exactly what I need where I need it by changing the ownership - this frees the memory previously held by the object
//         let known_hosts = self.known_hosts;
//         let known_hosts_client = Arc::clone(&known_hosts);
//         let known_hosts_discover = Arc::clone(&known_hosts);
//
//         let listener = self.server.listener;
//         let server_address = (&listener.local_addr().unwrap()).clone();
//
//         // STARTUP PROCEDURE - multicast calling out
//         // before I start the data layer of the p2p network TCP, I need to go through the start up
//         // procedure to make at least one connection to the network
//         let socket = listener.local_addr().unwrap();
//         thread::Builder::new().name("introduction_layer_caster".to_string()).spawn(move || { // this probably doesn't need to be in a thread cause i need to wait for a response before i can work with the data layer anyways
//             // this function blocks forever; running it a seperate thread
//             // autodiscover_rs::run(&socket, autodiscover_rs::Method::Multicast("[ff0e::1]:1337".parse().unwrap()),|s| {
//             // change this to task::spawn if using async_std or tokio
//             // thread::spawn(move || handle_introduction(s, known_hosts_discover));
//             // }).unwrap();
//             discover::run(&socket, |s| {
//                 let known_hosts = Arc::clone(&known_hosts_discover);
//                 handle_introduction(s, known_hosts, server_address)}).unwrap();});
//
//         // Once it has introduced itslef it needs to stop multicasting!! A the moment it continually multicasts
//
//         // Client thread, running client behaviour
//         thread::Builder::new().name("conversation_layer_talker".to_string()).spawn(move || {
//             // Allow the server to startup before client tries to connect
//             // thread::sleep(Duration::from_secs(2));
//             self.client_behaviour(known_hosts_client);
//         });
//
//         // Server runs on main thread and handles connections in a threadpool
//         for stream in listener.incoming() {
//             let stream = stream.unwrap();
//             let known_hosts_server = Arc::clone(&known_hosts); // This might cause a big overhead? Maybe make known hosts static?
//             pool.execute(move || {
//                 // PROBLEM: the UDP multicast caller is being added to the known hosts but it should
//                 // not be! Instead we need to add the TCP soccer of the calling node not it's caller UDP.
//                 let peer_address = stream.peer_addr().unwrap();
//                 println!("\tNew connection from: {}", peer_address);
//                 // handler(stream); //TODO: This needs to be made dynamic, depending on the route (means I also need to define some sort of stream request format)
//                 let mut codec = LineCodec::new(stream).unwrap();
//                 let message = codec.read_message().unwrap();
//                 let uri = message.split_whitespace().nth(0).unwrap();
//                 let mut reply = String::new();
//                 println!("{}", uri);
//                 if message == "/known_hosts" {
//                     for host in known_hosts_server.try_lock().unwrap().iter() {
//                         reply.push_str(host.to_string().as_str());
//                         reply.push_str(",");
//                     }
//                 } else if message == ""  { // at the moment the UDP call just sends an empty message so this is a hack to not add the udp caller to the known host list
//                     // do nothing - don't add UDP server to list - later customise the message to be a specific route
//                     println!("I'm here")
//                 } else if uri == "/let_me_introduce_myself" {
//                     let remote_server_address = message.split_whitespace().nth(1).unwrap();
//                     println!("The other server is: {}", remote_server_address);
//                     let remote_server_address_sock = SocketAddr::from_str(remote_server_address).unwrap();
//                     let mut lock = known_hosts_server.try_lock().unwrap();
//                     if !lock.contains(&remote_server_address_sock) {
//                         lock.push(remote_server_address_sock); // this is pushing the socker address of generated clients not of the listener
//                     }
//                 } else {
//                     reply = self.server_behaviour(message);
//                     // reply = server_behaviour_trait(message);
//                     // let mut lock = known_hosts_server.try_lock().unwrap();
//                     // if !lock.contains(&peer_address) {
//                     //     lock.push(peer_address); // this is pushing the socker address of generated clients not of the listener
//                     // }
//                 }
//                 let mut lock = known_hosts_server.try_lock().unwrap();
//                 println!("{}", lock.len());
//                 codec.send_message(reply.as_str()).unwrap();
//             });
//         }
//     }
//     fn client_behaviour(&self);
//     fn server_behaviour(&self);
// }
//
//
// fn handle_introduction(stream: std::io::Result<TcpStream>, known_hosts: Arc<Mutex<Vec<SocketAddr>>>, server_address: SocketAddr) {
//     let stream = stream.unwrap();
//     let peer_address = stream.peer_addr().unwrap();
//     println!("Got a reply from {}", peer_address);
//     // add him to my known hosts and ask hi who he knows
//     let mut lock = known_hosts.try_lock().unwrap();
//     if !lock.contains(&peer_address) {
//         lock.push(peer_address);
//     }
//     // send them my address
//     let mut codec = LineCodec::new(stream).unwrap();
//     let mut reply: String = "".to_owned();
//     reply.push_str("/let_me_introduce_myself ");
//     reply.push_str(server_address.to_string().as_str());
//     codec.send_message(reply.as_str()).unwrap();
// }