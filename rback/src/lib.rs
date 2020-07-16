pub mod input_queue;
pub mod network;
pub mod sync;

// With this we can keep track of about 3 years worth of frames
// at 60fps...
type FrameSize = u32;
