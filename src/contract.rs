#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Addr, Binary, BlockInfo, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response, StdResult, Timestamp, Uint128, WasmMsg
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
// use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{
    ClaimableInfoResponse, ContractInfoResponse, ExecuteMsg, InstantiateMsg, PeriodInfoResponse,
    QueryMsg, ReceiveMsg,
};
use crate::state::{Config, CONFIG, LAST_CLAIMED_PERIOD, TOTAL_REWARD, USER_PERIODIC_REWARD};

const CONTRACT_NAME: &str = "vesting";
const CONTRACT_VERSION: &str = "0.1.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let cfg = Config {
        vesting_token: deps.api.addr_validate(&msg.vesting_token)?,
        treasury: deps.api.addr_validate(&msg.treasury)?,
        start_time: msg.start_time,
        end_time: msg.end_time,
        period: msg.period,
        is_periodic: msg.is_periodic,
        owner: deps.api.addr_validate(&msg.owner)?,
    };
    if !cfg.validate_ts() {
        Err(ContractError::InvalidTS {})
    } else if !cfg.validate_period_non_zero() {
        Err(ContractError::InvalidPeriod {})
    } else {
        for (user, reward) in msg.users.into_iter().zip(msg.periodic_reward.into_iter()) {
            let addr = deps.api.addr_validate(&user)?;
            USER_PERIODIC_REWARD.save(deps.storage, &addr, &reward)?;
        }
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::default())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // let api = _deps.api;
    match _msg {
        ExecuteMsg::ClaimMsg {} => execute_claim(_deps, _env, _info),
    }
}

pub fn execute_claim(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let current_period = query_current_period(deps.as_ref(), _env.clone())?;
    let claimable_reward = query_claimable_reward(deps.as_ref(), _env.clone(), &_info.sender)
        .unwrap_or(ClaimableInfoResponse {
            claimable_reward: Uint128::zero(),
        });
    let sender = &_info.sender;
    // send token
    if !cfg.is_periodic {
        LAST_CLAIMED_PERIOD.save(deps.storage, sender, &current_period.current_period)?;
    }

    let transfer_msg = Cw20ExecuteMsg::Transfer {
        recipient: sender.to_string(),
        amount: claimable_reward.claimable_reward,
    };
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cfg.vesting_token.to_string(),
        msg: to_json_binary(&transfer_msg)?,
        funds: vec![],
    });
    Ok(Response::new().add_message(msg))
}

pub fn execute_fund_reward(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if _info.sender != cfg.vesting_token {
        return Err(ContractError::NotVestingToken {});
    }

    let sender = deps.api.addr_validate(&wrapper.sender)?;
    let amount = wrapper.amount;
    let msg: ReceiveMsg = from_json(&wrapper.msg)?;

    match msg {
        ReceiveMsg::Fund {} => {
            let total_reward : Uint128 = TOTAL_REWARD.load(deps.storage)?;
            TOTAL_REWARD.save(deps.storage, &(total_reward + amount))?;
            Ok(Response::new()
                .add_attribute("action", "fund_reward")
                .add_attribute("sender", sender)
                .add_attribute("amount", amount.to_string()))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_json_binary(&query_contract_info(_deps)?),
        QueryMsg::PeriodInfo {} => to_json_binary(&query_current_period(_deps, _env)?),
    }
}

pub fn query_current_period(deps: Deps, env: Env) -> Result<PeriodInfoResponse, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let block_time: u64 = env.block.time.seconds();
    let mut current_period = 0;
    if block_time > cfg.end_time {
        if cfg.is_periodic {
            let duration = cfg.end_time.checked_sub(cfg.start_time);
            current_period = duration.unwrap() / cfg.period;
        } else {
            return Err(ContractError::VestingEnded {});
        }
    } else if block_time < cfg.start_time {
        return Err(ContractError::VestingNotStarted {});
    } else {
        let duration = block_time.checked_sub(cfg.start_time);
        current_period = duration.unwrap() / cfg.period + 1;
    }

    Ok(PeriodInfoResponse { current_period })
}

pub fn query_claimable_reward(
    deps: Deps,
    env: Env,
    user: &Addr,
) -> Result<ClaimableInfoResponse, ContractError> {
    let current_period = query_current_period(deps, env)?;
    let cfg = CONFIG.load(deps.storage)?;
    let mut claimable_reward: Uint128 = Uint128::zero();
    if cfg.is_periodic {
        claimable_reward = Uint128::from(
            USER_PERIODIC_REWARD
                .may_load(deps.storage, &user)?
                .unwrap_or(0),
        );
    } else {
        let periodic_reward = USER_PERIODIC_REWARD
            .may_load(deps.storage, &user)?
            .unwrap_or(0);
        let last_claimed_period = LAST_CLAIMED_PERIOD
            .may_load(deps.storage, &user)?
            .unwrap_or(0);
        claimable_reward =
            Uint128::from((current_period.current_period - last_claimed_period) * periodic_reward);
    }
    Ok(ClaimableInfoResponse {
        claimable_reward: claimable_reward,
    })
}

pub fn query_contract_info(deps: Deps) -> Result<ContractInfoResponse, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ContractInfoResponse {
        vesting_token: cfg.vesting_token.to_string(),
        treasury: cfg.treasury.to_string(),
        start_time: cfg.start_time,
        end_time: cfg.end_time,
        period: cfg.period,
        is_periodic: cfg.is_periodic,
        owner: cfg.owner.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[track_caller]
    fn setup_test_case(
        deps: DepsMut,
        info: MessageInfo,
        vesting_token: &str,
        treasury: &str,
        start_time: u64,
        end_time: u64,
        period: u64,
        is_periodic: bool,
        owner: &str,
        users: Vec<String>,
        periodic_reward: Vec<u64>,
    ) -> Result<Response<Empty>, ContractError> {
        // Instantiate a contract with voters

        let instantiate_msg = InstantiateMsg {
            vesting_token: vesting_token.to_string(),
            treasury: treasury.to_string(),
            start_time,
            end_time,
            period,
            is_periodic,
            owner: owner.to_string(),
            users,
            periodic_reward,
        };
        instantiate(deps, mock_env(), info, instantiate_msg)
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &[]);
        let vesting_token = "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v".to_string();
        let treasury = "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v".to_string();
        let start_time = 5000;
        let end_time = 2000;
        let period = 100;
        let is_periodic = true;
        let owner = "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v".to_string();
        let users = vec!["terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v".to_string()];
        let periodic_reward = vec![100];
        let res = setup_test_case(
            deps.as_mut(),
            info,
            &vesting_token,
            &treasury,
            start_time,
            end_time,
            period,
            is_periodic,
            &owner,
            users,
            periodic_reward,
        )
        .unwrap();

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.vesting_token.to_string(), vesting_token);
        assert_eq!(config.treasury.to_string(), treasury);
        assert_eq!(config.start_time, start_time);
        assert_eq!(config.end_time, end_time);
        assert_eq!(config.period, period);
        assert_eq!(config.is_periodic, is_periodic);
        assert_eq!(config.owner.to_string(), owner);
    }
}
