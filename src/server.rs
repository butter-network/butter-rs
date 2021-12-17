use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};


pub struct TcpServer {
    pub listener: TcpListener,
    pub routes: HashMap<String, fn(TcpStream) -> ()>,
}

impl TcpServer {
    pub fn new(ip_address: IpAddr, port: u16) -> TcpServer {
        let socket_address = SocketAddr::new(ip_address, port);
        let listener = TcpListener::bind(socket_address).unwrap();

        let routes: HashMap<String, fn(TcpStream) -> ()> = HashMap::new();

        TcpServer { listener, routes }
    }

    pub fn register_routes(&mut self, path: String, route_behaviour: fn(TcpStream) -> ()) {
        self.routes.insert(path, route_behaviour);
    }
}