use std::net::IpAddr;
use crate::server::Server;


struct PeerToPeer {
    server: Server,
    known_hosts: Vec<IpAddr>,

    ip_address: IpAddr,
    port: u16,
}

impl PeerToPeer {
    fn new(ip_address: IpAddr, port: u16, server_functionality: &dyn Fn() -> (),
           client_functionality: &dyn Fn() -> ()) -> PeerToPeer {

        let server: Server = Server::new(ip_address, port);
        // known_hosts - add itself?

        PeerToPeer {
            server,
            known_hosts,
            ip_address,
            port
        }
    }
}