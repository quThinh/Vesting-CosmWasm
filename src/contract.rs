#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, BlockInfo, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order,
    Response, StdResult,
};
// use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{ContractInfoResponse, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

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
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::default())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => to_json_binary(&query_contract_info(_deps)?),
    }
}

pub fn query_contract_info(deps: Deps) -> StdResult<ContractInfoResponse> {
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
        )
        .unwrap();

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.vesting_token, vesting_token);
        assert_eq!(config.treasury, treasury);
        assert_eq!(config.start_time, start_time);
        assert_eq!(config.end_time, end_time);
        assert_eq!(config.period, period);
        assert_eq!(config.is_periodic, is_periodic);
        assert_eq!(config.owner, owner);
    }
}
