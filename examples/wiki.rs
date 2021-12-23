// Decentralised encyclopedia example

use std::io::{stdin};
use std::sync::{Mutex, Arc};
use butter::peer::Node;


fn client_behaviour(node: &Node) {
    loop {
        // To search for persistent information on the network you either:
        // - 'get' a specific piece of information (dns)
        // - 'query' for a list of potentially useful information (search engine)
        println!("Would you like to search(1) or add(2) information to the network:");
        let mut interaction_type = String::new();
        stdin()
            .read_line(&mut interaction_type)
            .ok()
            .expect("Couldn't read line");

        if interaction_type == "1\n" {
            // Search for information
            println!("Would you like to get a specific piece of information by id(1) or query the network(2):");

            let mut search_type = String::new();
            let mut search_result = String::new();

            stdin()
                .read_line(&mut search_type)
                .ok()
                .expect("Couldn't read line");

            if search_type == "1\n" {
                // 'get' a specific piece of information
                println!("What is the information id:");
                let mut information_id = String::new();
                stdin()
                    .read_line(&mut information_id)
                    .ok()
                    .expect("Couldn't read line");
                // search_result.push_str(p2p.search_get(information_id));
                search_result.push_str("This would be the requested information");
            } else if search_type == "2\n" {
                // 'query' for a list of potentially useful information (search engine)
                println!("What is the query:");
                let mut query = String::new();
                stdin()
                    .read_line(&mut query)
                    .ok()
                    .expect("Couldn't read line");
                // search_result.push_str(p2p.search_query(query));
                search_result.push_str("This would be the answer to the query");
            } else {
                search_result.push_str("Invalid option");
            }
            println!("{}", search_result);
        } else if interaction_type == "2\n" {
            // Add information
            println!("This would be where you add information");
            // p2p.store_information();
        } else {
            println!("Invalid option");
        }
    }
}

fn server_behaviour(node: &Node) {
    // Either store information or help search

    // Store
    // self.store_information(message);

    // Search
    // Do I have the information in my storage?
    // self.storage
    // who can I route the request to to increase my chances of obtaining an answer
    todo!()
}


fn main() {
    let peer: Node = Node::new(None);
    peer.start(client_behaviour, server_behaviour);
}