use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::thread;
use std::time::Duration;
use crate::server::Server;
use crate::threadpool::ThreadPool;

use lazy_static::lazy_static;
// use crate::line_codec::LineCodec;

use std::sync::Mutex;

lazy_static! {
    static ref KNOWN_HOSTS: Mutex<Vec<IpAddr>> = Mutex::new(Vec::new());
}

pub struct PeerToPeer {
    // server: Server,
    // known_hosts: KNOWN_HOSTS,
    ip_address: IpAddr,
    port: u16,
}

impl PeerToPeer {
    pub fn new(ip_address: IpAddr, port: u16, server_behaviour: fn(TcpStream) -> (),
               client_behaviour: fn() -> ()) -> PeerToPeer {

        // known_hosts - add itself?
        let entrypoint = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        KNOWN_HOSTS.lock().unwrap().push(entrypoint);

        thread::spawn(move || {
            let server: Server = Server::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8376);
            // let pool = ThreadPool::new(4);
            for stream in server.listener.incoming() {
                let stream = stream.unwrap();
                // pool.execute(|| {
                //     let peer_address = stream.peer_addr().unwrap().ip();
                //     println!("\tNew connection from: {}", peer_address);
                //     server_behaviour(stream);
                //     if !KNOWN_HOSTS.lock().unwrap().contains(&peer_address) {
                //         KNOWN_HOSTS.lock().unwrap().push(peer_address);
                //     }
                // });
                let peer_address = stream.peer_addr().unwrap().ip();
                println!("\tNew connection from: {}", peer_address);
                server_behaviour(stream);
                if !KNOWN_HOSTS.lock().unwrap().contains(&peer_address) {
                    KNOWN_HOSTS.lock().unwrap().push(peer_address);
                }
            }
        });

        // Allow the server to startup - what about just putting the server on the main thread?
        thread::sleep(Duration::from_secs(2));

        client_behaviour();

        PeerToPeer {
            ip_address,
            port
        }
    }
}