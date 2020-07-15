use laminar::{Packet, Socket, SocketEvent};
use std::net::SocketAddr;
use std::time::Instant;

#[derive(Debug)]
/// Handles sending and receiving packets
pub struct NetworkHandler {
    /// Listens and sends packets
    socket: Socket,

    /// Remote address to send packets
    remote_addr: SocketAddr,
}

impl NetworkHandler {
    /// Creates a new [NetworkHandler] where client_addr is the address the client will send from
    /// and server_addr is the address the server will listen on
    fn new(server_addr: SocketAddr, remote_addr: SocketAddr) -> Self {
        let socket = Socket::bind(server_addr).unwrap();
        NetworkHandler {
            socket,
            remote_addr,
        }
    }

    fn get_messages(&mut self) {
        self.socket.manual_poll(Instant::now());
        // let mut messages: Vec::new();
        while let Some(event) = self.socket.recv() {
            match event {
                SocketEvent::Packet(packet) => println!("packet: {:#?}", packet),
                SocketEvent::Connect(addr) => println!("Connect: {:#?}", addr),
                SocketEvent::Timeout(addr) => println!("Timeout: {:#?}", addr),
            }
        }
    }

    fn send_msg(&mut self, msg: &str) {
        // TODO: allow multiple msgs/toggle poll since it will send multiple
        let payload = msg.as_bytes().to_vec();
        let packet = Packet::unreliable(self.remote_addr, payload);
        match self.socket.send(packet) {
            Ok(()) => self.socket.manual_poll(Instant::now()),
            Err(e) => println!("error on send: {:#?}", e),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SERVER_ADDR: &str = "127.0.0.1:12345";
    const REMOTE_ADDR: &str = "127.0.0.1:12346";

    fn remote_address() -> SocketAddr {
        REMOTE_ADDR.parse().unwrap()
    }

    fn server_address() -> SocketAddr {
        SERVER_ADDR.parse().unwrap()
    }

    #[test]
    fn it_works() {
        let mut local = NetworkHandler::new(server_address(), remote_address());
        let mut remote = NetworkHandler::new(remote_address(), server_address());
        local.send_msg("hi");
        remote.get_messages();
    }
}