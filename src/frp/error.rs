use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Cycle detected in the event flow")]
    Cycle,

    #[error("Input was not found")]
    InputNotFound,
    #[error("Output was not found")]
    OutputNotFound,
    #[error("Unexpected event type")]
    UnexpectedEventType,
    #[error("The event type if input and output are not the matching")]
    IncompatiblePinTypes,
}
