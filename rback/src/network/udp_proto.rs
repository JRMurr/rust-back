use crate::{
    game_input_frame::GameInputFrame,
    network::message::{ConnectionStatus, NetworkMessage},
    FrameIndex, GameInput,
};
use bincode::serialize;
use crossbeam_channel::{SendError, Sender};
use laminar::{ErrorKind, Packet, SocketEvent};
use std::{net::SocketAddr, vec::Drain};
#[derive(Debug, Clone)]
pub enum UdpEvent<T: GameInput> {
    Input(GameInputFrame<T>),
    Connected,
    Synchronizing,
    Synchronized,
    Disconnected,
    NetworkInterrupted,
    NetworkResumed,
}

#[derive(Debug)]
pub struct UdpProtocol<T: GameInput> {
    remote_addr: SocketAddr,
    sender: Sender<Packet>,
    events: Vec<UdpEvent<T>>,
    pub(crate) status: ConnectionStatus,
}

impl<T: GameInput> UdpProtocol<T> {
    pub fn new(remote_addr: SocketAddr, sender: Sender<Packet>) -> Self {
        Self {
            sender,
            remote_addr,
            status: ConnectionStatus::LastFrame(None),
            events: vec![],
        }
    }

    pub fn set_frame(&mut self, frame: FrameIndex) {
        self.status = ConnectionStatus::LastFrame(frame);
    }

    pub fn is_disconnected(&self) -> bool {
        self.status == ConnectionStatus::Disconnected
    }

    fn send(&self, packet: Packet) -> Result<(), ErrorKind> {
        match self.sender.send(packet) {
            Ok(_) => Ok(()),
            Err(error) => Err(ErrorKind::SendError(SendError(SocketEvent::Packet(
                error.0,
            )))),
        }
    }

    pub fn send_msg(&self, payload: &NetworkMessage<T>) -> Result<(), ErrorKind> {
        // TODO: probably should use ordered
        let packet = Packet::reliable_unordered(self.remote_addr, serialize(payload).unwrap());
        self.send(packet)
    }

    pub fn on_msg(&self, msg: &NetworkMessage<T>) {
        todo!()
    }

    fn queue_event(&mut self, event: UdpEvent<T>) {
        self.events.push(event);
    }

    pub fn get_events(&mut self) -> Drain<UdpEvent<T>> {
        self.events.drain(..)
    }
}
