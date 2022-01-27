use cw721::Cw721ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ExchangeNfts { exchange_id: String },
    WithdrawAllReceiversNfts {},
    WithdrawAllExchangersNfts {},
    WithdrawReceiverNft { exchange_id: String },
    WithdrawExchangerNft { exchange_id: String },
    ReceiveNft(Cw721ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ExchangeNft {
    pub receiver_token_contract_address: String,
    pub receiver_token_id: String,
    pub receiver: String,
    pub action: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CounterOfferNft {
    pub exchange_id: String,
    pub action: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Action {
    pub action: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetExchanges {
        receiver: Option<String>,
        exchanger: Option<String>,
    },
    GetCompletedExchanges {
        receiver: Option<String>,
        exchanger: Option<String>,
    },

    GetCounterOffers {
        receiver: String,
    },
}
