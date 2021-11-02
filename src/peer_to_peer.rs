use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::thread;
use std::time::Duration;
use crate::server::Server;
use crate::threadpool::ThreadPool;

use lazy_static::lazy_static;

use std::sync::{Mutex};
use crate::line_codec::LineCodec;

lazy_static! {
    static ref KNOWN_HOSTS: Mutex<Vec<IpAddr>> = Mutex::new(Vec::new());
}

fn get_known_hosts(stream: TcpStream) {
    let mut codec = LineCodec::new(stream).unwrap();
    // And use the codec to return it
    codec.send_message("this is the list of known hosts").unwrap();
}

pub struct PeerToPeer {
    ip_address: IpAddr,
    port: u16,
    // server: Server,
}

impl PeerToPeer {
    pub fn new(ip_address: IpAddr, port: u16, server_behaviour: fn(TcpStream) -> (),
               client_behaviour: fn(&Mutex<Vec<IpAddr>>) -> ()) -> PeerToPeer {

        // known_hosts - add itself?
        let entrypoint = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        KNOWN_HOSTS.lock().unwrap().push(entrypoint);

        // TODO: Switch the client to the thread and the server to the main thread
        
        thread::spawn(move || {
            let mut server: Server = Server::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8376);

            let pool = ThreadPool::new(4);

            server.register_routes("/get_known_hosts".parse().unwrap(), get_known_hosts);

            for stream in server.listener.incoming() {
                let stream = stream.unwrap();
                pool.execute(move || {
                    let peer_address = stream.peer_addr().unwrap().ip();
                    println!("\tNew connection from: {}", peer_address);
                    server_behaviour(stream);
                    if !KNOWN_HOSTS.lock().unwrap().contains(&peer_address) {
                        KNOWN_HOSTS.lock().unwrap().push(peer_address);
                    }
                });
            }
        });

        // Allow the server to startup before client tries to connect
        thread::sleep(Duration::from_secs(2));

        client_behaviour(&KNOWN_HOSTS);

        PeerToPeer {
            ip_address,
            port,
            // server,
        }
    }

    pub fn register_server_route(&self, route: String, behaviour: fn(TcpStream) -> ()) {
    //    self.server.register_routes(route, behaviour);
    }
}