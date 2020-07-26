use crate::{network::message::NetworkMessage, GameInput};
use std::marker::PhantomData;

use bincode::{deserialize, serialize};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use std::{net::SocketAddr, time::Instant, vec::Vec};

/// Handles sending and receiving packets
pub struct NetworkHandler<T: GameInput> {
    /// Listens and sends packets on this [Socket]
    socket: Socket,
    /* TODO: can probs get rid of this and make each func define the
     * generic */
    game_input: PhantomData<T>,
}

impl<T: GameInput> NetworkHandler<T> {
    /// Creates a new [NetworkHandler] where server_addr is the address the
    /// server will listen on
    pub fn new(server_addr: SocketAddr) -> Self {
        let socket = Socket::bind(server_addr).unwrap();
        NetworkHandler {
            socket,
            game_input: PhantomData,
        }
    }

    pub fn get_messages(&mut self) -> Vec<NetworkMessage<T>> {
        self.socket.manual_poll(Instant::now());
        let mut messages = Vec::new();
        while let Some(event) = self.socket.recv() {
            match event {
                SocketEvent::Packet(packet) => {
                    let msg: NetworkMessage<T> = deserialize(packet.payload()).unwrap();
                    println!("message: {:#?}", msg);
                    messages.push(msg);
                }
                SocketEvent::Connect(addr) => println!("Connect: {:#?}", addr),
                SocketEvent::Timeout(addr) => println!("Timeout: {:#?}", addr),
            }
        }
        messages
    }

    pub fn send_msg_now(
        &mut self,
        payload: &NetworkMessage<T>,
        remote_addr: &SocketAddr,
    ) -> Result<(), ErrorKind> {
        self.queue_msg(payload, remote_addr)?;
        self.empty_msg_queue();
        Ok(())
    }

    pub fn queue_msg(
        &mut self,
        payload: &NetworkMessage<T>,
        remote_addr: &SocketAddr,
    ) -> Result<(), ErrorKind> {
        let packet = Packet::reliable_unordered(*remote_addr, serialize(payload).unwrap());
        self.socket.send(packet)
    }

    pub fn empty_msg_queue(&mut self) {
        self.socket.manual_poll(Instant::now())
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
    fn queue_and_send_messages() {
        let mut local: NetworkHandler<String> = NetworkHandler::new(server_address());
        let mut remote: NetworkHandler<String> = NetworkHandler::new(remote_address());
        let payload1 = NetworkMessage::make_input(&"msg1".into());
        let payload2 = NetworkMessage::make_input(&"msg2".into());
        local.queue_msg(&payload1, &remote_address()).unwrap();
        local.queue_msg(&payload2, &remote_address()).unwrap();

        // queue has not been emptied yet so no messages sent
        assert_eq!(remote.get_messages(), vec![]);

        local.empty_msg_queue();
        assert_eq!(remote.get_messages(), vec![payload1, payload2])
    }
}
