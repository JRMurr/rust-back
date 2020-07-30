use super::p2p::Peer2PeerBackend;
use crate::{GameInput, RollbackState};
// https://hoverbear.org/blog/rust-state-machine-pattern/#worked-examples

// TODO: REWRITE:
// Since From only works on owned values this would not work if any thing
// holding ggpo was behind a mut ref. I should probably just dump this idea and
// assume people using it will do the right thing based on returned results.
// A good middle ground would to return a state "token" that would only let them
// call the right methods if the token is in the right state. This isnt perfect
// since it would add an extra param to all funcs and since theres no func
// overloading it would look a little gross
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
// state!(InRollback);
state!(PostRollback);
state!(Normal);
// make in roll back manually to add a field
pub struct InRollback {
    pub load_frame: RollbackState,
}
impl State for InRollback {}

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
impl<T: GameInput> From<Peer2PeerBackend<T, Setup>> for Peer2PeerBackend<T, Normal> {
    fn from(other: Peer2PeerBackend<T, Setup>) -> Self {
        Self {
            sync: other.sync,
            net_handler: other.net_handler,
            players: other.players,
            num_players: other.num_players,
            state: Normal,
        }
    }
}

// transition!(Setup, Normal);
// transition!(Normal, PollNetwork);
// transition!(PollNetwork, Normal);
// transition!(PollNetwork, InRollback);

// transition!(Normal, InRollback);
transition!(InRollback, PostRollback);
transition!(PostRollback, Normal);
transition!(Normal, PostRollback);
