use std::net::{IpAddr, SocketAddr, TcpListener};

/*
 The Server struct is used as the basis for nodes that need to operate as servers.
 Generically, the server connects to the network and listens on a given port.
*/

pub struct Server {
    pub listener: TcpListener,
}

impl Server {
    pub fn new(ip_address: IpAddr, port: u16) -> Server {
        let socket_address = SocketAddr::new(ip_address, port);
        let listener = TcpListener::bind(socket_address).unwrap();

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
        Server {
            listener,
        }
    }
}