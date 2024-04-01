use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid End Time")]
    InvalidEndTime,

    #[error("Already initialized")]
    AlreadyInitialized,

    #[error("Not initialized yet")]
    NotInitialized,
}
