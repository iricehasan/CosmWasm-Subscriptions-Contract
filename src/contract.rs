use std::collections::HashMap;
use std::vec;
use std::ops::Add;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Event, Uint128, BankMsg, coin
};
use cw_utils::must_pay;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryConfigResponse, QueryMsg, QueryPlanByIdResponse, QuerySubscriberResponse};
use crate::state::{Config, CONFIG, PLAN_SEQ, Plan, PLANS, Subscriber, SUBSCRIBERS};

pub const DENOM: &str = "untrn";

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
    PLAN_SEQ.save(deps.storage, &0u64)?;

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
        ExecuteMsg::CreatePlan { name, description, price, external_url, freeze_right_per_subscriber, frequency } => execute_create_plan(deps, env, info, name, description, price, external_url, freeze_right_per_subscriber, frequency),
        ExecuteMsg::RemovePlan { id } => execute_remove_plan(deps, env, info, id),
        ExecuteMsg::UpdatePlan { id, name, description, price, external_url, freeze_right_per_subscriber, frequency } => execute_update_plan(deps, env, info, id, name, description, price, external_url, freeze_right_per_subscriber, frequency),
        ExecuteMsg::Subscribe {id} => execute_subscribe(deps, env, info, id),
        ExecuteMsg::RenewSubscription { id } => execute_renew_subscription(deps, env, info, id),
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
    freeze_right_per_subscriber: u64,
    frequency: u64,
) -> Result<Response, ContractError> {

    let id = PLAN_SEQ.update::<_, cosmwasm_std::StdError>(deps.storage, |id| Ok(id.add(1)))?;

    let subscribers: HashMap<String, Subscriber> = HashMap::new();

    let new_plan = Plan {
        id,
        creator: info.sender.to_string().clone(),
        price,
        name,
        description,
        external_url,
        subscribers,
        freeze_right_per_subscriber,
        frequency,
        balance: Uint128::from(0u128),
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
    id: u64,
) -> Result<Response, ContractError> {
    let plan = PLANS.may_load(deps.storage, id)?.unwrap();

    if info.sender.to_string() != plan.creator {
        return Err(ContractError::Unauthorized {});
    }

    let mut msg = None; 

    if plan.balance != Uint128::from(0u128) {

        msg = Some(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(plan.balance.u128(), DENOM)],
        });
    }
    
    PLANS.remove(deps.storage,id);

    let mut res = Response::new()
        .add_attribute("method", "execute_remove_plan")
        .add_attribute("plan_id", id.to_string());

    if let Some(m) = msg {
        res = res.add_message(m);
    }

    Ok(res)
}

pub fn execute_update_plan(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u64,
    name: Option<String>,
    description: Option<String>,
    price: Option<Uint128>,
    external_url: Option<String>,
    freeze_right_per_subscriber: Option<u64>,
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
        freeze_right_per_subscriber: freeze_right_per_subscriber.unwrap_or(plan.freeze_right_per_subscriber.clone()),
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
    id: u64,
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
        address: info.sender.to_string(),
        total_payments,
        currently_registered_plan: id.to_string(),
        next_payment: env.block.time.plus_days(plan.frequency),
        left_freeze_right: plan.freeze_right_per_subscriber.clone(),
        is_expired: false,
    };

    plan.balance += amount;
    plan.subscribers.insert(info.sender.to_string().clone(), new_subscriber.clone());
    subscriber_key.save(deps.storage, &new_subscriber)?;
    PLANS.save(deps.storage, id, &plan)?;

    Ok(Response::new()
        .add_event(Event::new("Subscribed"))
        .add_attribute("plan", id.to_string().clone())
        .add_attribute("subscriber", info.sender.to_string()))

}

pub fn execute_renew_subscription(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
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

    if !subscriber.is_expired {
        return Err(ContractError::SubscriptionNotExpired { })
    }

    subscriber.is_expired = false;
    subscriber.total_payments.entry(id.to_string().clone())
    .and_modify(|existing_value| *existing_value += amount);

    subscriber.next_payment = env.block.time.plus_days(plan.frequency);
    subscriber_key.save(deps.storage, &subscriber)?;

    plan.balance += amount;
    plan.subscribers.entry(subscriber.address.clone())
    .and_modify(|existing_value| *existing_value = subscriber);
    PLANS.save(deps.storage, id, &plan)?;

    Ok(Response::new()
        .add_event(Event::new("Subscription Renewed"))
        .add_attribute("plan", id.to_string().clone())
        .add_attribute("subscriber", info.sender.to_string()))
    
}

pub fn execute_cancel_subscription(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    let subscriber_key = SUBSCRIBERS.key((info.sender.to_string().clone(), id));
    if subscriber_key.may_load(deps.storage)?.is_none() {
        return Err(ContractError::NoSubscriptionFound {});
    }

    let mut subscriber = subscriber_key.load(deps.storage)?;


    subscriber.is_expired = true;
    subscriber_key.save(deps.storage, &subscriber)?;
    // also modify plan
    
    plan.subscribers.entry(subscriber.address.clone())
    .and_modify(|existing_value| *existing_value = subscriber);
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
    id: u64,
) -> Result<Response, ContractError> {

    let amount = must_pay(&info, DENOM).unwrap();
    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    let subscriber_key = SUBSCRIBERS.key((info.sender.to_string().clone(), id));
        if subscriber_key.may_load(deps.storage)?.is_none() {
            return Err(ContractError::NoSubscriptionFound {});
        }

    let mut subscriber = subscriber_key.load(deps.storage)?;

    if subscriber.is_expired {
        return Err(ContractError::SubscriptionExpired { })
    }

    if env.block.time.seconds() > subscriber.next_payment.seconds() {
        subscriber.is_expired = true;

        plan.subscribers.entry(subscriber.address.clone())
        .and_modify(|existing_value| *existing_value = subscriber.clone());

        subscriber_key.save(deps.storage, &subscriber)?;
        PLANS.save(deps.storage, id, &plan)?;
        return Err(ContractError::SubscriptionExpired {})
    
    }

    if amount != plan.price {
        return Err(ContractError::WrongAmountSent { price: plan.price })
    }

    subscriber.total_payments.entry(id.to_string().clone())
    .and_modify(|existing_value| *existing_value += amount);

    subscriber.next_payment = env.block.time.plus_days(plan.frequency);
    subscriber_key.save(deps.storage, &subscriber)?;

    plan.subscribers.entry(subscriber.address.clone())
    .and_modify(|existing_value| *existing_value = subscriber);
    plan.balance += amount;
    PLANS.save(deps.storage, id, &plan)?;

    Ok(Response::new()
        .add_event(Event::new("Subscription payed"))
        .add_attribute("plan", id.to_string().clone())
        .add_attribute("subscriber", info.sender.to_string()))

}

pub fn execute_freeze_subscription(
    deps: DepsMut,
    env: Env,
    info: MessageInfo, 
    id: u64,
    duration_day: u64,
) -> Result<Response, ContractError> {

    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    let subscriber_key = SUBSCRIBERS.key((info.sender.to_string().clone(), id));
    if subscriber_key.may_load(deps.storage)?.is_none() {
        return Err(ContractError::NoSubscriptionFound {});
    }

    let mut subscriber = subscriber_key.load(deps.storage)?;
    
    if subscriber.is_expired {
        return Err(ContractError::SubscriptionExpired {})
    }

    if env.block.time.seconds() > subscriber.next_payment.seconds() {
        subscriber.is_expired = true;

        plan.subscribers.entry(subscriber.address.clone())
        .and_modify(|existing_value| *existing_value = subscriber.clone());

        subscriber_key.save(deps.storage, &subscriber)?;

        PLANS.save(deps.storage, id, &plan)?;
        return Err(ContractError::SubscriptionExpired {})
    }

    if plan.freeze_right_per_subscriber == 0u64 {
        return Err(ContractError::CannotFreezeForThisPlan { id: id})
    }

    if duration_day > plan.freeze_right_per_subscriber {
        return Err(ContractError::CannotFreezeForThisPlan { id: id })
    }

    subscriber.left_freeze_right = subscriber.left_freeze_right.checked_sub(duration_day).unwrap();

    subscriber.next_payment = env.block.time.plus_days(duration_day);

    subscriber_key.save(deps.storage, &subscriber)?;

    plan.subscribers.entry(subscriber.address.clone())
    .and_modify(|existing_value| *existing_value = subscriber);

    PLANS.save(deps.storage, id, &plan)?;

    Ok(Response::new()
        .add_event(Event::new("Subscription Freezed"))
        .add_attribute("plan", id.to_string().clone())
        .add_attribute("subsrciber", info.sender.to_string())
        .add_attribute("days", duration_day.to_string()))
}

pub fn execute_withdraw_payments(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u64,
    amount: Uint128,
) -> Result<Response, ContractError> {

    let mut plan = PLANS.may_load(deps.storage, id)?.unwrap();

    if info.sender.to_string() != plan.creator {
        return Err(ContractError::Unauthorized {});
    }

    if amount > plan.balance {
        return Err(ContractError::NotEnoughBalance {})
    }

    plan.balance -= amount;

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
        QueryMsg::QuerySubscriber { id, address } => to_json_binary(&query_subscriber(deps, id, address)?),
    }
}

pub fn query_plan_by_id(
    deps: Deps,
    id: u64,
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
        freeze_right_per_subscriber: plan.freeze_right_per_subscriber,
        frequency: plan.frequency,
        balance: plan.balance,
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

pub fn query_subscriber(
    deps: Deps,
    id: u64,
    address: String,
) -> StdResult<QuerySubscriberResponse> {
    let subscriber_key = SUBSCRIBERS.key((address.clone(), id));

    let subscriber = subscriber_key.load(deps.storage)?;


    Ok( QuerySubscriberResponse {
        address,
        total_payments: subscriber.total_payments, // total payments to a plan with ID
        currently_registered_plan: subscriber.currently_registered_plan, // plan ID
        next_payment: subscriber.next_payment,
        is_expired: subscriber.is_expired,
        left_freeze_right: subscriber.left_freeze_right,

    })
}