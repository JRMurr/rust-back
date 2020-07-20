use crate::{
    error::SyncError, game_input_frame::GameInputFrame, input_queue::InputQueue, FrameIndex,
    FrameSize, GameInput,
};

//TODO: make it generic where the passed type is the game state
pub trait SyncCallBacks {
    fn save_game_state();
    fn load_game_state();
    fn advance_frame();
    fn on_event();
}

pub struct SyncConfig {
    num_prediction_frames: u8,
}

pub struct Sync<T: GameInput, C: SyncCallBacks> {
    max_prediction_frames: u8,
    frame_count: FrameSize,
    last_confirmed_frame: FrameIndex,
    // first is local, second is remote
    input_queues: (InputQueue<T>, InputQueue<T>),
    callbacks: C,
}

impl<T: GameInput, C: SyncCallBacks> Sync<T, C> {
    pub fn new(config: SyncConfig, callbacks: C) -> Self {
        Self {
            max_prediction_frames: config.num_prediction_frames,
            frame_count: 0,
            last_confirmed_frame: None,
            input_queues: (InputQueue::new(), InputQueue::new()),
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

    fn save_current_frame(&mut self) {}

    pub fn add_local_input(&mut self, queue: u8, input: T) -> Result<GameInputFrame<T>, SyncError> {
        // TODO: prediction barrier check
        let input = GameInputFrame {
            frame: Some(self.frame_count),
            input: Some(input),
        };
        match queue {
            0 => self
                .input_queues
                .0
                .add_input(input)
                .map_err(SyncError::from),
            1 => self
                .input_queues
                .1
                .add_input(input)
                .map_err(SyncError::from),
            _ => Err(SyncError::BadQueueHandle(queue)),
        }
    }
}
