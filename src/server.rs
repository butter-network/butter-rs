use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream, UdpSocket, Ipv4Addr};
use std::io::Error;

static MULTI_CAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 1);

// The Server struct is used as the basis for nodes that need to operate as servers.
// Generically, the server connects to the network and listens on a given port.

pub struct TCPServer {
    pub listener: TcpListener,
    pub routes: HashMap<String, fn(TcpStream) -> ()>,
}

impl TCPServer {
    pub fn new(ip_address: IpAddr, port: u16) -> TCPServer {
        let socket_address = SocketAddr::new(ip_address, port);
        let listener = TcpListener::bind(socket_address).unwrap();

        let routes: HashMap<String, fn(TcpStream) -> ()> = HashMap::new();

        println!("Server is listening...");

        TCPServer { listener, routes }
    }

    pub fn register_routes(&mut self, path: String, route_behaviour: fn(TcpStream) -> ()) {
        self.routes.insert(path, route_behaviour);
    }
}