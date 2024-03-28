# Subscription Management Smart Contract

This is a smart contract written in Rust using CosmWasm for managing subscription-based services on the blockchain. It enables the creation, modification, and cancellation of subscription plans, as well as handling subscriber interactions such as subscription, renewal, payment, freezing, and withdrawal of payments. The contract also provides querying functionalities to retrieve plan details, configuration, and subscriber information.

# Features
- **Subscription Plan Management**: Create, update, and remove subscription plans with customizable attributes such as name, description, price, freeze rights, and frequency.
- **Subscriber Interactions**: Subscribers can subscribe to plans, renew subscriptions, pay subscription fees in a specified denomination, cancel subscriptions, and freeze subscriptions for a specified duration if plan has this feature.
- **Payment Handling**: Plan creator can withdraw accumulated payments in a specified denomination.
- **Query Functionalities**: Retrieve details about subscription plans by their ids, contract configuration, and subscriber information by address and subscribed plan id.

# Usage
## Creating a Subscription Plan
Use the execute_create_plan function to create a new subscription plan. Provide details such as name, description, price, URL, freeze rights per subscriber, and frequency.

## Subscribing to a Plan
To subscribe to a plan, call the execute_subscribe function with the plan ID. Ensure to send the correct amount of payment in the specified denomination.

## Renewing a Subscription
Renew a subscription using the execute_renew_subscription function by providing the plan ID. This function extends the subscription period and updates the payment details.

## Cancelling a Subscription
Cancel a subscription for a specific plan using the execute_cancel_subscription function. This function marks the subscription as expired and updates relevant information.

## Freezing a Subscription
Temporarily freeze a subscription by calling the execute_freeze_subscription function with the plan ID and duration in days. This function suspends the subscription for the specified period.

## Withdrawing Payments
The contract creator can withdraw accumulated payments in a specfic denomination using the execute_withdraw_payments function. Provide the plan ID and the amount to withdraw.

# Queries
Retrieve information from the contract using the following query functions:

- **Query Plan by ID**: Retrieve details about a subscription plan by its ID.
- **Query Configuration**: Get the current configuration of the contract, including the admin address.
- **Query Subscriber Information**: Retrieve details about a subscriber, including total payments, subscription status, and freeze rights.

# Messages

```rust
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
```