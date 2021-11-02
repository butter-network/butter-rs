use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};

/*
 The Server struct is used as the basis for nodes that need to operate as servers.
 Generically, the server connects to the network and listens on a given port.
*/

pub struct Server {
    pub listener: TcpListener,
    pub routes: HashMap<String, fn(TcpStream) -> ()>,
}

impl Server {
    pub fn new(ip_address: IpAddr, port: u16) -> Server {
        let socket_address = SocketAddr::new(ip_address, port);
        let listener = TcpListener::bind(socket_address).unwrap();

        let routes: HashMap<String, fn(TcpStream) -> ()> = HashMap::new();

        // // Handling bind error.
        // match listener {
        //     Ok(listener) => {
        //         Server {
        //             ip_address,
        //             port,
        //             listener,
        //         }
        //     }
        //     Err(e) => {
        //         println!("Could not bind to socket.");
        //         println!("Error: {}", e);
        //     }
        // }

        println!("Server is listening...");
        Server { listener, routes }
    }

    pub fn register_routes(&mut self, path: String, route_behaviour: fn(TcpStream) -> ()) {
        self.routes.insert(path, route_behaviour);
    }
}
