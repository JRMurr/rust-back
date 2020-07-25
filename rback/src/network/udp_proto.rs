use crate::network::udp::NetworkHandler;
use std::net::SocketAddr;
pub struct UdpProtocol {
    udp: NetworkHandler,
}

impl UdpProtocol {
    pub fn new(server_addr: SocketAddr, remote_addr: SocketAddr) -> Self {
        Self {
            udp: NetworkHandler::new(server_addr, remote_addr),
        }
    }
}
