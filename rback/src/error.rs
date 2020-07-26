use crate::FrameSize;
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

#[derive(Debug, PartialEq)]
// These are the different assert errors GGPO would throw
// TODO: so panic when these happen?
pub enum InputQueueError {
    NonSequentialUserInput {
        given: FrameSize,
        expected: FrameSize,
    },
    // if this is thrown the library messed up somehow
    NonSequentialRollbackInput {
        given: FrameSize,
        expected: FrameSize,
    },
    BadFrameIndex {
        given: FrameSize,
        tail_frame: FrameSize,
    },
    BadFrameRequest {
        given: FrameSize,
        first_incorrect_frame: FrameSize,
    },
    BadResetPrediction {
        given: FrameSize,
        first_incorrect_frame: FrameSize,
    },
    FrameNotFound(FrameSize),
    GetDurningPrediction,
    BadInput,
}

impl Display for InputQueueError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InputQueueError::NonSequentialUserInput{given, expected} => write!(
                fmt,
                "Given input with frame number of {}, expected input to be for frame {}",
                given, expected
            ),
            InputQueueError::NonSequentialRollbackInput{given, expected} => write!(
                fmt,
                "Given frame number of {}, expected frame number {}",
                given, expected
            ),
            InputQueueError::BadFrameIndex{given, tail_frame} => write!(
                fmt,
                "Tried to request frame number of {}, which is behind the tail frame of {}",
                given, tail_frame
            ),
            InputQueueError::BadFrameRequest{given, first_incorrect_frame} => write!(
                fmt,
                "Tried to request frame number of {}, which is behind the first_incorrect_frame of {}",
                given, first_incorrect_frame
            ),
            InputQueueError::BadResetPrediction{given, first_incorrect_frame} => write!(
                fmt,
                "Tried to reset prediction to frame {}, which is ahead of the first_incorrect_frame of {}",
                given, first_incorrect_frame
            ),
            InputQueueError::FrameNotFound(given) => {
                write!(fmt, "Tried to request frame number of {}, which was not found", given)
            }
            InputQueueError::GetDurningPrediction => {
                write!(fmt, "Attempted to get input when there is a prediction error. You need to rollback")
            }
            InputQueueError::BadInput => write!(fmt, "Given input with None for frame number"),
        }
    }
}

impl Error for InputQueueError {}

#[derive(Debug, PartialEq)]
pub enum SyncError {
    QueueError(InputQueueError),
    BadQueueHandle(u8),
    PredictionBarrierReached {
        frames_behind: FrameSize,
        max_prediction_frames: FrameSize,
    },
    SimulationError {
        given: FrameSize,
        expected: FrameSize,
    },
    StateNotFound(FrameSize),
    NotInRollback,
}

impl Display for SyncError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SyncError::QueueError(e) => {
                write!(fmt, "Something went adding to input queue. error: {:?}.", e)
            }
            SyncError::BadQueueHandle(q) => {
                write!(fmt, "Tried to write to q {}, which does not exist", q)
            }
            SyncError::PredictionBarrierReached {
                frames_behind,
                max_prediction_frames,
            } => write!(
                fmt,
                "Rejecting input prediction barrier: currently {} frames behind with max_prediction_frames: {} ",
                frames_behind, max_prediction_frames
            ),
            SyncError::SimulationError{given, expected} => write!(
                fmt,
                "After rolling back frame count is at {}, when it should be at {}",
                given, expected
            ),
            SyncError::StateNotFound(frame) => {
                write!(fmt, "frame {} not found in saved states", frame)
            },
            SyncError::NotInRollback => write!(fmt, "Called post_adjust_simulation when not in a rollback")
        }
    }
}

impl Error for SyncError {}

impl From<InputQueueError> for SyncError {
    fn from(inner: InputQueueError) -> Self {
        SyncError::QueueError(inner)
    }
}

#[derive(Debug, PartialEq)]
pub enum BackendError {
    PlayerOutOfRange { given: u8, num_players: u8 },
}
impl Display for BackendError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::PlayerOutOfRange { given, num_players } => write!(
                fmt,
                "given player number of {}, num players is {}",
                given, num_players
            ),
        }
    }
}

impl Error for BackendError {}
