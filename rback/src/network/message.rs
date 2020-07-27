use crate::{FrameIndex, GameInput};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    LastFrame(FrameIndex),
}

// TODO: ggpo has a `magic_num` param to filter messages not from the current
// session can probably add a timestamp to do the same thing
// check if laminar already does this, probably has something that would
// accomplish this
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage<T: GameInput> {
    #[serde(deserialize_with = "T::deserialize")]
    Input(T),
}

impl<T: GameInput> NetworkMessage<T> {
    pub fn make_input(input: &T) -> NetworkMessage<T> {
        NetworkMessage::Input(input.clone())
    }
}
