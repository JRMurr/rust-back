use super::p2p::Peer2PeerBackend;
use crate::GameInput;
// https://hoverbear.org/blog/rust-state-machine-pattern/#worked-examples

pub trait State {}

macro_rules! state {
    ($state:ident) => {
        #[derive(Debug)]
        pub struct $state;
        impl State for $state {}
    };
}

// Make structs for each state and mark them with the state trait
state!(Setup);
// state!(PollNetwork);
state!(InRollback);
state!(PostRollback);
state!(Normal);

macro_rules! transition {
    ($from:ident, $to:ident) => {
        impl<T: GameInput> From<Peer2PeerBackend<T, $from>> for Peer2PeerBackend<T, $to> {
            fn from(other: Peer2PeerBackend<T, $from>) -> Self {
                Self {
                    sync: other.sync,
                    net_handler: other.net_handler,
                    players: other.players,
                    num_players: other.num_players,
                    state: $to,
                }
            }
        }
    };
}

transition!(Setup, Normal);
// transition!(Normal, PollNetwork);
// transition!(PollNetwork, Normal);
// transition!(PollNetwork, InRollback);
transition!(Normal, InRollback);
transition!(InRollback, PostRollback);
transition!(PostRollback, Normal);
