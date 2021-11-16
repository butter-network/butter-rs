use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
use std::thread;
use std::time::Duration;
// use rand::Rng;
use local_ip_address::local_ip;
use crate::server::{TCPServer};
use crate::threadpool::ThreadPool;

use std::sync::{Arc, Mutex};
use crate::line_codec::LineCodec;

use crate::discover;
// use autodiscover_rs;

// EPICS
// - Implement a peer on the network (TCP sender/recipient)
// - Implement peer discovery mechanisms
//    - LAN dicovery using UDP multicast
//    - WAN disovery using NAT UPNP
// - ? implement dht?

//TODO: 1. Move discovery to be handled internally
//TODO: 2. Fix error when other node sends message
//TODO: 3. Stop calling out to the network when the node is not connected

// NAT IDEA - https://docs.libp2p.io/concepts/nat/
// The network would look like a collection of subnetworks with no interconnections.
// This is why we need NAT
// Router IP acts as the IP for the entire network - the difficult thing is determining what node on the local network needs to process the responce
// The solution to that is port forwarding i.e. assign a port of the router to point to a specific peer on the local network
// This can be configure in an automated way by using upnp.


pub struct PeerToPeer {
    ip_address: IpAddr,
    port: u16,
    server: TCPServer,
    server_behaviour: fn(String) -> String,
    client_behaviour: fn(Arc<Mutex<Vec<SocketAddr>>>) -> (),
    known_hosts: Arc<Mutex<Vec<SocketAddr>>>,
}

impl PeerToPeer {
    pub fn new(port: u16, server_behaviour: fn(String) -> String,
               client_behaviour: fn(Arc<Mutex<Vec<SocketAddr>>>) -> ()) -> PeerToPeer {

        let ip_address = local_ip().unwrap();
        // known_hosts - add itself?
        // let entrypoint = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        // known_hosts.lock().unwrap().push(entrypoint);

        let known_hosts = Arc::new(Mutex::new(Vec::new()));
        // known_hosts.lock().unwrap().push(entrypoint);

        let server: TCPServer = TCPServer::new(ip_address, 0);

        // server.register_routes("/".parse().unwrap(), server_behaviour);
        // server.register_routes("/get_known_hosts".parse().unwrap(), get_known_hosts);

        // makes sense for this to be static as it will exist for the entire runtime of the program and needs to be accessed by several threads all of which run in infinte loops
        // this prevents having to copy the whole object between each thread (moving ownsership of a version of the ovbject constantly)


        // println!("This is my local IP address: {:?}", ip_address);

        PeerToPeer {
            ip_address,
            port,
            server,
            server_behaviour,
            client_behaviour,
            known_hosts,
        }
    }

    pub fn start(self) {
        let pool = ThreadPool::new(4);

        // i don't need to move entire self into the thread scope - I just need to move the client server
        // now client behaviour owns self.client behaviour - right?
        // this is why rust is good! I create the object and then move exactly what I need where I need it by changing the ownership - this frees the memory previously held by the object
        let client_behaviour = self.client_behaviour;
        let server_behaviour = self.server_behaviour;
        let known_hosts = self.known_hosts;
        let known_hosts_client = Arc::clone(&known_hosts);
        let known_hosts_discover = Arc::clone(&known_hosts);

        let listener = self.server.listener;

        // STARTUP PROCEDURE - multicast calling out
        // before I start the data layer of the p2p network TCP, I need to go through the start up
        // procedure to make at least one connection to the network
        let socket = listener.local_addr().unwrap();
        thread::Builder::new().name("introduction_layer_caster".to_string()).spawn(move || { // this probably doesn't need to be in a thread cause i need to wait for a response before i can work with the data layer anyways
            // this function blocks forever; running it a seperate thread
            // autodiscover_rs::run(&socket, autodiscover_rs::Method::Multicast("[ff0e::1]:1337".parse().unwrap()),|s| {
                // change this to task::spawn if using async_std or tokio
                // thread::spawn(move || handle_introduction(s, known_hosts_discover));
            // }).unwrap();
            discover::run(&socket, |s| {
                let known_hosts = Arc::clone(&known_hosts_discover);
                handle_introduction(s, known_hosts)}).unwrap();});

        // Once it has introduced itslef it needs to stop multicasting!! A the moment it continually multicasts

        // Client thread, running client behaviour
        thread::Builder::new().name("conversation_layer_talker".to_string()).spawn(move || {
            // Allow the server to startup before client tries to connect
            thread::sleep(Duration::from_secs(2));
            (client_behaviour)(known_hosts_client);
        });

        // Server runs on main thread and handles connections in a threadpool
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let known_hosts_server = Arc::clone(&known_hosts); // This might cause a big overhead? Maybe make known hosts static?
            pool.execute(move || {
                // PROBLEM: the UDP multicast caller is being added to the known hosts but it should
                // not be! Instead we need to add the TCP soccer of the calling node not it's caller UDP.
                let peer_address = stream.peer_addr().unwrap();
                println!("\tNew connection from: {}", peer_address);
                // handler(stream); //TODO: This needs to be made dynamic, depending on the route (means I also need to define some sort of stream request format)
                let mut codec = LineCodec::new(stream).unwrap();
                let message = codec.read_message().unwrap();
                let mut reply = String::new();
                println!("{}", message);
                if message == "/known_hosts" {
                    for host in known_hosts_server.try_lock().unwrap().iter() {
                        reply.push_str(host.to_string().as_str());
                        reply.push_str(",");
                    }
                } else if message == ""  { // at the moment the UDP call just sends an empty message so this is a hack to not add the udp caller to the known host list
                    // do nothing - don't add UDP server to list - later customise the message to be a specific route
                    println!("I'm here")
                } else {
                    reply = (server_behaviour)(message);
                    let mut lock = known_hosts_server.try_lock().unwrap();
                    if !lock.contains(&peer_address) {
                        lock.push(peer_address);
                    }
                }
                let mut lock = known_hosts_server.try_lock().unwrap();
                println!("{}", lock.len());
                codec.send_message(reply.as_str()).unwrap();
            });
        }
    }

    // Upon initialising the peer, introduce yourself to the network to avoid cold start problem
    // fn introduce_yourself_naive() {
    //     // generate random ip address and ask for known hosts
    //     // How long will it take to get a hit?
    //     loop {
    //         let rand_ip = IpAddr::V4(Ipv4Addr::new(rand::thread_rng().gen_range(0..255),
    //                                                rand::thread_rng().gen_range(0..255),
    //                                                rand::thread_rng().gen_range(0..255),
    //                                                rand::thread_rng().gen_range(0..255)));
    //         let address = rand_ip.to_string() + ":0";
    //         // try to connect to that IP address
    //         let stream = TcpStream::connect(address);
    //         match stream {
    //             Ok(stream) => {
    //                 // println!("\tConnected to: {}", address);
    //                 let mut codec = LineCodec::new(stream).unwrap();
    //                 codec.send_message("/known_host").unwrap();
    //                 println!("{}", codec.read_message().unwrap());
    //                 break;
    //             }
    //             Err(_) => {
    //                 // do nothing and loop
    //             }
    //         }
    //     }
    // }

    // In UDP, the client does not form a connection with the server like in TCP and instead just
    // sends a datagram. Similarly, the server need not accept a connection and just waits for
    // datagrams to arrive. Datagrams upon arrival contain the address of sender which the server
    // uses to send data to the correct client.

    // If I have multiple UDP listeners i.e. servers, will they all respond when 1 client makes a call? Or is it just a single packet that gets intercepted by a server and then destroyed
    fn introduce_yourself_by_shouting() {
        // let socket = UdpSocket::send();
    }

    pub fn register_server_route(&mut self, route: String, behaviour: fn(TcpStream) -> ()) {
        self.server.register_routes(route, behaviour);
    }

    fn handler(&mut self, stream: TcpStream) {
        // Initialise the coded interface
        let mut codec = LineCodec::new(stream).unwrap();

        // Read the message
        let message = codec.read_message().unwrap();

        // get the uri part of the message (which determines what we do with the rest)
        let uri = message.split_whitespace().nth(1).unwrap();

        // call the appropriate behaviour and pass remaining part of message based on the uri
        // self.server.routes.get(uri).unwrap()(stream);
    }


}

fn handle_introduction(stream: std::io::Result<TcpStream>, known_hosts: Arc<Mutex<Vec<SocketAddr>>>) {
    let peer_address = stream.unwrap().peer_addr().unwrap();
    println!("Got a reply from {}", peer_address);
    // add him to my known hosts and ask hi who he knows
    let mut lock = known_hosts.try_lock().unwrap();
    if !lock.contains(&peer_address) {
        lock.push(peer_address);
    }
}