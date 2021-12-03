// Ref: https://github.com/over-codes/autodiscover-rs

// use log::{trace, warn};
use socket2::{Domain, Socket, Type};
use std::convert::TryInto;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

//TODO: modify handle_broadcast_message to not last forever - just loop until you have found a peer - then stop thread

// Seems to be the solution to the blocking problem https://play.rust-lang.org/?gist=a660f98aa692072160700b74d2f2e264&version=stable
fn handle_broadcast_message<F: Fn(std::io::Result<TcpStream>)>(
    socket: UdpSocket,
    my_socket: &SocketAddr,
    callback: &F,
    state: Arc<AtomicUsize>,
) -> std::io::Result<()> {
    let mut buff = vec![0; 18];
    socket.set_nonblocking(true);
    loop {
        // println!("{}", state.load(Ordering::Relaxed));
        // if state.load(Ordering::Relaxed) == 1 {
        //     break;
        // }
        println!("test");
        let (bytes, _) = socket.recv_from(&mut buff)?; // this waits till it receives a datagram - it blocks
        if let Ok(socket) = parse_bytes(bytes, &buff) {
            if socket == *my_socket {
                // trace!("saw connection attempt from myself, this should happen once");
                continue;
            }
            let stream = TcpStream::connect(socket);
            callback(stream);
            break; // if we get here, we have connected to a peer, we can break out of loop
        }
    }
    Ok(())
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
    state: Arc<AtomicUsize>
) -> std::io::Result<()> {
    let addr: SocketAddr = "[ff0e::1]:1337".parse().unwrap();
    let socket = Socket::new(Domain::ipv6(), Type::dgram(), None)?;
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    let socket: UdpSocket = socket.into_udp_socket();
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
        let result = socket.send_to(&to_bytes(connect_to), addr)?;
        // trace!("sent {} byte announcement to {:?}", result, addr);
    }
    handle_broadcast_message(socket, connect_to, &spawn_callback, state);
    // warn!("It looks like I stopped listening; this shouldn't happen.");
    Ok(())
}