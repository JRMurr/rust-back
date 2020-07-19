use crate::{game_input::GameInput, FrameSize};
use log::info;
use std::{
    cmp::min,
    collections::VecDeque,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};
#[derive(Debug)]
// These are the different assert errors GGPO would throw
// TODO: so panic when these happen?
pub enum InputQueueError {
    NonSequentialUserInput(FrameSize, FrameSize),
    // if this is thrown the library messed up somehow
    NonSequentialRollbackInput(FrameSize, FrameSize),
    BadFrameIndex(FrameSize, FrameSize),
    BadFrameRequest(FrameSize, FrameSize),
    FrameNotFound(FrameSize),
    GetDurningPredictionError,
    BadInput,
}

impl Error for InputQueueError {}

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
            InputQueueError::BadFrameIndex(given, tail_frame) => write!(
                fmt,
                "Tried to request frame number of {}, which is behind the tail frame of {}",
                given, tail_frame
            ),
            InputQueueError::BadFrameRequest(given, first_incorrect_frame) => write!(
                fmt,
                "Tried to request frame number of {}, which is behind the first_incorrect_frame of {}",
                given, first_incorrect_frame
            ),
            InputQueueError::FrameNotFound(given) => {
                write!(fmt, "Tried to request frame number of {}, which was not found", given)
            }
            InputQueueError::GetDurningPredictionError => {
                write!(fmt, "Attempted to get input when there is a prediction error.")
            }
            InputQueueError::BadInput => write!(fmt, "Given input with None for frame number"),
        }
    }
}

type FrameIndex = Option<FrameSize>;

#[derive(Debug)]
/// Queue of inputs for a single player in the game
pub struct InputQueue<T: Clone + Debug + PartialEq> {
    queue: VecDeque<GameInput<T>>, // TODO: maybe make this a box type to reduce/remove clones
    /// Frame number of the last user added input
    last_user_added_frame: FrameIndex,
    last_added_frame: FrameIndex,
    frame_delay: FrameSize,
    prediction: GameInput<T>,
    first_incorrect_frame: FrameIndex,
    last_frame_requested: FrameIndex,
}

impl<T: Clone + Debug + PartialEq> Default for InputQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Debug + PartialEq> InputQueue<T> {
    pub fn new() -> Self {
        // TODO: maybe use with capacity or reserve size of queue to prevent extra
        // allocations
        Self {
            queue: VecDeque::new(),
            last_user_added_frame: None,
            last_added_frame: None,
            frame_delay: 0,
            prediction: GameInput::empty_input(),
            first_incorrect_frame: None,
            last_frame_requested: None,
        }
    }

    fn check_sequential(
        last_frame: FrameIndex,
        input_frame: FrameIndex,
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

    pub fn get_input(&mut self, requested_frame: FrameSize) -> Result<GameInput<T>, InputQueueError> {
        if self.first_incorrect_frame.is_some() {
            // https://github.com/pond3r/ggpo/blob/7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a/src/lib/ggpo/input_queue.cpp#L122
            return Err(InputQueueError::GetDurningPredictionError);
        }
        let tail = self
            .queue
            .back()
            .expect("Queue should be non empty when getting inputs");
        let tail_frame = tail.frame.expect("Queue inputs should have a frame set");
        if requested_frame < tail_frame {
            return Err(InputQueueError::BadFrameIndex(requested_frame, tail_frame));
        }

        self.last_frame_requested = Some(requested_frame);
        if self.prediction.frame.is_none() {
            let idx_from_back = (requested_frame - tail_frame) as usize;
            if idx_from_back < self.queue.len() {
                // Valid frame no need to predict
                let q_idx = (self.queue.len() - 1) - idx_from_back;
                let desired_input = self.queue.get(q_idx).expect("Requested frame should be in the queue");
                debug_assert_eq!(
                    desired_input.frame,
                    Some(requested_frame),
                    "requested frame does not match with input in q. Got {:#?}, expected {}",
                    desired_input.frame,
                    requested_frame
                );
                return Ok(desired_input.clone());
            }

            // We need to do some predictions since they want a frame we don't
            // have
            if requested_frame == 0 {
                info!("basing new prediction frame from nothing, you're client wants frame 0.");
                self.prediction.erase_input();
            } else if self.last_added_frame.is_none() {
                info!("basing new prediction frame from nothing, since we have no frames yet.");
                self.prediction.erase_input();
            } else {
                let previous = self.queue.front().expect("Queue should be non empty in get input");
                info!(
                    "basing new prediction frame from previously added frame (queue entry: {:?}).",
                    previous
                );
                self.prediction = previous.clone();
            }
            self.prediction.frame = self.prediction.frame.map(|f| f + 1);
        }
        // TODO: assert prediction frame is >= 0?
        let mut input = self.prediction.clone();
        input.frame = Some(requested_frame);
        Ok(input)
    }

    pub fn add_input(&mut self, input: GameInput<T>) -> Result<GameInput<T>, InputQueueError> {
        let input_frame = Self::check_sequential(self.last_user_added_frame, input.frame, true)?;
        self.last_user_added_frame = Some(input_frame);
        let new_frame = self.advance_queue_head(input_frame)?;
        if let Some(new_frame) = new_frame {
            return self.add_delayed_input(input, new_frame);
        }

        let mut input = input;
        input.frame = new_frame;
        Ok(input)
    }

    fn add_delayed_input(
        &mut self,
        input: GameInput<T>,
        frame_num: FrameSize,
    ) -> Result<GameInput<T>, InputQueueError> {
        // do i need this assert? https://github.com/pond3r/ggpo/blob/7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a/src/lib/ggpo/input_queue.cpp#L221
        let last_added_frame = match self.queue.front() {
            Some(q_input) => q_input.frame.expect("All queue inputs should be set"),
            None => 0, // TODO: might cause issues in sequential check
        };
        // TODO: if this causes issues with the first frame added either don't call it
        // if the q is empty the queue is empty or don't use the same helper
        let input_frame = Self::check_sequential(Some(last_added_frame), Some(frame_num), false)?;
        let mut input = input;
        input.frame = Some(input_frame);
        self.queue.push_front(input.clone());

        if let Some(prediction_frame) = self.prediction.frame {
            debug_assert_eq!(
                frame_num, prediction_frame,
                "need added input to be the prediction frame, got {}, expected {}",
                frame_num, prediction_frame
            );

            // We have been doing predictions so check if what we have
            // prediction matched the inputs we got
            if self.first_incorrect_frame.is_none() && self.prediction != input {
                info!("frame {} does not match prediction.  marking error.", frame_num);
                self.first_incorrect_frame = Some(frame_num);
            }

            if self.prediction.frame == self.last_frame_requested && self.first_incorrect_frame.is_none() {
                info!("prediction is correct!  dumping out of prediction mode.");
                self.prediction.frame = None;
            } else {
                // should be some here but this is cleaner than unwrapping
                self.prediction.frame = self.prediction.frame.map(|f| f + 1);
            }
        }

        Ok(input)
    }

    fn advance_queue_head(&mut self, frame: FrameSize) -> Result<FrameIndex, InputQueueError> {
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
            // delay has dropped so don't add anything to queue
            return Ok(None);
        }

        for frame_num in expected_frame..frame {
            // https://github.com/pond3r/ggpo/blob/7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a/src/lib/ggpo/input_queue.cpp#L288
            let last_input = self.queue.get(1).expect("queue should have at least 2 elements");
            let last_input = last_input.clone();
            self.add_delayed_input(last_input, frame_num)?;
        }
        Ok(Some(frame))
    }

    pub fn get_confirmed_input(self, requested_frame: FrameSize) -> Result<GameInput<T>, InputQueueError> {
        if let Some(first_incorrect_frame) = self.first_incorrect_frame {
            if requested_frame > first_incorrect_frame {
                return Err(InputQueueError::BadFrameRequest(requested_frame, first_incorrect_frame));
            }
        }
        // TODO: find based on tail?
        let input = self.queue.iter().find(|input| match input.frame {
            Some(frame_num) => frame_num == requested_frame,
            None => false,
        });
        match input {
            Some(game_input) => Ok(game_input.clone()),
            None => Err(InputQueueError::FrameNotFound(requested_frame)),
        }
    }

    pub fn discard_confirmed_frames(&mut self, frame: FrameSize) {
        let frame = match self.last_frame_requested {
            Some(last_frame) => min(last_frame, frame),
            None => frame,
        };
        self.queue.retain(|input| match input.frame {
            Some(input_frame) => input_frame >= frame, // TODO: should be just greater or is >= fine?
            None => false,
        });
    }

    pub fn set_frame_delay(&mut self, delay: FrameSize) {
        self.frame_delay = delay;
    }
}
