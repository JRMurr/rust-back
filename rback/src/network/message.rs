use crate::GameInput;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    Input(Bytes),
}

impl NetworkMessage {
    pub fn make_input<T: GameInput>(input: &T) -> NetworkMessage {
        NetworkMessage::Input(Bytes::copy_from_slice(input.to_bytes()))
    }
}
