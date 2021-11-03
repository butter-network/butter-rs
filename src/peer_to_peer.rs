use std::net::{IpAddr, Ipv4Addr, TcpStream, UdpSocket};
use std::thread;
use std::time::Duration;
use rand::Rng;
use crate::server::Server;
use crate::threadpool::ThreadPool;

use lazy_static::lazy_static;

use std::sync::{Mutex};
use crate::line_codec::LineCodec;

lazy_static! {
    static ref KNOWN_HOSTS: Mutex<Vec<IpAddr>> = Mutex::new(Vec::new());
}

fn get_known_hosts(stream: TcpStream) -> () {
    let mut codec = LineCodec::new(stream).unwrap();
    // And use the codec to return it
    codec.send_message("this is the list of known hosts").unwrap();
}


pub struct PeerToPeer {
    ip_address: IpAddr,
    port: u16,
    server: Server,
}

impl PeerToPeer {
    pub fn new(ip_address: IpAddr, port: u16, server_behaviour: fn(TcpStream) -> (),
               client_behaviour: fn(&Mutex<Vec<IpAddr>>) -> ()) -> PeerToPeer {

        // known_hosts - add itself?
        let entrypoint = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        KNOWN_HOSTS.lock().unwrap().push(entrypoint);
        
        thread::spawn(move || {
            // Allow the server to startup before client tries to connect
            thread::sleep(Duration::from_secs(2));
            client_behaviour(&KNOWN_HOSTS);
        });

        let mut server: Server = Server::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8376);

        let pool = ThreadPool::new(4);

        server.register_routes("/".parse().unwrap(), server_behaviour);
        server.register_routes("/get_known_hosts".parse().unwrap(), get_known_hosts);
        
        // This is an infinite loop
        for stream in server.listener.incoming() {
            let stream = stream.unwrap();
            pool.execute(move || {
                let peer_address = stream.peer_addr().unwrap().ip();
                println!("\tNew connection from: {}", peer_address);
                // handler(stream); //TODO: This needs to be made dynamic, depending on the route (means I also need to define some sort of stream request format)
                server_behaviour(stream);
                if !KNOWN_HOSTS.lock().unwrap().contains(&peer_address) {
                    KNOWN_HOSTS.lock().unwrap().push(peer_address);
                }
            });
        }

        PeerToPeer {
            ip_address,
            port,
            server,
        }
    }

    // fn run()

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
                },
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
}