use cosmwasm_std::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContractInfoResponse {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct QueryExchangesResult {
    pub id: String,
    pub exchanger_token_id: String,
    pub receiver_token_id: String,
    pub exchanger: String,
    pub receiver: String,
    pub exchanger_contract_addr: String,
    pub receiver_contract_addr: String,
    pub exchange_time: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct QueryCounterOffersResult {
    pub id: String,
    pub exchange_id: String,
    pub counter_offer_time: Timestamp,
    pub sender: String,
    pub contract_addr: String,
    pub token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangesResponse {
    pub exchanges: Vec<QueryExchangesResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CounterOffersResponse {
    pub counter_offers: Vec<QueryCounterOffersResult>,
}
