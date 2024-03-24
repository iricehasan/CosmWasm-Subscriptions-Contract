use std::vec;
use std::ops::Add;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg, Timestamp, Event, CosmosMsg, Uint256
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, PLAN_SEQ, Plan, PLANS, Subscriber, SUBSCRIBERS};


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let admin =  msg
    .admin
    .and_then(|addr_string| deps.api.addr_validate(addr_string.as_str()).ok())
    .unwrap_or(info.sender);

    let config = Config {
        admin: admin.clone(),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePlan { name, description, price, external_url, enable_freeze, frequency } => execute_create_plan(deps, env, info, name, description, price, external_url, enable_freeze, frequency),
        ExecuteMsg::RemovePlan { id } => execute_remove_plan(deps, env, info, id),
        ExecuteMsg::UpdatePlan { id, name, description, price, external_url, enable_freeze, frequency } => execute_update_plan(deps, env, info, id, name, description, price, external_url, enable_freeze, frequency),
        _ => todo!()
    }
}

pub fn execute_create_plan(
    deps: DepsMut, 
    _env: Env,
    info: MessageInfo,
    name: Option<String>,
    description: Option<String>,
    price: String,
    external_url: Option<String>,
    enable_freeze: bool,
    frequency: String,
) -> Result<Response, ContractError> {

    let mut plans: Vec<Plan> = Vec::new();

    let id = PLAN_SEQ.update::<_, cosmwasm_std::StdError>(deps.storage, |id| Ok(id.add(1)))?;

    let new_plan = Plan {
        id,
        creator: info.sender.to_string().clone(),
        price,
        name,
        description,
        external_url,
        subscribers: Vec::new(),
        enable_freeze,
        frequency,
        total_payments: Uint256::from(0u128),
    };

    plans.push(new_plan);

    PLANS.save(deps.storage, info.sender.to_string().as_bytes(), &plans)?;
    Ok(Response::new()
        .add_attribute("method", "execute_create_new_plan")
        .add_attribute("new_plan_id", id.to_string()))
    
}

pub fn execute_remove_plan(
    deps: DepsMut, 
    _env: Env,
    info: MessageInfo,
    id: u128,
) -> Result<Response, ContractError> {
    let mut plans = PLANS.may_load(deps.storage, info.sender.to_string().as_bytes())?.unwrap();

    if let Some(index) = plans.iter().position(|plan| plan.id == id) {
        let plan = &mut plans[index];
        if info.sender.to_string() != plan.creator {
            return Err(ContractError::Unauthorized {});
        }
        
        plans.remove(index);
    } else {
        return Err(ContractError::PlanNotFound {})
    };

    PLANS.save(deps.storage, info.sender.to_string().as_bytes(), &plans)?;
    Ok(Response::new()
        .add_attribute("method", "execute_remove_plan")
        .add_attribute("plan_id", id.to_string())
    )
}

pub fn execute_update_plan(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u128,
    name: Option<String>,
    description: Option<String>,
    price: Option<String>,
    external_url: Option<String>,
    enable_freeze: Option<bool>,
    frequency: Option<String>,
) -> Result<Response,ContractError> {
    let mut plans = PLANS.may_load(deps.storage, info.sender.to_string().as_bytes())?.unwrap();

    if let Some(index) = plans.iter().position(|plan| plan.id == id) {
        let plan = &mut plans[index];
        if info.sender.to_string() != plan.creator {
            return Err(ContractError::Unauthorized {});
        }

        let updated_plan = Plan {
            price: price.unwrap_or(plan.price.clone()),
            name: name.or(plan.name.clone()), // Option or
            description: description.or(plan.description.clone()),
            external_url: external_url.or(plan.external_url.clone()),
            enable_freeze: enable_freeze.unwrap_or(plan.enable_freeze.clone()),
            frequency: frequency.unwrap_or(plan.frequency.clone()),
            ..plan.clone()
        };

        *plan = updated_plan;

    } else {
        return Err(ContractError::PlanNotFound {})
    };

    PLANS.save(deps.storage, info.sender.to_string().as_bytes(), &plans)?;
    
    Ok(Response::new()
    .add_attribute("method", "execute_update_plan")
    .add_attribute("plan_id", id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {

    }
}

