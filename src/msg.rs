use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Empty, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub vesting_token: String,
    pub treasury: String,
    pub start_time: u64,
    pub end_time: u64,
    pub period: u64,
    pub is_periodic: bool,
    pub owner: String,
    pub users: Vec<String>,
    pub periodic_reward: Vec<u64>,
}

#[cw_serde]
pub struct ContractInfoResponse {
    pub vesting_token: String,
    pub treasury: String,
    pub start_time: u64,
    pub end_time: u64,
    pub period: u64,
    pub is_periodic: bool,
    pub owner: String,
}

#[cw_serde]
pub struct ClaimableInfoResponse {
    pub claimable_reward: Uint128,
}

#[cw_serde]
pub struct PeriodInfoResponse {
    pub current_period: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    ClaimMsg {},
}

#[cw_serde]
pub enum ReceiveMsg {
    Fund {}, // Example: Fund the contract for claims
}

// We can also add this as a cw3 extension
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ContractInfoResponse)]
    ContractInfo {},

    #[returns(PeriodInfoResponse)]
    PeriodInfo {},
}
