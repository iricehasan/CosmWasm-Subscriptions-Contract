use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint256};
use cw_storage_plus::{Item, Map};
use std::collections::HashMap;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}

#[cw_serde]
pub struct Plan {
    pub id: String,
    pub creator: String,
    pub price: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub external_url: Option<String>,
    pub subscriber: Vec<Subscriber>,
    pub enable_freeze: bool,
    pub frequency: Timestamp,
    pub total_payments: Uint256,
}

#[cw_serde]
pub struct Subscriber {
    total_payments: HashMap<String, Uint256>, // total payments to a plan with ID
    currently_registered_plan: String, // plan ID
    next_payment: Timestamp,
    is_expired: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PLANS: Map<String, Vec<Plan>> = Map::new("plans");
pub const SUBSCRIBERS: Map<String, Vec<Subscriber>> = Map::new("subscribers");