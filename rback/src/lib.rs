// #![warn(missing_docs)]

use std::fmt::Debug;

pub mod game_input_frame;
pub mod input_queue;
pub mod network;
pub mod sync;
// With this we can keep track of about 3 years worth of frames
// at 60fps...
type FrameSize = u32;

/// Input passed to be used in rollbacks must satisfy this trait
pub trait GameInput: Clone + Debug + PartialEq {}
impl<T> GameInput for T where T: Clone + Debug + PartialEq {}
