use cosmwasm_schema::cw_serde;
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

    #[error("Invalid period time")]
    InvalidPeriod {},

    #[error("Vesting ended")]
    VestingEnded {},

    #[error("Vesting not started")]
    VestingNotStarted {},

    #[error("Not vesting token")]
    NotVestingToken {},
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> Self {
        StdError::generic_err(err.to_string())
    }
}