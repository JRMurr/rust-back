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

pub trait SavedGameState {
    fn get_frame(&self) -> FrameSize;
}

pub trait SyncCallBacks<T: SavedGameState> {
    fn save_game_state(&self, frame: FrameSize) -> T;
    fn load_game_state(&self, saved_state: T);
    fn advance_frame(&mut self);
    fn on_event();
}
