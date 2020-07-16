use rback::network::{message::NetworkMessage, udp::NetworkHandler};
use std::net::SocketAddr;

fn main() {
    const SERVER_ADDR: &str = "127.0.0.1:12345";
    const REMOTE_ADDR: &str = "127.0.0.1:12346";

    fn remote_address() -> SocketAddr {
        REMOTE_ADDR.parse().unwrap()
    }

    fn server_address() -> SocketAddr {
        SERVER_ADDR.parse().unwrap()
    }

    let mut local = NetworkHandler::new(server_address(), remote_address());
    let mut remote = NetworkHandler::new(remote_address(), server_address());
    let payload = NetworkMessage::make_input("hello");
    local.send_msg_now(&payload).unwrap();
    remote.get_messages();
}
