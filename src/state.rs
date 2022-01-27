use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::package::ContractInfoResponse;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Exchange {
    pub exchanger_token_contract_addr: CanonicalAddr, // address of exchangers CW721 token

    pub receiver_token_contract_addr: CanonicalAddr, // address of receiver CW721 token

    pub exchanger: CanonicalAddr, // person who initiates exchange

    pub exchanger_token_id: String, // token_id of exchange initiator

    pub receiver: CanonicalAddr, // person who is can accept exchange

    pub receiver_token_id: String, // address receiver of token required by exchange initiator

    pub exchange_time: Timestamp, // exchange time
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CounterOffer {
    pub exchange_id: String, // exchange if token_id is linked to

    pub counter_offer_time: Timestamp, // counter offer time

    pub sender: String,

    pub contract_addr: String,

    pub token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const EXCHANGES: Map<&str, Exchange> = Map::new("exchanges"); // exchanges created by exchanger (initiator)

pub const EXCHANGES_COUNT: Item<u64> = Item::new("num_exchanges"); 

pub const COUNTER_OFFERS: Map<&str, CounterOffer> = Map::new("counter_offers"); 

pub const COMPLETED_EXCHANGES: Map<&str, Exchange> = Map::new("completed_exchanges");

pub const COMPLETED_EXCHANGES_COUNT: Item<u64> = Item::new("num_completed_exchanges"); 

pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new("marketplace_info");

pub fn num_exchanges(storage: &dyn Storage) -> StdResult<u64> {
    Ok(EXCHANGES_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_exchanges(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = num_exchanges(storage)? + 1;

    EXCHANGES_COUNT.save(storage, &val)?;

    Ok(val)
}

pub fn num_completed_exchanges(storage: &dyn Storage) -> StdResult<u64> {
    Ok(COMPLETED_EXCHANGES_COUNT
        .may_load(storage)?
        .unwrap_or_default())
}

pub fn increment_completed_exchanges(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = num_completed_exchanges(storage)? + 1;

    COMPLETED_EXCHANGES_COUNT.save(storage, &val)?;

    Ok(val)
}