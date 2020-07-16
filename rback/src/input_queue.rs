use crate::FrameSize;
use std::collections::VecDeque;
// T is the type of the input to store
pub struct GameInput<T> {
    frame: FrameSize,
    input: T,
}

pub struct InputQueue<T> {
    queue: VecDeque<GameInput<T>>,
}

impl<T> Default for InputQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> InputQueue<T> {
    pub fn new() -> Self {
        // TODO: maybe use with capacity or reserve size of queue to prevent extra
        // allocs
        Self { queue: VecDeque::new() }
    }

    pub fn add_input(&mut self, input: GameInput<T>) {
        // TODO: it might be better to store it as a hashmap since ggpo always seems to
        // look into the queue by frame number
        self.queue.push_front(input)
    }
}
