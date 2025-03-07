use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("StdError: {0}")]
    Std(#[from] StdError),

    #[error("Only Admin")]
    OnlyAdmin {},

    #[error("Invalid start/end time")]
    InvalidTS {},
}