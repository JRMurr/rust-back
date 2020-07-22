use crate::{
    error::SyncError, game_input_frame::GameInputFrame, input_queue::InputQueue, FrameIndex,
    FrameSize, GameInput, RcRef, SyncCallBacks,
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
    callbacks: RcRef<C>,
}

impl<T: GameInput, C: SyncCallBacks> Sync<T, C> {
    pub fn new(max_prediction_frames: FrameSize, callbacks: RcRef<C>) -> Self {
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
        let saved_state = self
            .callbacks
            .borrow_mut()
            .save_game_state(self.frame_count);
        self.saved_states
            .push_back((saved_state, self.frame_count).into());
    }

    fn load_frame(&mut self, frame: FrameSize) -> Result<(), SyncError> {
        // remove older frames from saved states
        self.saved_states.retain(|state| state.frame >= frame);

        match self.saved_states.pop_front() {
            Some(state) if state.frame == frame => {
                self.callbacks
                    .borrow_mut()
                    .load_game_state(state.state, frame);
                self.frame_count = frame;
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

    pub fn check_simulation(&mut self) -> Result<(), SyncError> {
        let seek_to = self.check_simulation_consistency();
        println!("seek_to: {:#?}", seek_to);
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
            self.callbacks.borrow_mut().advance_frame();
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
    /// Returns Vec where each index corresponds to the input for that
    /// queue/player
    pub fn synchronize_inputs(&mut self) -> Result<Vec<Option<T>>, SyncError> {
        let mut res = Vec::new();
        let frame = self.frame_count;
        println!("--------------------------------frame: {}", frame);
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
    use std::{cell::RefCell, rc::Rc};

    #[derive(Debug, PartialEq, Clone)]
    struct SavedState {
        frame: FrameSize,
    }

    struct TestCallBacks {
        pub saved_frames: Vec<SavedState>,
        pub loaded_frames: Vec<SavedState>,
    }

    impl Default for TestCallBacks {
        fn default() -> Self {
            Self {
                saved_frames: vec![],
                loaded_frames: vec![],
            }
        }
    }

    impl SyncCallBacks for TestCallBacks {
        type SavedState = SavedState;
        fn save_game_state(&mut self, frame: FrameSize) -> Self::SavedState {
            let saved_frames = SavedState { frame };
            self.saved_frames.push(saved_frames.clone());
            saved_frames
        }
        fn load_game_state(&mut self, saved_state: Self::SavedState, _frame: FrameSize) {
            self.loaded_frames.push(saved_state)
        }
        fn advance_frame(&mut self) {}
        fn on_event() {
            todo!()
        }
    }

    #[test]
    fn test_add() {
        let callbacks = TestCallBacks::default();
        let callbacks = Rc::new(RefCell::new(callbacks));
        let mut sync: Sync<&str, TestCallBacks> = Sync::new(4, callbacks);
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
        let callbacks = TestCallBacks::default();
        let callbacks = Rc::new(RefCell::new(callbacks));
        let mut sync: Sync<&str, TestCallBacks> = Sync::new(4, callbacks.clone());

        let added = sync.add_local_input(0, ("hi_0", 0).into()).unwrap();
        assert_eq!(
            added,
            GameInputFrame {
                frame: Some(0),
                input: Some("hi_0"),
            }
        );
        // since frame_count is 0 we should have called save current_frame
        assert_eq!(
            callbacks.borrow().saved_frames,
            vec![SavedState { frame: 0 }]
        );
    }

    #[test]
    fn test_check_simulation() -> Result<(), SyncError> {
        let callbacks = TestCallBacks::default();
        let callbacks = Rc::new(RefCell::new(callbacks));
        let mut sync: Sync<&str, TestCallBacks> = Sync::new(4, callbacks.clone());

        // add local inputs but don't add remote to simulate a delay
        sync.add_local_input(0, ("first", 0).into())?;

        let res = sync.synchronize_inputs()?;
        assert_eq!(
            res,
            vec![
                Some("first"),
                // second queue has nothing to predict from so it will return null input
                None
            ]
        );
        // simulate game state going forward without getting remote input
        sync.increment_frame();

        // simulate a few more frames, then get the inputs for the first
        sync.add_local_input(0, ("second", 1).into())?;
        let res = sync.synchronize_inputs()?;
        assert_eq!(res, vec![Some("second"), None]);
        sync.increment_frame();

        // we got inputs 3 frames late
        println!("?????????????????????????????????????");
        sync.add_local_input(0, ("third", 2).into())?;
        sync.add_remote_input(1, ("remote_1", 0).into())?;

        assert_eq!(
            sync.synchronize_inputs().err().unwrap(),
            SyncError::QueueError(crate::error::InputQueueError::GetDurningPrediction)
        );

        // let res = sync.synchronize_inputs()?;
        // assert_eq!(
        //     res,
        //     vec![
        //         Some("third"),
        //         // second queue can now predict from the input it just got
        //         Some("remote_1")
        //     ]
        // );
        // println!("POOP>>>>>>>>>>>>>>>>");
        // sync.increment_frame();

        // we don't check if the input was correct until the
        sync.check_simulation()?;

        // assert_eq!(
        //     callbacks.borrow().loaded_frames,
        //     vec![SavedState { frame: 0 }]
        // );

        // should have called adjust_simulation and loaded the right frame

        Ok(())
    }
}
