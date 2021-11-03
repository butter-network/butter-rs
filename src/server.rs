use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};


// The Server struct is used as the basis for nodes that need to operate as servers.
// Generically, the server connects to the network and listens on a given port.

pub struct Server {
    pub listener: TcpListener,
    pub routes: HashMap<String, fn(String) -> String>,
}

impl Server {
    pub fn new(ip_address: IpAddr, port: u16) -> Server {
        let socket_address = SocketAddr::new(ip_address, port);
        let listener = TcpListener::bind(socket_address).unwrap();

        let routes: HashMap<String, fn(String) -> String> = HashMap::new();

        println!("Server is listening...");

        Server { listener, routes }
    }

    pub fn register_routes(&mut self, path: String, route_behaviour: fn(String) -> String) {
        self.routes.insert(path, route_behaviour);
    }
}
