use crate::{
    error::SyncError, game_input_frame::GameInputFrame, input_queue::InputQueue, FrameIndex,
    FrameSize, GameInput, SyncCallBacks,
};
use std::collections::VecDeque;
// TODO: simplify errors to only be the errors that could be thrown in that func

struct SavedGameState<T> {
    state: T,
    frame: FrameSize,
}

impl<T> From<(T, FrameSize)> for SavedGameState<T> {
    fn from(inner: (T, FrameSize)) -> Self {
        Self {
            state: inner.0,
            frame: inner.1,
        }
    }
}

const NUM_PLAYERS: u8 = 2;
pub struct Sync<T: GameInput, C: SyncCallBacks> {
    max_prediction_frames: FrameSize,
    pub(crate) frame_count: FrameSize,
    last_confirmed_frame: FrameIndex,
    pub(crate) rolling_back: bool,
    // first is local, second is remote
    // TODO: maybe make this slice of fixed size or just a vec?
    input_queues: (InputQueue<T>, InputQueue<T>),
    saved_states: VecDeque<SavedGameState<<C as SyncCallBacks>::SavedState>>,
    callbacks: C,
}

impl<T: GameInput, C: SyncCallBacks> Sync<T, C> {
    pub fn new(max_prediction_frames: FrameSize, callbacks: C) -> Self {
        Self {
            max_prediction_frames,
            frame_count: 0,
            last_confirmed_frame: None,
            rolling_back: false,
            input_queues: (InputQueue::new(), InputQueue::new()),
            saved_states: VecDeque::new(),
            callbacks,
        }
    }

    pub fn set_last_confirmed_frame(&mut self, frame: FrameSize) {
        self.last_confirmed_frame = Some(frame);
        if frame > 0 {
            self.input_queues.0.discard_confirmed_frames(frame - 1);
            self.input_queues.1.discard_confirmed_frames(frame - 1);
        }
    }

    fn save_current_frame(&mut self) {
        let saved_state = self.callbacks.save_game_state();
        self.saved_states
            .push_back((saved_state, self.frame_count).into());
    }

    fn load_frame(&mut self, frame: FrameSize) -> Result<(), SyncError> {
        // remove older frames from saved states
        self.saved_states.retain(|state| state.frame >= frame);

        match self.saved_states.pop_front() {
            Some(state) if state.frame == frame => {
                self.callbacks.load_game_state(state.state);
                Ok(())
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

    fn add_input(&mut self, queue: u8, input: T) -> Result<GameInputFrame<T>, SyncError> {
        let input = GameInputFrame {
            frame: Some(self.frame_count),
            input: Some(input),
        };
        let queue = self.get_queue_mut(queue)?;
        queue.add_input(input).map_err(SyncError::from)
    }

    pub fn add_remote_input(
        &mut self,
        queue: u8,
        input: T,
    ) -> Result<GameInputFrame<T>, SyncError> {
        // TODO: should it only be queue == 1?
        self.add_input(queue, input)
    }

    pub fn add_local_input(&mut self, queue: u8, input: T) -> Result<GameInputFrame<T>, SyncError> {
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
        if self.frame_count == 0 {
            self.save_current_frame()
        }

        // TODO: should it only be queue == 0?
        self.add_input(queue, input)
    }

    pub fn increment_frame(&mut self) {
        self.frame_count += 1;
        self.save_current_frame();
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
            if let (Some(q_frame), Some(sim_frame)) =
                (q.first_incorrect_frame, first_incorrect_frame)
            {
                if q_frame < sim_frame {
                    first_incorrect_frame = q.first_incorrect_frame;
                }
            }
        }
        first_incorrect_frame
    }

    pub fn check_simulation(&mut self) -> Result<(), SyncError> {
        let seek_to = self.check_simulation_consistency();
        match seek_to {
            Some(seek_to) => self.adjust_simulation(seek_to),
            None => Ok(()),
        }
    }

    pub fn adjust_simulation(&mut self, seek_to: FrameSize) -> Result<(), SyncError> {
        let frame_count = self.frame_count;
        let count = self.frame_count - seek_to;
        self.rolling_back = true;

        self.load_frame(seek_to)?;
        // TODO: ggpo has assert here https://github.com/pond3r/ggpo/blob/7ddadef8546a7d99ff0b3530c6056bc8ee4b9c0a/src/lib/ggpo/sync.cpp#L156
        // but i think load frame covers it

        self.reset_prediction(self.frame_count)?;
        for _ in 0..count {
            self.callbacks.advance_frame();
        }

        self.rolling_back = false;
        if frame_count != self.frame_count {
            Err(SyncError::SimulationError {
                given: self.frame_count,
                expected: frame_count,
            })
        } else {
            Ok(())
        }
    }

    /// Called each frame by the game to get inputs for each player
    pub fn synchronize_inputs(&mut self) -> Result<Vec<GameInputFrame<T>>, SyncError> {
        let mut res = Vec::new();
        let frame = self.frame_count;
        for i in 0..NUM_PLAYERS {
            let queue = self.get_queue_mut(i)?;
            // TODO: check if player disconnected
            res.push(queue.get_input(frame)?);
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

    struct SavedState {}

    struct TestCallBacks {}

    impl SyncCallBacks for TestCallBacks {
        type SavedState = SavedState;
        fn save_game_state(&self) -> Self::SavedState {
            SavedState {}
        }
        fn load_game_state(&self, _saved_state: Self::SavedState) {}
        fn advance_frame(&mut self) {}
        fn on_event() {
            todo!()
        }
    }

    #[test]
    fn test_add() {}
}
