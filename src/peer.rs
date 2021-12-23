use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::thread;
use sysinfo::{System, SystemExt};

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

        // TODO: Determining appropriate known hosts list size upper limit
        let mut sys = System::new();
        sys.refresh_all();
        // let known_host_allocation_size = mem::size_of::<SocketAddr>();
        // println!("{}", known_host_allocation_size);
        // let known_hosts: [SocketAddr; known_host_allocation_size as usize] = [];

        // Create the list of known hosts for the node
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

        // Creating instance of server
        let server: TcpServer = TcpServer::new(self.ip_address, self.port);
        println!("Node will listen at: {}", server.listener.local_addr().unwrap());

        // Creating thread-pool for the server to handle incoming connections
        // TODO: Set the optimal size of the thread-pool automatically
        // https://engineering.zalando.com/posts/2019/04/how-to-set-an-ideal-thread-pool-size.html
        let pool = ThreadPool::new(4);

        // --- STARTUP SEQUENCE THREAD ---
        // Create a thread and multicast call to discover other nodes on the LAN
        let server_addr = server.listener.local_addr().unwrap();
        let server_addr_cp = (&server.listener.local_addr().unwrap()).clone();
        let known_hosts_for_startup_thread = Arc::clone(&self.known_hosts);
        thread::Builder::new().name("startup_sequence".to_string()).spawn(move || { // this probably doesn't need to be in a thread cause i need to wait for a response before i can work with the data layer anyways
            discover::run(&server_addr, |s| {
                let known_hosts = Arc::clone(&known_hosts_for_startup_thread);
                handle_introduction(s, known_hosts, server_addr_cp);
            }).unwrap();
        });

        // --- CLIENT THREAD ---
        let node_cp = self.clone();
        thread::Builder::new().name("client_thread".to_string()).spawn(move || {
            (client_behaviour)(node_cp);
        });

        // --- SERVER THREAD (runs on the main thread) ---
        // Listens for incoming connections on the main thread and handles connections in a
        // thread-pool
        for stream in server.listener.incoming() {
            let stream = stream.unwrap();
            let node_cp = self.clone();
            pool.execute(move || {
                let remote_addr = stream.peer_addr().unwrap();
                println!("\tNew connection from: {}", remote_addr);
                handle_incoming_connection(stream, server_behaviour, node_cp);
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

fn handle_incoming_connection(stream: TcpStream, server_behaviour: fn(Node, String) -> String, node: Node) {
    // Create an instance of the codec and pass the stream to it
    let mut codec = LineCodec::new(stream).unwrap();

    // Get the data from the incoming stream
    let data = codec.read_message().unwrap();

    let mut reply = String::new();

    if data.chars().nth(0).unwrap() == '/' {
        // This is a package has a URI

        // Separate the URI from the payload of the data packet
        let uri = data.split_whitespace().nth(0).unwrap();
        let start_of_payload_index = uri.len() + 1;
        let _payload = &data[start_of_payload_index..];

        match uri {
            "/known_hosts" => {
                let known_hosts = node.known_hosts.try_lock().unwrap();
                for host in known_hosts.iter() {
                    reply.push_str(host.to_string().as_str());
                    reply.push_str(",");
                }
            },
            "/let_me_introduce_myself" => {
                let remote_server_address = data.split_whitespace().nth(1).unwrap();
                println!("The other server is: {}", remote_server_address);
                let remote_server_address_sock = SocketAddr::from_str(remote_server_address).unwrap();
                let mut known_hosts = node.known_hosts.try_lock().unwrap();
                if !known_hosts.contains(&remote_server_address_sock) {
                    known_hosts.push(remote_server_address_sock); // this is pushing the socker address of generated clients not of the listener
                }
            },
            _ => {
                println!("Got an unknown URI from peer")
            }
        }
    } else {
        reply = server_behaviour(node, data);
    }

    // Send response back to the client
    codec.send_message(reply.as_str()).unwrap();
}