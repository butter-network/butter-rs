// Implementation modified from the following repository:
// https://github.com/over-codes/autodiscover-rs

// use log::{trace, warn};
use socket2::{Domain, Socket, Type};
use std::convert::TryInto;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};

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

/// run will block forever. It sends a notification using the configured method, then listens for other notifications and begins
/// connecting to them, calling spawn_callback (which should return right away!) with the connected streams. The connect_to address
/// should be a socket we have already bind'ed too, since we advertise that to other autodiscovery clients.
pub fn run<F: Fn(std::io::Result<TcpStream>)>(
    connect_to: &SocketAddr,
    spawn_callback: F,
) -> std::io::Result<()> {
    // let ip: Ipv6Addr = Ipv6Addr::from_str("[ff0e::1]").unwrap();
    // let port: u16 = 1337;
    let addr: SocketAddr = "[ff0e::1]:1337".parse().unwrap();
    // let addr: SocketAddr = SocketAddr:
    let socket = Socket::new(Domain::ipv6(), Type::dgram(), None)?;
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    let socket: UdpSocket = socket.into_udp_socket();
    // TODO: Remove the IPV4 option for autodiscover as this is only on LAN and we will always have IPV6 available
    // (IPV6 has limited availability over the internet)
    match addr.ip() {
        IpAddr::V4(addr) => {
            let iface: Ipv4Addr = 0u32.into();
            socket.join_multicast_v4(&addr, &iface)?;
        }
        IpAddr::V6(addr) => {
            socket.join_multicast_v6(&addr, 0)?;
        }
    }
    // we need a different, temporary socket, to send multicast in IPv6
    {
        let socket = UdpSocket::bind(":::0")?;
        let _result = socket.send_to(&to_bytes(connect_to), addr)?;
        // trace!("sent {} byte announcement to {:?}", result, addr);
    }
    handle_broadcast_message(socket, connect_to, &spawn_callback)?;
    // warn!("It looks like I stopped listening; this shouldn't happen.");
    Ok(())
}