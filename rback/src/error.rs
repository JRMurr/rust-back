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
            InputQueueError::FrameNotFound(given) => {
                write!(fmt, "Tried to request frame number of {}, which was not found", given)
            }
            InputQueueError::GetDurningPrediction => {
                write!(fmt, "Attempted to get input when there is a prediction error.")
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
        }
    }
}

impl From<InputQueueError> for SyncError {
    fn from(inner: InputQueueError) -> Self {
        SyncError::QueueError(inner)
    }
}
