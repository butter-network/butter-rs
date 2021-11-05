use std::net::{IpAddr, Ipv4Addr, TcpStream, UdpSocket};
use std::thread;
use std::time::Duration;
use rand::Rng;
use crate::server::Server;
use crate::threadpool::ThreadPool;

use std::sync::{Arc, Mutex};
use crate::line_codec::LineCodec;


pub struct PeerToPeer {
    ip_address: IpAddr,
    port: u16,
    server: Server,
    server_behaviour: fn(TcpStream) -> (),
    client_behaviour: fn(Arc<Mutex<Vec<IpAddr>>>) -> (),
    known_hosts: Arc<Mutex<Vec<IpAddr>>>,
}

impl PeerToPeer {
    pub fn new(ip_address: IpAddr, port: u16, server_behaviour: fn(TcpStream) -> (),
               client_behaviour: fn(Arc<Mutex<Vec<IpAddr>>>) -> ()) -> PeerToPeer {

        // known_hosts - add itself?
        let entrypoint = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        // known_hosts.lock().unwrap().push(entrypoint);

        let known_hosts = Arc::new(Mutex::new(Vec::new()));
        known_hosts.lock().unwrap().push(entrypoint);

        let server: Server = Server::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8376);

        // server.register_routes("/".parse().unwrap(), server_behaviour);
        // server.register_routes("/get_known_hosts".parse().unwrap(), get_known_hosts);

        // makes sense for this to be static as it will exist for the entire runtime of the program and needs to be accessed by several threads all of which run in infinte loops
        // this prevents having to copy the whole object between each thread (moving ownsership of a version of the ovbject constantly)

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

        // Client thread, running client behaviour
        thread::spawn(move || {
            // Allow the server to startup before client tries to connect
            thread::sleep(Duration::from_secs(2));
            (client_behaviour)(known_hosts_client);
        });

        // Server runs on main thread and handles connections in a threadpool
        for stream in self.server.listener.incoming() {
            let stream = stream.unwrap();
            let known_hosts_server = Arc::clone(&known_hosts); // This might cause a big overhead? Maybe make known hosts static?
            pool.execute(move || {
                let peer_address = stream.peer_addr().unwrap().ip();
                println!("\tNew connection from: {}", peer_address);
                // handler(stream); //TODO: This needs to be made dynamic, depending on the route (means I also need to define some sort of stream request format)
                (server_behaviour)(stream);
                if !known_hosts_server.lock().unwrap().contains(&peer_address) {
                    known_hosts_server.lock().unwrap().push(peer_address);
                }
            });
        }
    }

    // Upon initialising the peer, introduce yourself to the network to avoid cold start problem
    fn introduce_yourself_naive() {
        // generate random ip address and ask for known hosts
        // How long will it take to get a hit?
        loop {
            let rand_ip = IpAddr::V4(Ipv4Addr::new(rand::thread_rng().gen_range(0..255),
                                                   rand::thread_rng().gen_range(0..255),
                                                   rand::thread_rng().gen_range(0..255),
                                                   rand::thread_rng().gen_range(0..255)));
            let address = rand_ip.to_string() + ":0";
            // try to connect to that IP address
            let stream = TcpStream::connect(address);
            match stream {
                Ok(stream) => {
                    // println!("\tConnected to: {}", address);
                    let mut codec = LineCodec::new(stream).unwrap();
                    codec.send_message("/known_host").unwrap();
                    println!("{}", codec.read_message().unwrap());
                    break;
                }
                Err(_) => {
                    // do nothing and loop
                }
            }
        }
    }

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

    pub fn get_known_hosts(self) -> String {
        let mut message = String::new();
        for host in self.known_hosts.lock().unwrap().iter() {
            message.push_str(host.to_string().as_str());
            message.push_str(",");
        }
        message
    }
}