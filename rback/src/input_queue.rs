use crate::FrameSize;
use std::{
    collections::VecDeque,
    fmt::{self, Display, Formatter},
};
#[derive(Debug)]
pub enum InputQueueError {
    NonSequentialUserInput(FrameSize, FrameSize),
    // if this is thrown the library messed up somehow
    NonSequentialRollbackInput(FrameSize, FrameSize),
    BadInput,
}

impl Display for InputQueueError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InputQueueError::NonSequentialUserInput(given, expected) => write!(
                fmt,
                "Given input with frame number of {}, expected input to be for frame {}",
                given, expected
            ),
            InputQueueError::NonSequentialRollbackInput(given, expected) => write!(
                fmt,
                "Given frame number of {}, expected frame number {}",
                given, expected
            ),
            InputQueueError::BadInput => write!(fmt, "Given input with None for frame number"),
        }
    }
}

#[derive(Debug, Clone)]
// T is the type of the input to store
pub struct GameInput<T: Clone> {
    frame: Option<FrameSize>,
    input: T,
}

#[derive(Debug)]
/// Queue of inputs for a single player in the game
pub struct InputQueue<'input, T: Clone> {
    queue: VecDeque<&'input GameInput<T>>,
    /// Frame number of the last user added input
    last_user_added_frame: Option<FrameSize>,
    frame_delay: FrameSize,
}

impl<'input, T: Clone> Default for InputQueue<'input, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'input, T: Clone> InputQueue<'input, T> {
    pub fn new() -> Self {
        // TODO: maybe use with capacity or reserve size of queue to prevent extra
        // allocs
        Self {
            queue: VecDeque::new(),
            last_user_added_frame: None,
            frame_delay: 0,
        }
    }

    fn check_sequential(
        last_frame: Option<FrameSize>,
        input_frame: Option<FrameSize>,
        user_input: bool,
    ) -> Result<FrameSize, InputQueueError> {
        let input_frame = input_frame.ok_or(InputQueueError::BadInput)?;

        if let Some(last_user_added_frame) = last_frame {
            // input must be added sequentially
            if input_frame != last_user_added_frame + 1 {
                if user_input {
                    return Err(InputQueueError::NonSequentialUserInput(
                        input_frame,
                        last_user_added_frame + 1,
                    ));
                } else {
                    return Err(InputQueueError::NonSequentialRollbackInput(
                        input_frame,
                        last_user_added_frame + 1,
                    ));
                };
            }
        }
        Ok(input_frame)
    }

    pub fn add_input(&mut self, input: &'input mut GameInput<T>) -> Result<(), InputQueueError> {
        let input_frame = Self::check_sequential(self.last_user_added_frame, input.frame, true)?;
        self.last_user_added_frame = Some(input_frame);
        let new_frame = self.advance_queue_head(input_frame)?;
        if let Some(new_frame) = new_frame {
            self.add_delayed_input(input, new_frame)?;
        }

        // TODO: Maybe return the input instead of mutating it?
        // also do the same modification in add_delayed_input so is it needed?
        input.frame = new_frame;
        Ok(())
    }

    fn add_delayed_input(
        &mut self,
        input: &'input mut GameInput<T>,
        frame_num: FrameSize,
    ) -> Result<Option<FrameSize>, InputQueueError> {
        // do i need this assert? https://github.com/pond3r/ggpo/blob/7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a/src/lib/ggpo/input_queue.cpp#L221
        let last_added_frame = match self.queue.front() {
            Some(q_input) => q_input.frame.expect("All queue inputs should be set"),
            None => 0, // TODO: might cause issues in sequential check
        };
        // TODO: if this causes issues with the first frame added either don't call it
        // if the q is empty the queue is empty or don't use the same helper
        let input_frame = Self::check_sequential(Some(last_added_frame), Some(frame_num), false)?;
        input.frame = Some(input_frame);
        self.queue.push_front(input);
        panic!("ass");
    }

    fn advance_queue_head(&mut self, frame: FrameSize) -> Result<Option<FrameSize>, InputQueueError> {
        // expected frame is the 2nd input in the queue
        let expected_frame = match self.queue.get(1) {
            Some(input) => input.frame.expect("All inputs in queue should have a frame number set"),
            // ggpo sets expected to 0 if `_first_frame` is true so
            // i think if theres no 2nd elm in the queue it would do the same thing?
            None => 0,
        };
        let frame = frame + self.frame_delay;

        if expected_frame > frame {
            // https://github.com/pond3r/ggpo/blob/7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a/src/lib/ggpo/input_queue.cpp#L278
            // delay has dropped so dont add anything to queue
            return Ok(None);
        }

        for frame_num in expected_frame..frame {
            // https://github.com/pond3r/ggpo/blob/7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a/src/lib/ggpo/input_queue.cpp#L288
            let last_input = *(self.queue.get(1).expect("queue should have at least 2 elements"));
            let mut last_input = (*last_input).clone();
            self.add_delayed_input(&mut last_input, frame_num)?;
        }
        Ok(Some(frame))
    }
}
