use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::thread;
use sysinfo::{System, SystemExt};
use std::mem;

use crate::threadpool::ThreadPool;
use crate::server::TcpServer;
use crate::codec::LineCodec;
use crate::discover;
use crate::utils;

#[derive(Debug, Clone)]
pub struct Node {
    ip_address: IpAddr,
    port: u16,
    pub known_hosts: Arc<Mutex<Vec<SocketAddr>>>,
    // some form of storage data structure? hashmap<key:UUID, value: Data> or use JSON as the a sort of key value store
}

impl Node {
    pub fn new(port: Option<u16>) -> Node {

        // Get the local ip address of the machine
        let ip_address = utils::get_local_ip().unwrap();

        // Get the user defined port
        // If port is unspecified, the port is set to zero and allocated by the OS when the node is
        // started
        let port = port.unwrap_or(0);

        // Determining appropriate known host list size
        let mut sys = System::new();
        sys.refresh_all();
        // let known_host_allocation_size = mem::size_of::<SocketAddr>();
        // println!("{}", known_host_allocation_size);
        // let known_hosts: [SocketAddr; known_host_allocation_size as usize] = [];

        // Create the list of known hosts to this node
        let known_hosts = Arc::new(Mutex::new(Vec::new()));

        if port == 0 {
            println!("Unspecified port, will use port allocated by OS");
            println!("Creating node with \n- ip address: {} \n- port: UNSPECIFIED", ip_address);
        } else {
            println!("Creating node with \nip address: {} \nport: {}", ip_address, port);
        }

        Node {
            ip_address,
            port,
            known_hosts,
            // storage,
        }
    }

    pub fn start(self, server_behaviour: fn(Node, String) -> String,
                 client_behaviour: fn(Node) -> ()) {

        let server: TcpServer = TcpServer::new(self.ip_address, self.port);
        println!("Node will listen at: {}", server.listener.local_addr().unwrap());

        let pool = ThreadPool::new(4);

        // let known_hosts = self.known_hosts;
        let known_hosts_client = Arc::clone(&self.known_hosts);
        let known_hosts_discover = Arc::clone(&self.known_hosts);

        let listener = server.listener;

        // STARTUP PROCEDURE - multicast calling out
        // before I start the data layer of the p2p network TCP, I need to go through the start up
        // procedure to make at least one connection to the network
        let socket = listener.local_addr().unwrap();
        let server_address = (&listener.local_addr().unwrap()).clone();
        thread::Builder::new().name("introduction_layer_caster".to_string()).spawn(move || { // this probably doesn't need to be in a thread cause i need to wait for a response before i can work with the data layer anyways
            discover::run(&socket, |s| {
                let known_hosts = Arc::clone(&known_hosts_discover);
                handle_introduction(s, known_hosts, server_address)}).unwrap();
        });

        // Client thread, running client behaviour
        let node_cp = self.clone();
        thread::Builder::new().name("conversation_layer_talker".to_string()).spawn(move || {
            (client_behaviour)(node_cp);
        });

        // Server runs on main thread and handles connections in a threadpool
        // let known_hosts_for_server = self.known_hosts;

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let known_hosts_server = Arc::clone(&self.known_hosts); // This might cause a big overhead? Maybe make known hosts static?
            let node_cp_for_server = self.clone();
            pool.execute(move || {
                let peer_address = stream.peer_addr().unwrap();
                println!("\tNew connection from: {}", peer_address);
                let mut codec = LineCodec::new(stream).unwrap();
                let message = codec.read_message().unwrap();
                let uri = message.split_whitespace().nth(0).unwrap();
                let mut reply = String::new();
                println!("{}", uri);
                if message == "/known_hosts" {
                    for host in known_hosts_server.try_lock().unwrap().iter() {
                        reply.push_str(host.to_string().as_str());
                        reply.push_str(",");
                    }
                } else if message == ""  { // at the moment the UDP call just sends an empty message so this is a hack to not add the udp caller to the known host list
                    // do nothing - don't add UDP server to list - later customise the message to be a specific route
                    println!("I'm here")
                } else if uri == "/let_me_introduce_myself" {
                    let remote_server_address = message.split_whitespace().nth(1).unwrap();
                    println!("The other server is: {}", remote_server_address);
                    let remote_server_address_sock = SocketAddr::from_str(remote_server_address).unwrap();
                    let mut lock = known_hosts_server.try_lock().unwrap();
                    if !lock.contains(&remote_server_address_sock) {
                        lock.push(remote_server_address_sock); // this is pushing the socker address of generated clients not of the listener
                    }
                } else {
                    reply = (server_behaviour)(node_cp_for_server, message);
                }
                let mut lock = known_hosts_server.try_lock().unwrap();
                println!("{}", lock.len());
                codec.send_message(reply.as_str()).unwrap();
            });
        }
    }

    // High level API call that allows users to store information on the network persistently
    // It may end up storing t information on this peer or on other peers

    fn store() {}

    // this needs to be recursive i.e. look if I have the information else look at the most likely known host to get to that information
    fn get() {}

    fn query() {}

    // removes dead known hosts - runs periodically
    fn clean() {}

    // In UDP, the client does not form a connection with the server like in TCP and instead just
    // sends a datagram. Similarly, the server need not accept a connection and just waits for
    // datagrams to arrive. Datagrams upon arrival contain the address of sender which the server
    // uses to send data to the correct client.



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

fn handle_introduction(stream: std::io::Result<TcpStream>, known_hosts: Arc<Mutex<Vec<SocketAddr>>>, server_address: SocketAddr) {
    let stream = stream.unwrap();
    let peer_address = stream.peer_addr().unwrap();
    println!("Got a reply from {}", peer_address);
    // add him to my known hosts and ask hi who he knows
    let mut lock = known_hosts.try_lock().unwrap();
    if !lock.contains(&peer_address) {
        lock.push(peer_address);
    }
    // send them my address
    let mut codec = LineCodec::new(stream).unwrap();
    let mut reply: String = "".to_owned();
    reply.push_str("/let_me_introduce_myself ");
    reply.push_str(server_address.to_string().as_str());
    codec.send_message(reply.as_str()).unwrap();
}


// Am I going to give the user the option to define routes?
// pub fn register_server_route(&mut self, route: String, behaviour: fn(TcpStream) -> ()) {
//     self.server.register_routes(route, behaviour);
// }