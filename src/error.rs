use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Plan not found")]
    PlanNotFound {},

    #[error("Subscription not found")]
    NoSubscriptionFound {},

    #[error("Subscription Expired")]
    SubscriptionExpired {},

    #[error("Address has already subscribed to a plan")]
    AddressAlreadySubscribed {},

    #[error("Not enough Balance")]
    NotEnoughBalance {},

    #[error("Wrong amount send (price: price)")]
    WrongAmountSent { price: Uint128 },

    #[error("Cannot freeze subscription for this plan (id: id)")]
   CannotFreezeForThisPlan { id: u128 },

}