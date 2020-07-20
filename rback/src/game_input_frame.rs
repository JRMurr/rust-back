use crate::{FrameSize, GameInput};
use std::fmt::Debug;
#[derive(Debug, Clone)]
// TODO: maybe make frame not an option since i think only the prediction input
// would have it be none
pub struct GameInputFrame<T: GameInput> {
    pub frame: Option<FrameSize>,
    pub input: Option<T>,
}

impl<T: GameInput> PartialEq for GameInputFrame<T> {
    /// Game Input equality only cares about the inputs not frames
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
    }
}

impl<T: GameInput> GameInputFrame<T> {
    pub fn empty_input() -> Self {
        Self {
            frame: None,
            input: None,
        }
    }

    pub fn erase_input(&mut self) {
        self.input = None
    }
}