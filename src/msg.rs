use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Empty};

#[cw_serde]
pub struct InstantiateMsg {
    pub vesting_token: String,
    pub treasury: String,
    pub start_time: u64,
    pub end_time: u64,
    pub period: u64,
    pub is_periodic: bool,
    pub owner: String,
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

// We can also add this as a cw3 extension
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ContractInfoResponse)]
    ContractInfo {},
}