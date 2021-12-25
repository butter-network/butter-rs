use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::{mem, thread};
use std::collections::HashMap;
use serde_json::Value;
use sysinfo::{System, SystemExt};
use uuid::Uuid;

use crate::threadpool::ThreadPool;
use crate::server::TcpServer;
use crate::codec::LineCodec;
use crate::discover;
use crate::utils;

#[derive(Debug, Clone)]
pub struct Node {
    ip_address: IpAddr,
    port: u16,
    max_known_hosts: usize,
    pub known_hosts: Arc<Mutex<Vec<SocketAddr>>>,
    storage: HashMap<String, String>, // uuid, data
    // some form of storage data structure? hashmap<key:UUID, value: Data> or use JSON as the a sort of key value store
}

// The node needs to be lightweight as it often gets cloned

impl Node {
    pub fn new(port: Option<u16>) -> Node {

        // Get the local ip address of the machine
        let ip_address = utils::get_local_ip().unwrap();

        // Get the user defined port
        // If port is unspecified, the port is set to zero and allocated by the OS when the node is
        // started
        let port = port.unwrap_or(0);

        // Determining appropriate known hosts list size upper limit based on system memory
        let mut sys = System::new();
        sys.refresh_all();
        let max_known_hosts = sys.total_memory() as usize * 8/100000 / mem::size_of::<SocketAddr>();

        // Create the list of known hosts for the node
        let known_hosts = Arc::new(Mutex::new(Vec::new()));

        // Create storage for the node
        let storage = HashMap::new();

        if port == 0 {
            println!("Unspecified port, will use port allocated by OS");
            println!("Creating node with \n- ip address: {} \n- port: UNSPECIFIED", ip_address);
        } else {
            println!("Creating node with \nip address: {} \nport: {}", ip_address, port);
        }

        Node {
            ip_address,
            port,
            max_known_hosts,
            known_hosts,
            storage,
        }
    }

    pub fn start(self, server_behaviour: fn(Node, String) -> String,
                 client_behaviour: fn(Node) -> ()) {

        // Creating instance of server
        let server: TcpServer = TcpServer::new(self.ip_address, self.port);
        println!("Node will listen at: {}", server.listener.local_addr().unwrap());

        // Creating thread-pool for the server to handle incoming connections
        // Set the optimal size of the thread-pool automatically
        // https://stackoverflow.com/questions/47033696/how-to-decide-the-number-of-thread-in-a-thread-pool
        let mut sys = System::new();
        sys.refresh_all();
        let nb_core = sys.processors().len();
        let cpu_sticky = 2;
        // Formula for calculating optimal thread-pool size: number_of_cores*(1/cpu_sticky)
        // Where cpu_sticky is between 0 and 1. So if you perceive it to be highly IO, cpu_sticky
        // is very low (say .1), whereas a highly CPU intensive have a high cpu_sticky (say .9).
        let optimal_pool_size = nb_core*1/cpu_sticky;
        println!("{}", optimal_pool_size);
        let pool = ThreadPool::new(optimal_pool_size);

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
    pub fn naive_store(mut self, data: String) -> String {
        // Generate UUID for the data
        let uuid = Uuid::new_v4()?;
        // just store on this node (a more intelligent approach would place the data on the best node)
        self.storage.insert(uuid.to_string(), data)?
    }

    // High level entrypoint for searching for a specific piece of information on the network
    // this needs to be recursive i.e. look if I have the information else look at the most likely known host to get to that information
    // one query per piece of information (one-to-one) hence the query has to be unique i.e i.d.
    pub fn naive_retrieve(self, query: String) -> String {
        // do I have this information, if so return it
        // else BFS (pass the query on to all known hosts (partial view)
        return if self.storage.contains_key(&query) {
            let mut packet = String::new();
            packet.push_str("/success ");
            packet.push_str(self.storage.get(&query).unwrap());
            packet
        } else {
            // BFS
            let mut known_hosts = self.known_hosts.lock().unwrap();
            let mut next_hosts = Vec::new();
            for host in known_hosts.iter() {
                next_hosts.push(host.clone());
            }
            while next_hosts.len() > 0 {
                let host = next_hosts.pop().unwrap();
                // create a tcp codec asking them to return themselves (i.e. the node)
                let stream = TcpStream::connect(host).unwrap();
                let mut codec = LineCodec::new(stream).unwrap();
                // just get the remote node's storage and check if it contains the information
                codec.send_message("/get-storage");
                let remote_storage = codec.read_message().unwrap();
                let remote_storage_json: Value = serde_json::from_str(&remote_storage).unwrap();
                // If it does "Yay!" we finish our search!
                if remote_storage_json.contains(query) {
                    return remote_storage_json[query].to_string();
                }
                // else get all the known hosts of the remote host and add them to the queue
                codec.send_message("/get-known-hosts");
                let remote_known_hosts = codec.read_message().unwrap();
                let remote_known_hosts_json: Value = serde_json::from_str(&remote_known_hosts).unwrap();
                for host in remote_known_hosts_json.iter() {
                    next_hosts.push(host.clone());
                }
            }
            "Does not exists".to_string()
        }
    }

    // search for information omn the network as a search engine (more fuzzy) one-to-many relationship between the query and the information
    fn explore() {}

    // removes dead known hosts - runs periodically
    fn clean() {}

    fn add_host(self, host: SocketAddr) {
        let mut lock = self.known_hosts.try_lock().unwrap();
        if lock.len() < self.max_known_hosts && !lock.contains(&host) {
            lock.push(host);
        }
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
                // let mut known_hosts = node.known_hosts.try_lock().unwrap();
                // if !known_hosts.contains(&remote_server_address_sock) {
                //     known_hosts.push(remote_server_address_sock); // this is pushing the socker address of generated clients not of the listener
                // }
                node.add_host(remote_server_address_sock);
            },
            "/remote-retrieve" => {
                reply = node.naive_retrieve(_payload.to_string());
            },
            "/get-storage" => {
                let storage = node.storage;
                // serialise the hashmap into JSON
                // send the JSON string as the reply
                reply = serde_json::to_string(&storage).unwrap();
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

/// Separates the URI from the payload of a specified data packet
/// # Returns
/// A tuple of the URI and the payload
fn separate_uri_payload(packet: String) -> (String, String) {
    let uri = packet.split_whitespace().nth(0).unwrap().to_string();
    let start_of_payload_index = uri.len() + 1;
    let payload: String = packet[start_of_payload_index..].to_string();
    (uri, payload)
}

// returns the best peer to forward the query to
// fn peer_quality(node: Node, query: String) -> Node {
//
// }