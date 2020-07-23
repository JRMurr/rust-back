// #![warn(missing_docs)]

use std::fmt::Debug;

pub mod error;
pub(crate) mod game_input_frame;
pub mod input_queue;
pub mod network;
pub mod sync;
// With this we can keep track of about 3 years worth of frames
// at 60fps...
type FrameSize = u32;
pub(crate) type FrameIndex = Option<FrameSize>;
/// Input passed to be used in rollbacks must satisfy this trait
pub trait GameInput: Clone + Debug + PartialEq {}
impl<T> GameInput for T where T: Clone + Debug + PartialEq {}

pub trait SyncCallBacks {
    type SavedState;
    // Don't need to use frame in save/load passed for convince if caller wants to
    // also keep track of states
    fn save_game_state(&mut self, frame: FrameSize) -> Self::SavedState;
    fn load_game_state(&mut self, saved_state: Self::SavedState, frame: FrameSize);
    fn advance_frame(&mut self);
    fn on_event();
}

#[derive(PartialEq, Debug)]
pub struct SaveFrame {
    pub frame: FrameSize,
}

#[derive(PartialEq, Debug)]
pub struct RollbackState {
    pub frame: FrameSize,
    pub num_steps: FrameSize,
}

// pub enum RequiredAction {
//     /// Save current game state which can be looked up by the frame param
//     SaveState(SaveFrame),
//     /// Load state corresponding to the frame param and advance your game
// state     /// by num_steps
//     Rollback(RollbackState),
// }
