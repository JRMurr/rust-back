use crate::{
    error::SyncError, game_input_frame::GameInputFrame, input_queue::InputQueue, FrameIndex,
    FrameSize, GameInput, RollbackState, SaveFrame,
};
use std::collections::VecDeque;
// TODO: simplify errors to only be the errors that could be thrown in that func

const NUM_PLAYERS: u8 = 2;
#[derive(Debug)]
pub struct Sync<T: GameInput> {
    max_prediction_frames: FrameSize,
    pub(crate) frame_count: FrameSize,
    last_confirmed_frame: FrameIndex,
    target_post_roll_back_frame: FrameIndex,
    // first is local, second is remote
    // TODO: maybe make this slice of fixed size or just a vec?
    input_queues: (InputQueue<T>, InputQueue<T>),
    saved_states: VecDeque<FrameSize>,
}

impl<T: GameInput> Sync<T> {
    pub fn new(max_prediction_frames: FrameSize) -> Self {
        Self {
            max_prediction_frames,
            frame_count: 0,
            last_confirmed_frame: None,
            input_queues: (InputQueue::new(), InputQueue::new()),
            saved_states: VecDeque::new(),
            target_post_roll_back_frame: None,
        }
    }

    pub fn in_rollback(&self) -> bool {
        self.target_post_roll_back_frame.is_some()
    }

    pub fn set_last_confirmed_frame(&mut self, frame: FrameSize) {
        self.last_confirmed_frame = Some(frame);
        if frame > 0 {
            self.input_queues.0.discard_confirmed_frames(frame - 1);
            self.input_queues.1.discard_confirmed_frames(frame - 1);
        }
    }

    pub fn save_current_frame(&mut self) -> SaveFrame {
        self.saved_states.push_back(self.frame_count);
        SaveFrame {
            frame: self.frame_count,
        }
    }

    fn load_frame(
        &mut self,
        frame: FrameSize,
        num_steps: FrameSize,
    ) -> Result<RollbackState, SyncError> {
        // remove older frames from saved states
        self.saved_states
            .retain(|saved_frame: &FrameSize| *saved_frame >= frame);

        match self.saved_states.pop_front() {
            Some(saved_frame) if saved_frame == frame => {
                self.frame_count = frame;
                self.reset_prediction(self.frame_count)?;
                Ok(RollbackState { frame, num_steps })
            }
            // TODO: could error if i suck at the queue
            _ => Err(SyncError::StateNotFound(frame)),
        }
    }

    #[inline(always)]
    fn get_queue_mut(&mut self, queue: u8) -> Result<&mut InputQueue<T>, SyncError> {
        // TODO: right now we only hold 2 but ggpo does up to 4
        // not sure if we need more for spectators
        match queue {
            0 => Ok(&mut self.input_queues.0),
            1 => Ok(&mut self.input_queues.1),
            _ => Err(SyncError::BadQueueHandle(queue)),
        }
    }

    #[inline(always)]
    fn get_queue(&self, queue: u8) -> Result<&InputQueue<T>, SyncError> {
        // TODO: right now we only hold 2 but ggpo does up to 4
        // not sure if we need more for spectators
        match queue {
            0 => Ok(&self.input_queues.0),
            1 => Ok(&self.input_queues.1),
            _ => Err(SyncError::BadQueueHandle(queue)),
        }
    }

    fn add_input(
        &mut self,
        queue: u8,
        input: GameInputFrame<T>,
    ) -> Result<GameInputFrame<T>, SyncError> {
        let queue = self.get_queue_mut(queue)?;
        queue.add_input(input).map_err(SyncError::from)
    }

    pub fn add_remote_input(
        &mut self,
        queue: u8,
        input: GameInputFrame<T>,
    ) -> Result<GameInputFrame<T>, SyncError> {
        // TODO: should it only be queue == 1?
        self.add_input(queue, input)
    }

    pub fn add_local_input(
        &mut self,
        queue: u8,
        input: GameInputFrame<T>,
    ) -> Result<GameInputFrame<T>, SyncError> {
        if let Some(last_confirmed_frame) = self.last_confirmed_frame {
            let frames_behind = self.frame_count - last_confirmed_frame;
            if frames_behind >= self.max_prediction_frames {
                return Err(SyncError::PredictionBarrierReached {
                    frames_behind,
                    max_prediction_frames: self.max_prediction_frames,
                });
            }
        }

        // TODO: ggpo has this but not sure why yet
        // TODO: could this be called twice if 2 queues add input before
        // incrementating? It should really happen but maybe check if q = 0 so it only
        // does it for local?
        if self.frame_count == 0 {
            // self.save_current_frame()
            // TODO: require they save frame 0 before calling this?
            // Or return option of SaveFrame telling them to save?
        }

        // TODO: should it only be queue == 0?
        self.add_input(queue, input)
    }

    pub fn increment_frame(&mut self) -> SaveFrame {
        self.frame_count += 1;
        self.save_current_frame()
    }

    pub fn set_frame_delay(&mut self, queue: u8, delay: FrameSize) -> Result<(), SyncError> {
        self.get_queue_mut(queue)?.set_frame_delay(delay);
        Ok(())
    }

    fn reset_prediction(&mut self, frame: FrameSize) -> Result<(), SyncError> {
        self.input_queues.0.reset_prediction(frame)?;
        self.input_queues.1.reset_prediction(frame)?;
        Ok(())
    }

    fn check_simulation_consistency(&self) -> FrameIndex {
        // TODO: cleanup seems really gross
        let mut first_incorrect_frame = None;
        for i in 0..NUM_PLAYERS {
            let q = self.get_queue(i).expect("Should always be a valid queue");
            match (q.first_incorrect_frame, first_incorrect_frame) {
                (Some(q_frame), Some(sim_frame)) => {
                    if q_frame < sim_frame {
                        first_incorrect_frame = q.first_incorrect_frame;
                    }
                }
                (Some(_), None) => {
                    first_incorrect_frame = q.first_incorrect_frame;
                }
                _ => {}
            }
        }
        first_incorrect_frame
    }

    pub fn check_simulation(&mut self) -> Result<Option<RollbackState>, SyncError> {
        let seek_to = self.check_simulation_consistency();
        match seek_to {
            Some(seek_to) => Ok(Some(self.pre_roll_back(seek_to)?)),
            None => Ok(None),
        }
    }

    // pre_roll_back and post_roll_back map to AdjustSimulation in ggpo
    pub fn pre_roll_back(&mut self, seek_to: FrameSize) -> Result<RollbackState, SyncError> {
        let count = self.frame_count - seek_to;
        self.target_post_roll_back_frame = Some(self.frame_count);

        self.load_frame(seek_to, count)
        // TODO: ggpo has assert here https://github.com/pond3r/ggpo/blob/7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a/src/lib/ggpo/sync.cpp#L156
        // but i think load frame covers it
    }

    pub fn post_roll_back(&mut self) -> Result<(), SyncError> {
        match self.target_post_roll_back_frame {
            Some(frame_count) => {
                if frame_count != self.frame_count {
                    Err(SyncError::SimulationError {
                        given: self.frame_count,
                        expected: frame_count,
                    })
                } else {
                    self.target_post_roll_back_frame = None;
                    Ok(())
                }
            }
            None => Err(SyncError::NotInRollback),
        }
    }

    /// Called each frame by the game to get inputs for each player
    /// Returns Vec where each index corresponds to the input for that
    /// queue/player
    pub fn synchronize_inputs(&mut self) -> Result<Vec<Option<T>>, SyncError> {
        let mut res = Vec::new();
        let frame = self.frame_count;
        for i in 0..NUM_PLAYERS {
            let queue = self.get_queue_mut(i)?;
            // TODO: check if player disconnected
            res.push(queue.get_input(frame)?.input);
        }
        Ok(res)
    }

    // TODO: i think this is only called by spectators
    pub fn get_confirmed_inputs(
        &mut self,
        frame: FrameSize,
    ) -> Result<Vec<GameInputFrame<T>>, SyncError> {
        let mut res = Vec::new();
        for i in 0..NUM_PLAYERS {
            let queue = self.get_queue(i)?;
            // TODO: check if player disconnected
            res.push(queue.get_confirmed_input(frame)?);
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut sync: Sync<&str> = Sync::new(4);
        // first frame adds
        let added = sync.add_input(0, ("hi_0", 0).into()).unwrap();
        assert_eq!(
            added,
            GameInputFrame {
                frame: Some(0),
                input: Some("hi_0"),
            }
        );
        let added = sync.add_input(1, ("hi_1", 0).into()).unwrap();
        assert_eq!(
            added,
            GameInputFrame {
                frame: Some(0),
                input: Some("hi_1"),
            }
        );

        let err = sync.add_input(10, ("bad queue", 0).into()).err().unwrap();
        assert_eq!(err, SyncError::BadQueueHandle(10));
    }

    #[test]
    fn test_add_local_input() {
        let mut sync: Sync<&str> = Sync::new(4);

        let added = sync.add_local_input(0, ("hi_0", 0).into()).unwrap();
        assert_eq!(
            added,
            GameInputFrame {
                frame: Some(0),
                input: Some("hi_0"),
            }
        );
    }

    fn advance_frame(
        sync: &mut Sync<&str>,
        expected_frame: FrameSize,
        expected_check_simulation_res: Option<RollbackState>,
    ) -> Result<(), SyncError> {
        assert_eq!(
            sync.increment_frame(),
            SaveFrame {
                frame: expected_frame
            }
        );
        assert_eq!(sync.check_simulation()?, expected_check_simulation_res);
        Ok(())
    }

    #[test]
    fn test_check_simulation() -> Result<(), SyncError> {
        let mut sync: Sync<&str> = Sync::new(4);

        // TODO: for now we require they call save state before doing anything
        assert_eq!(sync.save_current_frame(), SaveFrame { frame: 0 });

        // add local inputs but don't add remote to simulate a delay
        sync.add_local_input(0, ("first", 0).into())?;

        assert_eq!(
            sync.synchronize_inputs()?,
            vec![
                Some("first"),
                // second queue has nothing to predict from so it will return null input
                None
            ]
        );
        // simulate game state going forward without getting remote input
        advance_frame(&mut sync, 1, None)?;

        // simulate a few more frames, then get the inputs for the first
        sync.add_local_input(0, ("second", 1).into())?;
        assert_eq!(sync.synchronize_inputs()?, vec![Some("second"), None]);
        advance_frame(&mut sync, 2, None)?;

        // we got inputs for frame 0 on the start of frame 2 so we should roll back to
        // the start of frame 2
        sync.add_local_input(0, ("third", 2).into())?;
        sync.add_remote_input(1, ("remote_1", 0).into())?;

        // the remote input queue now knows it needs to rollback so error getting inputs
        assert_eq!(
            sync.synchronize_inputs().err().unwrap(),
            SyncError::QueueError(crate::error::InputQueueError::GetDurningPrediction)
        );

        // This would be called every frame with increment_frame
        assert_eq!(
            sync.check_simulation()?,
            Some(RollbackState {
                frame: 0,
                num_steps: 2,
            },)
        );

        // If they don't rollback error
        assert_eq!(
            sync.post_roll_back().err().unwrap(),
            SyncError::SimulationError {
                given: 0,
                expected: 2,
            }
        );

        // should get remote input now and use old local input
        assert_eq!(
            sync.synchronize_inputs()?,
            vec![Some("first"), Some("remote_1")]
        );
        advance_frame(&mut sync, 1, None)?;

        // not enough rollback
        assert_eq!(
            sync.post_roll_back().err().unwrap(),
            SyncError::SimulationError {
                given: 1,
                expected: 2,
            }
        );

        // does not yet have the next input so it should predict with the last remote
        assert_eq!(
            sync.synchronize_inputs()?,
            vec![Some("second"), Some("remote_1")]
        );
        advance_frame(&mut sync, 2, None)?;

        // Correctly rolled back so no error
        sync.post_roll_back()?;

        // now play the frame as normal
        advance_frame(&mut sync, 3, None)?;

        // we get inputs for frame 1 on the start of frame 3 so roll back to here
        sync.add_local_input(0, ("fourth", 3).into())?;
        sync.add_remote_input(1, ("remote_2", 1).into())?;

        // This would be called every frame with increment_frame
        assert_eq!(
            sync.check_simulation()?,
            Some(RollbackState {
                frame: 1,
                num_steps: 2,
            },)
        );

        assert_eq!(
            sync.synchronize_inputs()?,
            vec![Some("second"), Some("remote_2")]
        );
        advance_frame(&mut sync, 2, None)?;

        assert_eq!(
            sync.synchronize_inputs()?,
            vec![Some("third"), Some("remote_2")]
        );
        advance_frame(&mut sync, 3, None)?;

        // Correctly rolled back so no error
        sync.post_roll_back()?;

        Ok(())
    }
}
