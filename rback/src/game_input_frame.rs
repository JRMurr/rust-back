use crate::{FrameSize, GameInput};
use std::fmt::Debug;
#[derive(Debug, Clone, PartialEq)]
// TODO: i think only prediction could have both of these be none
// so might be better if i make a special type for prediction
// so normal game input does not have to deal with unwraps
pub struct GameInputFrame<T: GameInput> {
    pub frame: Option<FrameSize>,
    pub input: Option<T>,
}

impl<T: GameInput> GameInputFrame<T> {
    pub fn new(input: T, frame: FrameSize) -> Self {
        Self {
            frame: Some(frame),
            input: Some(input),
        }
    }

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

// Mostly used for tests to make frames easily
impl<T: GameInput> From<(T, FrameSize)> for GameInputFrame<T> {
    fn from(inner: (T, FrameSize)) -> Self {
        Self {
            input: Some(inner.0),
            frame: Some(inner.1),
        }
    }
}
