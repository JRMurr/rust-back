use crate::{
    network::{message::NetworkMessage, udp::NetworkHandler},
    GameInput,
};
use laminar::ErrorKind;
use std::{marker::PhantomData, net::SocketAddr};
pub struct UdpProtocol<T: GameInput> {
    remote_addr: SocketAddr,
    game_input: PhantomData<T>,
}

impl<T: GameInput> UdpProtocol<T> {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            remote_addr: server_addr,
            game_input: PhantomData,
        }
    }

    pub fn send_msg(
        &self,
        udp: &mut NetworkHandler<T>,
        payload: &NetworkMessage,
    ) -> Result<(), ErrorKind> {
        udp.send_msg_now(payload, &self.remote_addr)
    }
}
