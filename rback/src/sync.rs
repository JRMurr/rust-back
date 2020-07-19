use crate::{game_input_frame::GameInputFrame, FrameSize};

//TODO: make it generic where the passed type is the game state
trait SyncCallBacks {
    fn save_game_state();
    fn load_game_state();
    fn advance_frame();
    fn on_event();
}

pub struct SyncConfig {
    num_prediction_frames: u8,
}

pub struct Sync {
    config: SyncConfig,
    frame_count: FrameSize,
    last_confirmed_frame: usize,
}

// impl Sync {
//     pub fn new(config: SyncConfig) -> Self {
//         Self {
//             config,
//             frame_count: 0,
//             last_confirmed_frame: 0,
//         }
//     }
// }
