pub mod p2p;
use crate::{sync::Sync, FrameSize, GameInput};
// TODO: probably will extract some of this into a trait when i add spectator

pub struct Peer2PeerBackend<T: GameInput> {
    sync: Sync<T>,
}

impl<T: GameInput> Peer2PeerBackend<T> {
    pub fn new(max_prediction_frames: FrameSize) -> Self {
        Self {
            sync: Sync::new(max_prediction_frames),
        }
    }
}
