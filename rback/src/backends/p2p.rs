use crate::{error::BackendError, sync::Sync, FrameSize, GameInput, Player, PlayerType};
use std::net::SocketAddr;

// TODO: probably will extract some of this into a trait when i add spectator
#[derive(Debug)]
pub struct Peer2PeerBackend<T: GameInput> {
    sync: Sync<T>,
    num_players: u8,
}

impl<T: GameInput> Peer2PeerBackend<T> {
    pub fn new(max_prediction_frames: FrameSize, num_players: u8) -> Self {
        Self {
            sync: Sync::new(max_prediction_frames),
            num_players,
        }
    }

    fn add_spectator(&mut self, addr: SocketAddr) -> Result<(), BackendError> {
        todo!()
    }

    fn add_remote(&mut self, addr: SocketAddr) -> Result<(), BackendError> {
        todo!()
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), BackendError> {
        // TODO: check player num for local and remote
        match player.player_type {
            PlayerType::Local => Ok(()),
            PlayerType::Spectator(addr) => self.add_spectator(addr),
            PlayerType::Remote(addr) => self.add_remote(addr),
        }
    }

    pub fn do_poll(&mut self) {}
}
