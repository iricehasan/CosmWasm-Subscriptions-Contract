use std::collections::HashMap;
use std::vec;
use std::ops::Add;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Timestamp, Event, Uint128, BankMsg, coin
};
use cw_utils::must_pay;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryConfigResponse, QueryMsg, QueryPlanByIdResponse};
use crate::state::{Config, CONFIG, PLAN_SEQ, Plan, PLANS, Subscriber, SUBSCRIBERS};

pub const DENOM: &str = "uatom";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
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
        ExecuteMsg::CreatePlan { name, description, price, external_url, freeze_right, frequency } => execute_create_plan(deps, env, info, name, description, price, external_url, freeze_right, frequency),
        ExecuteMsg::RemovePlan { id } => execute_remove_plan(deps, env, info, id),
        ExecuteMsg::UpdatePlan { id, name, description, price, external_url, freeze_right, frequency } => execute_update_plan(deps, env, info, id, name, description, price, external_url, freeze_right, frequency),
        ExecuteMsg::Subscribe {id} => execute_subscribe(deps, env, info, id),
        ExecuteMsg::PaySubscription { id } => execute_pay_subscription(deps, env, info, id),
        ExecuteMsg::CancelSubscription { id } => execute_cancel_subscription(deps, env, info, id),
        ExecuteMsg::FreezeSubscription { id, duration_day } => execute_freeze_subscription(deps, env, info, id, duration_day),
        ExecuteMsg::WithdrawPayments { id, amount } => execute_withdraw_payments(deps, env, info, id, amount),
    }
}

pub fn execute_create_plan(
    deps: DepsMut, 
    _env: Env,
    info: MessageInfo,
    name: Option<String>,
    description: Option<String>,
    price: Uint128,
    external_url: Option<String>,
    freeze_right: u64,
    frequency: u64,
) -> Result<Response, ContractError> {

    let id = PLAN_SEQ.update::<_, cosmwasm_std::StdError>(deps.storage, |id| Ok(id.add(1)))?;

    let new_plan = Plan {
        id,
        creator: info.sender.to_string().clone(),
        price,
        name,
        description,
        external_url,
        subscribers: Vec::new(),
        freeze_right,
        frequency,
        total_payments: Uint128::from(0u128),
    };


    PLANS.save(deps.storage, id, &new_plan)?;
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
    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    if info.sender.to_string() != plan.creator {
        return Err(ContractError::Unauthorized {});
    }
    
    PLANS.remove(deps.storage,id);

    Ok(Response::new()
        .add_attribute("method", "execute_remove_plan")
        .add_attribute("plan_id", id.to_string())
    )
}

pub fn execute_update_plan(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u128,
    name: Option<String>,
    description: Option<String>,
    price: Option<Uint128>,
    external_url: Option<String>,
    freeze_right: Option<u64>,
    frequency: Option<u64>,
) -> Result<Response,ContractError> {
    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    if info.sender.to_string() != plan.creator {
        return Err(ContractError::Unauthorized {});
    }

    let updated_plan = Plan {
        price: price.unwrap_or(plan.price.clone()),
        name: name.or(plan.name.clone()), // Option or
        description: description.or(plan.description.clone()),
        external_url: external_url.or(plan.external_url.clone()),
        freeze_right: freeze_right.unwrap_or(plan.freeze_right.clone()),
        frequency: frequency.unwrap_or(plan.frequency.clone()),
        ..plan.clone()
    };

    plan = updated_plan;

    PLANS.save(deps.storage, id, &plan)?;
    
    Ok(Response::new()
    .add_attribute("method", "execute_update_plan")
    .add_attribute("plan_id", id.to_string()))
}

pub fn execute_subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u128,
) -> Result<Response, ContractError> {

    let amount = must_pay(&info, DENOM).unwrap();
    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    let subscriber_key = SUBSCRIBERS.key((info.sender.to_string().clone(), id));
        if subscriber_key.may_load(deps.storage)?.is_some() {
            return Err(ContractError::AddressAlreadySubscribed {});
        }

    if amount != plan.price {
        return Err(ContractError::WrongAmountSent { price: plan.price })
    }

    let mut total_payments: HashMap<String, Uint128> = HashMap::new();
    total_payments.insert(id.to_string(), amount);

    let new_subscriber= Subscriber {
        total_payments,
        currently_registered_plan: id.to_string(),
        next_payment: env.block.time.plus_seconds(plan.frequency),
        left_freeze_right: plan.freeze_right.clone(),
        is_expired: false,
    };

    plan.subscribers.push(new_subscriber.clone());
    subscriber_key.save(deps.storage, &new_subscriber)?;
    PLANS.save(deps.storage, id, &plan)?;

    Ok(Response::new()
        .add_event(Event::new("Subcribed"))
        .add_attribute("plan", id.to_string().clone())
        .add_attribute("subsrciber", info.sender.to_string()))

}

pub fn execute_cancel_subscription(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u128,
) -> Result<Response, ContractError> {
    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    let subscriber_key = SUBSCRIBERS.key((info.sender.to_string().clone(), id));
        if subscriber_key.may_load(deps.storage)?.is_none() {
            return Err(ContractError::NoSubscriptionFound {});
        }

    let mut subscriber = subscriber_key.load(deps.storage)?;


    subscriber.is_expired = false;

    // remove subscribers from plan
    if let Some(s_index) = plan.subscribers.iter().position(|sub| sub == &subscriber) {

        plan.subscribers.remove(s_index);
    }

    subscriber_key.save(deps.storage, &subscriber)?;
    PLANS.save(deps.storage, id, &plan)?;

    Ok(Response::new()
        .add_event(Event::new("Unsubscribed"))
        .add_attribute("plan", id.to_string())
        .add_attribute("address", info.sender.to_string()))
}

pub fn execute_pay_subscription(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u128,
) -> Result<Response, ContractError> {

    let amount = must_pay(&info, DENOM).unwrap();
    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    let subscriber_key = SUBSCRIBERS.key((info.sender.to_string().clone(), id));
        if subscriber_key.may_load(deps.storage)?.is_none() {
            return Err(ContractError::NoSubscriptionFound {});
        }

    let mut subscriber = subscriber_key.load(deps.storage)?;


    if amount != plan.price {
        return Err(ContractError::WrongAmountSent { price: plan.price })
    }

    if env.block.time.seconds() > subscriber.next_payment.seconds() {
        subscriber.is_expired = true;
        subscriber_key.save(deps.storage, &subscriber)?;
        return Err(ContractError::SubscriptionExpired {})
    
    }

    subscriber.total_payments.entry(id.to_string().clone())
    .and_modify(|existing_value| *existing_value += amount);

    subscriber.next_payment = env.block.time.plus_seconds(plan.frequency);

    subscriber_key.save(deps.storage, &subscriber)?;
    PLANS.save(deps.storage, id, &plan)?;

    Ok(Response::new()
        .add_event(Event::new("Subcribed"))
        .add_attribute("plan", id.to_string().clone())
        .add_attribute("subsrciber", info.sender.to_string()))

}


pub fn execute_freeze_subscription(
    deps: DepsMut,
    env: Env,
    info: MessageInfo, 
    id: u128,
    duration_day: u64,
) -> Result<Response, ContractError> {

    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    let subscriber_key = SUBSCRIBERS.key((info.sender.to_string().clone(), id));
    if subscriber_key.may_load(deps.storage)?.is_none() {
        return Err(ContractError::NoSubscriptionFound {});
    }

    let mut subscriber = subscriber_key.load(deps.storage)?;

    if env.block.time.seconds() > subscriber.next_payment.seconds() {
        subscriber.is_expired = true;
        subscriber_key.save(deps.storage, &subscriber)?;
        return Err(ContractError::SubscriptionExpired {})
    }

    if plan.freeze_right == 0u64 {
        return Err(ContractError::CannotFreezeForThisPlan { id: id})
    }

    if duration_day > plan.freeze_right {
        return Err(ContractError::CannotFreezeForThisPlan { id: id })
    }

    subscriber.left_freeze_right -= duration_day;

    subscriber.next_payment = env.block.time.plus_days(duration_day);

    subscriber_key.save(deps.storage, &subscriber)?;
    PLANS.save(deps.storage, id, &plan)?;

    Ok(Response::new()
        .add_event(Event::new("Subscription Freezed"))
        .add_attribute("plan", id.to_string().clone())
        .add_attribute("subsrciber", info.sender.to_string())
        .add_attribute("days", duration_day.to_string()))
}

pub fn execute_withdraw_payments(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u128,
    amount: Uint128,
) -> Result<Response, ContractError> {

    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    if info.sender.to_string() != plan.creator {
        return Err(ContractError::Unauthorized {});
    }

    if amount > plan.total_payments {
        return Err(ContractError::NotEnoughBalance {})
    }
    plan.total_payments -= amount;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![coin(amount.u128(), DENOM)],
    };

    PLANS.save(deps.storage, id, &plan)?;

    Ok(Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("amount", amount)
        .add_message(msg))

}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryPlanById { id } => to_json_binary(&query_plan_by_id(deps,id)?),
        QueryMsg::QueryConfig {  } => to_json_binary(&query_config(deps)?),
    }
}

pub fn query_plan_by_id(
    deps: Deps,
    id: u128,
) -> StdResult<QueryPlanByIdResponse> {
    let plan = PLANS.may_load(deps.storage, id)?.unwrap();

    Ok(QueryPlanByIdResponse{
        id: plan.id,
        creator: plan.creator,
        price: plan.price,
        name: plan.name,
        description: plan.description,
        external_url: plan.external_url,
        subscribers: plan.subscribers,
        freeze_right: plan.freeze_right,
        frequency: plan.frequency,
        total_payments: plan.total_payments,
    })

}

pub fn query_config(
    deps: Deps,
) -> StdResult<QueryConfigResponse> {

    let config = CONFIG.load(deps.storage)?;

    Ok(QueryConfigResponse {
        admin: config.admin.to_string(),
    })
}
