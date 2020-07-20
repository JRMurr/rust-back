use crate::{
    error::InputQueueError, game_input_frame::GameInputFrame, FrameIndex, FrameSize, GameInput,
};
use log::info;
use std::{cmp::min, collections::VecDeque};

#[derive(Debug)]
/// Queue of inputs for a single player in the game
pub struct InputQueue<T: GameInput> {
    queue: VecDeque<GameInputFrame<T>>, /* TODO: maybe make this a box type
                                         * to reduce/remove clones */
    frame_delay: FrameSize,
    prediction: GameInputFrame<T>,
    /// Frame number of the last user added input
    last_user_added_frame: FrameIndex,
    pub(crate) last_added_frame: FrameIndex,
    pub(crate) first_incorrect_frame: FrameIndex,
    last_frame_requested: FrameIndex,
}

impl<T: GameInput> Default for InputQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: GameInput> InputQueue<T> {
    pub fn new() -> Self {
        // TODO: maybe use with capacity or reserve size of queue to prevent
        // extra allocations
        Self {
            queue: VecDeque::new(),
            last_user_added_frame: None,
            last_added_frame: None,
            frame_delay: 0,
            prediction: GameInputFrame::empty_input(),
            first_incorrect_frame: None,
            last_frame_requested: None,
        }
    }

    #[inline]
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
                    return Err(InputQueueError::NonSequentialUserInput {
                        given: input_frame,
                        expected: last_user_added_frame + 1,
                    });
                } else {
                    return Err(InputQueueError::NonSequentialRollbackInput {
                        given: input_frame,
                        expected: last_user_added_frame + 1,
                    });
                };
            }
        }
        Ok(input_frame)
    }

    pub fn get_input(
        &mut self,
        requested_frame: FrameSize,
    ) -> Result<GameInputFrame<T>, InputQueueError> {
        if self.first_incorrect_frame.is_some() {
            // https://github.com/pond3r/ggpo/blob/7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a/src/lib/ggpo/input_queue.cpp#L122
            return Err(InputQueueError::GetDurningPrediction);
        }
        let tail = self
            .queue
            .back()
            .expect("Queue should be non empty when getting inputs");
        let tail_frame = tail.frame.expect("Queue inputs should have a frame set");
        if requested_frame < tail_frame {
            return Err(InputQueueError::BadFrameIndex {
                given: requested_frame,
                tail_frame,
            });
        }

        self.last_frame_requested = Some(requested_frame);
        if self.prediction.frame.is_none() {
            let idx_from_back = (requested_frame - tail_frame) as usize;
            if idx_from_back < self.queue.len() {
                // Valid frame no need to predict
                let q_idx = (self.queue.len() - 1) - idx_from_back;
                let desired_input = self
                    .queue
                    .get(q_idx)
                    .expect("Requested frame should be in the queue");
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
                let previous = self
                    .queue
                    .front()
                    .expect("Queue should be non empty in get input");
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

    pub fn add_input(
        &mut self,
        input: GameInputFrame<T>,
    ) -> Result<GameInputFrame<T>, InputQueueError> {
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
        input: GameInputFrame<T>,
        frame_num: FrameSize,
    ) -> Result<GameInputFrame<T>, InputQueueError> {
        // do i need this assert? https://github.com/pond3r/ggpo/blob/7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a/src/lib/ggpo/input_queue.cpp#L221

        let input_frame = match self.queue.front() {
            Some(q_input) => {
                let last_added_frame = q_input.frame.expect("All queue inputs should be set");
                Self::check_sequential(Some(last_added_frame), Some(frame_num), false)?
            }
            None => 0,
        };
        let mut input = input;
        input.frame = Some(input_frame);
        self.queue.push_front(input.clone());
        self.last_added_frame = input.frame;

        if let Some(prediction_frame) = self.prediction.frame {
            debug_assert_eq!(
                frame_num, prediction_frame,
                "need added input to be the prediction frame, got {}, expected {}",
                frame_num, prediction_frame
            );

            // We have been doing predictions so check if what we have
            // prediction matched the inputs we got
            if self.first_incorrect_frame.is_none() && self.prediction != input {
                info!(
                    "frame {} does not match prediction.  marking error.",
                    frame_num
                );
                self.first_incorrect_frame = Some(frame_num);
            }

            if self.prediction.frame == self.last_frame_requested
                && self.first_incorrect_frame.is_none()
            {
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
        let expected_frame = match self.queue.front() {
            Some(input) => {
                input
                    .frame
                    .expect("All inputs in queue should have a frame number set")
                    + 1
            }
            // ggpo sets expected to 0 if `_first_frame` is true so
            // i think if theres no 2nd elm in the queue it would do the same
            // thing?
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
            let last_input = self.queue.front().expect("queue should be non empty");
            let last_input = last_input.clone();
            self.add_delayed_input(last_input, frame_num)?;
        }
        Ok(Some(frame))
    }

    pub fn get_confirmed_input(
        self,
        requested_frame: FrameSize,
    ) -> Result<GameInputFrame<T>, InputQueueError> {
        if let Some(first_incorrect_frame) = self.first_incorrect_frame {
            if requested_frame > first_incorrect_frame {
                return Err(InputQueueError::BadFrameRequest {
                    given: requested_frame,
                    first_incorrect_frame,
                });
            }
        }
        // TODO: find based on tail?
        let input = self.queue.iter().find(|input| match input.frame {
            Some(frame_num) => frame_num == requested_frame,
            None => false,
        });
        match input {
            Some(game_input_frame) => Ok(game_input_frame.clone()),
            None => Err(InputQueueError::FrameNotFound(requested_frame)),
        }
    }

    pub fn discard_confirmed_frames(&mut self, frame: FrameSize) {
        let frame = match self.last_frame_requested {
            Some(last_frame) => min(last_frame, frame),
            None => frame,
        };
        self.queue.retain(|input| match input.frame {
            Some(input_frame) => input_frame >= frame, /* TODO: should be
                                                         * just greater or is
                                                         * >= */
            // fine?
            None => false,
        });
    }

    pub fn set_frame_delay(&mut self, delay: FrameSize) {
        self.frame_delay = delay;
    }

    pub fn get_length(self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut q: InputQueue<&str> = InputQueue::new();
        let input = GameInputFrame {
            frame: Some(0),
            input: Some("hi"),
        };
        let added = q.add_input(input).unwrap();
        assert_eq!(
            added,
            GameInputFrame {
                frame: Some(0),
                input: Some("hi")
            }
        );

        // Try to add same frame number
        let input = GameInputFrame {
            frame: Some(0),
            input: Some("hello"),
        };
        let added = q.add_input(input);
        assert!(added.is_err());
        let err = added.err().unwrap();
        assert_eq!(
            err,
            InputQueueError::NonSequentialUserInput {
                given: 0,
                expected: 1
            }
        );

        // try bad frame number
        let input = GameInputFrame {
            frame: Some(10),
            input: Some("hello"),
        };

        let added = q.add_input(input);
        assert!(added.is_err());
        let err = added.err().unwrap();
        assert_eq!(
            err,
            InputQueueError::NonSequentialUserInput {
                given: 10,
                expected: 1
            }
        );

        // correct frame number
        let input = GameInputFrame {
            frame: Some(1),
            input: Some("its real"),
        };
        let added = q.add_input(input).unwrap();
        assert_eq!(
            added,
            GameInputFrame {
                frame: Some(1),
                input: Some("its real")
            }
        );
    }

    #[test]
    fn test_get_input() -> Result<(), InputQueueError> {
        let mut q: InputQueue<&str> = InputQueue::new();

        let input = GameInputFrame {
            frame: Some(0),
            input: Some("hi"),
        };
        q.add_input(input)?;
        let input = GameInputFrame {
            frame: Some(1),
            input: Some("hello"),
        };
        q.add_input(input)?;

        // get good frames
        assert_eq!(
            q.get_input(0)?,
            GameInputFrame {
                frame: Some(0),
                input: Some("hi"),
            }
        );
        assert_eq!(
            q.get_input(1)?,
            GameInputFrame {
                frame: Some(0),
                input: Some("hello"),
            }
        );

        // TODO: add test when requested_frame < tail_frame
        // TODO: test empty predictions, i think the queue has to be empty for
        // these and we error

        // get bad frame so should try to predict based on last added frame
        assert_eq!(
            q.get_input(3)?,
            GameInputFrame {
                frame: Some(3),
                input: Some("hello"),
            }
        );
        Ok(())
    }
}
