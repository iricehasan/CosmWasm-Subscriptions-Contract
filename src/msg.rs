use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreatePlan {
        name: Option<String>,
        description: Option<String>,
        price: String,
        external_url: Option<String>,
        enable_freeze: bool,
        frequency: String,
    },
    UpdatePlan {
        id: u128,
    },
    RemovePlan {
        id: u128,
    },
    WithdrawPayments {
        id: u128,
    },
    FreezeSubscription { 
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