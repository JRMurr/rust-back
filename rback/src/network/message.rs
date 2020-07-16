use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    Input(String),
}

impl NetworkMessage {
    pub fn make_input(input: &str) -> NetworkMessage {
        NetworkMessage::Input(String::from(input))
    }
}
