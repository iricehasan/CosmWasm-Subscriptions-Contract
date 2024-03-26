use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use std::collections::HashMap;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}

#[cw_serde]
pub struct Plan {
    pub id: u128,
    pub creator: String,
    pub price: Uint128,
    pub name: Option<String>,
    pub description: Option<String>,
    pub external_url: Option<String>,
    pub subscribers: Vec<Subscriber>,
    pub freeze_right: u64,
    pub frequency: u64,
    pub total_payments: Uint128,
}

#[cw_serde]
pub struct Subscriber {
    pub total_payments: HashMap<String, Uint128>, // total payments to a plan with ID
    pub currently_registered_plan: String, // plan ID
    pub next_payment: Timestamp,
    pub is_expired: bool,
    pub left_freeze_right: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PLAN_SEQ: Item<u128> = Item::new("plan_seq");
pub const PLANS: Map<u128, Plan> = Map::new("plans"); // addr as bytes
pub const SUBSCRIBERS: Map<(String, u128), Subscriber> = Map::new("subscribers"); // Addr and id keys