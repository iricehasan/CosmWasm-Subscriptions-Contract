use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

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
        freeze_right: u64, // in days
        frequency: u64,
    },
    UpdatePlan {
        id: u128,
        name: Option<String>,
        description: Option<String>,
        price: Option<Uint128>,
        external_url: Option<String>,
        freeze_right: Option<u64>, // in days
        frequency: Option<u64>,
    },
    RemovePlan {
        id: u128,
    },
    WithdrawPayments {
        id: u128,
        amount: Uint128,
    },
    FreezeSubscription { 
        id: u128,
        duration_day: u64,
    },
    Subscribe {
        id: u128,
    },
    PaySubscription {
        id: u128,
    },
    CancelSubscription {
        id: u128,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {

}