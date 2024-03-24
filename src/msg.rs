use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreatePlan {
    },
    UpdatePlan {
        id: u64,
    },
    RemovePlan {
        id: u64,
    },
    WithdrawPayments {
        id: u64,
    },
    FreezeSubscription { 
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

}