use cosmwasm_schema::cw_serde;
// use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub vesting_token: Addr,
    pub treasury: Addr,
    pub start_time: u64,
    pub end_time: u64,
    pub period: u64,
    pub is_periodic: bool,
    pub owner: Addr,
}
impl Config {
    pub fn validate_ts(&self) -> bool {
        self.start_time > self.end_time
    }

    pub fn validate_period_non_zero(&self) -> bool {
        self.period != 0
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOTAL_REWARD : Item<Uint128> = Item::new("total_reward");
// map
pub const USER_PERIODIC_REWARD: Map<&Addr, u64> = Map::new("user_periodic_reward");
pub const LAST_CLAIMED_PERIOD: Map<&Addr, u64> = Map::new("last_claimed_period");
