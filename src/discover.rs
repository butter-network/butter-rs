use socket2::{Domain, Socket, Type};
use std::convert::TryInto;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
use std::thread;
use autodiscover_rs;

fn handle_broadcast_message<F: Fn(std::io::Result<TcpStream>)>(
    socket: UdpSocket,
    my_socket: &SocketAddr,
    callback: &F,
) -> std::io::Result<()> {
    let mut buff = vec![0; 18];
    loop {
        let (bytes, _) = socket.recv_from(&mut buff)?;
        if let Ok(socket) = parse_bytes(bytes, &buff) {
            if socket == *my_socket {
                // trace!("saw connection attempt from myself, this should happen once");
                continue;
            }
            let stream = TcpStream::connect(socket);
            callback(stream);
        }
    }
}

fn parse_bytes(len: usize, buff: &[u8]) -> Result<SocketAddr, ()> {
    let addr = match len {
        6 => {
            let ip = IpAddr::V4(u32::from_be_bytes(buff[0..4].try_into().unwrap()).into());
            let port = u16::from_be_bytes(buff[4..6].try_into().unwrap());
            SocketAddr::new(ip, port)
        }
        18 => {
            let ip: [u8; 16] = buff[0..16].try_into().unwrap();
            let ip = ip.into();
            let port = u16::from_be_bytes(buff[16..18].try_into().unwrap());
            SocketAddr::new(ip, port)
        }
        _ => {
            // warn!("Dropping malformed packet; length was {}", len);
            return Err(());
        }
    };
    Ok(addr)
}

fn to_bytes(connect_to: &SocketAddr) -> Vec<u8> {
    match connect_to {
        SocketAddr::V6(addr) => {
            // length is 16 bytes + 2 bytes
            let mut buff = vec![0; 18];
            buff[0..16].clone_from_slice(&addr.ip().octets());
            buff[16..18].clone_from_slice(&addr.port().to_be_bytes());
            buff
        }
        SocketAddr::V4(addr) => {
            // length is 4 bytes + 2 bytes
            let mut buff = vec![0; 6];
            buff[0..4].clone_from_slice(&addr.ip().octets());
            buff[4..6].clone_from_slice(&addr.port().to_be_bytes());
            buff
        }
    }
}