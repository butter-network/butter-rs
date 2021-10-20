use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};

pub struct Client {
    // pub server_ip_address: IpAddr,
    // pub server_port: u16,
    pub listener: TcpListener,
}

impl Client {
    pub fn new(ip_address: IpAddr, port: u16) -> Client {
        Client {

        }
    }

    pub fn request() {

    }
}