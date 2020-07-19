use crate::FrameSize;
use std::fmt::Debug;
#[derive(Debug, Clone)]
// T is the type of the input to store
pub struct GameInput<T: Clone + Debug + PartialEq> {
    pub frame: Option<FrameSize>,
    pub input: Option<T>,
}

impl<T: Clone + Debug + PartialEq> PartialEq for GameInput<T> {
    /// Game Input equality only cares about the inputs not frames
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
    }
}

impl<T: Clone + Debug + PartialEq> GameInput<T> {
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
