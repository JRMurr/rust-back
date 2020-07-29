use super::states::{self, State};
use crate::{
    error::BackendError,
    events::Event,
    network::{
        udp::NetworkHandler,
        udp_proto::{UdpEvent, UdpProtocol},
    },
    sync::Sync,
    FrameSize, GameInput, Player, PlayerType, RollbackState, SaveFrame,
};
use either;
use std::{collections::HashMap, net::SocketAddr};
// TODO: probably will extract some of this into a trait when i add spectator
#[derive(Debug)]
pub struct Peer2PeerBackend<T: GameInput, S: State> {
    pub(crate) sync: Sync<T>,
    pub(crate) num_players: u8,
    pub(crate) state: S,
    pub(crate) net_handler: NetworkHandler<T>,
    // TODO: its easy for now but check if splitting spectators out helps performance
    /// Map of player (spectator or remote) to the ud handler for them
    pub(crate) players: HashMap<Player, UdpProtocol<T>>,
}

// start in the setup state
impl<T: GameInput> Peer2PeerBackend<T, states::Setup> {
    pub fn new(server_addr: SocketAddr, max_prediction_frames: FrameSize, num_players: u8) -> Self {
        Self {
            sync: Sync::new(max_prediction_frames),
            net_handler: NetworkHandler::new(server_addr),
            players: HashMap::with_capacity(num_players as usize),
            num_players,
            state: states::Setup,
        }
    }

    fn add_spectator(&mut self, addr: SocketAddr) -> Result<(), BackendError> {
        // TODO: might be able just use add_remote
        todo!()
    }

    fn add_remote(&mut self, remote_addr: SocketAddr, player: Player) -> Result<(), BackendError> {
        let udp_proto = UdpProtocol::new(remote_addr, self.net_handler.get_sender());
        self.players.insert(player, udp_proto);
        // TODO: check player count
        // TODO: transition out of setup when all players added?
        Ok(())
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), BackendError> {
        // TODO: check player num for local and remote
        match player.player_type {
            PlayerType::Local => Ok(()),
            PlayerType::Spectator(addr) => self.add_spectator(addr),
            PlayerType::Remote(addr) => self.add_remote(addr, player),
        }
    }
}

// impl<T: GameInput> Peer2PeerBackend<T, states::PollNetwork> {
//     // GGPO PollUdpProtocolEvents
//     pub fn poll(&mut self) {}
// }

impl<T: GameInput> Peer2PeerBackend<T, states::InRollback> {
    // TODO: if i could share this btwn Normal and InRollback that would be lit
    pub fn sync_inputs(&mut self) -> Result<Vec<Option<T>>, BackendError> {
        // TODO: handle  player disconnect
        self.sync.synchronize_inputs().map_err(BackendError::from)
    }

    pub fn increment_frame(&mut self) -> SaveFrame {
        self.sync.increment_frame()
    }
}
// type IncrRes = (Peer2PeerBackend<T, State>, Option<RollbackState>);
impl<T: GameInput> Peer2PeerBackend<T, states::Normal> {
    pub fn sync_inputs(&mut self) -> Result<Vec<Option<T>>, BackendError> {
        // TODO: handle  player disconnect
        self.sync.synchronize_inputs().map_err(BackendError::from)
    }

    fn into_rollback(self, state: states::InRollback) -> Peer2PeerBackend<T, states::InRollback> {
        Peer2PeerBackend {
            sync: self.sync,
            net_handler: self.net_handler,
            players: self.players,
            num_players: self.num_players,
            state,
        }
    }

    pub fn increment_frame(
        mut self,
    ) -> Result<
        (
            either::Either<
                Peer2PeerBackend<T, states::InRollback>,
                Peer2PeerBackend<T, states::PostRollback>,
            >,
            SaveFrame,
            Vec<Event>,
        ),
        BackendError,
    > {
        let saved_frame = self.sync.increment_frame();
        self.net_handler.empty_msg_queue();
        let events = self.poll_udp_protocol_events()?;
        match self.sync.check_simulation()? {
            Some(state) => {
                let state = states::InRollback { load_frame: state };
                Ok((either::Left(self.into_rollback(state)), saved_frame, events))
            }
            None => Ok((either::Right(self.into()), saved_frame, events)),
        }
    }

    fn on_udp_protocol_event(&self, event: &UdpEvent<T>) -> Option<Event> {
        todo!()
    }

    fn poll_udp_protocol_events(&mut self) -> Result<Vec<Event>, BackendError> {
        let mut events: Vec<UdpEvent<T>> = vec![];
        for (player, udp_proto) in self.players.iter_mut() {
            match player.player_type {
                PlayerType::Remote(_) => {
                    // need to collect to avoid doing two mutable borrows on udp_proto
                    let udp_events: Vec<UdpEvent<T>> = udp_proto.get_events().collect();
                    for event in udp_events {
                        // TODO: removing this clone would be ideal
                        match event.clone() {
                            UdpEvent::Input(input) => {
                                if !udp_proto.is_disconnected() {
                                    let new_remote_frame = input.frame;
                                    udp_proto.set_frame(new_remote_frame);
                                    self.sync.add_remote_input(player.player_number, input)?;
                                }
                            }
                            UdpEvent::Disconnected => todo!(),
                            _ => {}
                        }
                        events.push(event);
                    }
                }
                PlayerType::Spectator(_) => todo!(),
                _ => panic!("Should not have local players in this map"),
            }
        }
        Ok(events
            .iter()
            .filter_map(|e| self.on_udp_protocol_event(e))
            .collect())
    }
}

// impl<T: GameInput, S: State> Peer2PeerBackend<T, S> {
//     pub fn do_poll_pre_rollback(&mut self) -> Result<Option<RollbackState>,
// BackendError> {         if self.sync.in_rollback() {
//             return Ok(None);
//         }
//         self.poll_udp();
//         // if !self.synchronizing {
//         //     return Ok(None);
//         // }

//         match self.sync.check_simulation()? {
//             None => {
//                 // TODO: might be better to just end here instead of call
// poll_post                 self.poll_post_rollback()?;
//                 Ok(None)
//             }
//             Some(res) => Ok(Some(res)),
//         }
//     }

//     pub fn poll_post_rollback(&mut self) -> Result<(), BackendError> {
//         todo!()
//     }

//     fn poll_udp(&mut self) {}
// }
