use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    Input(String),
}

impl NetworkMessage {
    pub fn make_input(input: &str) -> NetworkMessage {
        NetworkMessage::Input(String::from(input))
    }
}
