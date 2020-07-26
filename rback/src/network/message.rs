use crate::GameInput;
use serde::{Deserialize, Serialize};

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
