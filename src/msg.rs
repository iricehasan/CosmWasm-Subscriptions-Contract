use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Timestamp};
use crate::state::Subscriber;
use std::collections::HashMap;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreatePlan {
        name: Option<String>,
        description: Option<String>,
        price: Uint128,
        external_url: Option<String>,
        freeze_right_per_subscriber: u64, // in days
        frequency: u64,
    },
    UpdatePlan {
        id: u64,
        name: Option<String>,
        description: Option<String>,
        price: Option<Uint128>,
        external_url: Option<String>,
        freeze_right_per_subscriber: Option<u64>, // in days
        frequency: Option<u64>,
    },
    RemovePlan {
        id: u64,
    },
    WithdrawPayments {
        id: u64,
        amount: Uint128,
    },
    FreezeSubscription { 
        id: u64,
        duration_day: u64,
    },
    Subscribe {
        id: u64,
    },
    RenewSubscription {
        id: u64,
    },
    PaySubscription {
        id: u64,
    },
    CancelSubscription {
        id: u64,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(QueryPlanByIdResponse)]
    QueryPlanById { id: u64 },
    #[returns(QueryConfigResponse)]
    QueryConfig {},
    #[returns(QuerySubscriberResponse)]
    QuerySubscriber { id: u64, address: String},
}

#[cw_serde]
pub struct QueryPlanByIdResponse {
    pub id: u64,
    pub creator: String,
    pub price: Uint128,
    pub name: Option<String>,
    pub description: Option<String>,
    pub external_url: Option<String>,
    pub subscribers: HashMap<String, Subscriber>,
    pub freeze_right_per_subscriber: u64,
    pub frequency: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct QueryConfigResponse{
    pub admin: String,
}

#[cw_serde]
pub struct QuerySubscriberResponse {
    pub address: String,
    pub total_payments: HashMap<String, Uint128>, // total payments to a plan with ID
    pub currently_registered_plan: String, // plan ID
    pub next_payment: Timestamp,
    pub is_expired: bool,
    pub left_freeze_right: u64,
}