use std::str::from_utf8;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, CanonicalAddr, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Order, Pair, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{Action, CounterOfferNft, ExchangeNft, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::package::{
    ContractInfoResponse, CounterOffersResponse, ExchangesResponse, QueryCounterOffersResult,
    QueryExchangesResult,
};
use crate::state::{
    increment_completed_exchanges, increment_exchanges, Config, CounterOffer, Exchange,
    COMPLETED_EXCHANGES, CONFIG, CONTRACT_INFO, COUNTER_OFFERS, EXCHANGES,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-one2one-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    msg_info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let info = ContractInfoResponse { name: msg.name };

    let config = Config {
        owner: deps.api.addr_canonicalize(&msg_info.sender.to_string())?,
    };

    CONFIG.save(deps.storage, &config)?;

    CONTRACT_INFO.save(deps.storage, &info)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ExchangeNfts { exchange_id } => try_exchange(deps, env, info, exchange_id),
        ExecuteMsg::WithdrawAllExchangersNfts {} => {
            try_withdraw_all_exchangers_nfts(deps, env, info)
        }
        ExecuteMsg::WithdrawAllReceiversNfts {} => try_withdraw_all_receivers_nfts(deps, env, info),
        ExecuteMsg::WithdrawExchangerNft { exchange_id } => {
            try_exchanger_withdraw(deps, env, info, exchange_id)
        }
        ExecuteMsg::WithdrawReceiverNft { exchange_id } => {
            try_receiver_withdraw(deps, env, info, exchange_id)
        }
        ExecuteMsg::ReceiveNft(msg) => try_receive_router(deps, env, info, msg),
    }
}

pub fn try_receive_router(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: Action = from_binary(&rcv_msg.msg)?;

    match msg.action.as_str() {
        "receiver_receive" => try_receiver_receive(deps, env, info, rcv_msg),
        "exchanger_receive" => try_exchanger_receive(deps, env, info, rcv_msg),
        _ => Err(ContractError::WrongAction {}),
    }
}

pub fn try_exchanger_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ExchangeNft = from_binary(&rcv_msg.msg)?;

    // check if same token Id form same original contract is already on sale
    // get EXCHANGES_COUNT
    let id = increment_exchanges(deps.storage)?.to_string();

    // save exchange
    let exchange = Exchange {
        receiver_token_contract_addr: deps
            .api
            .addr_canonicalize(&msg.receiver_token_contract_address)?,

        exchanger_token_contract_addr: deps.api.addr_canonicalize(&info.sender.as_str())?,

        exchanger_token_id: rcv_msg.token_id,
        receiver_token_id: msg.receiver_token_id,

        receiver: deps.api.addr_canonicalize(&msg.receiver)?,
        exchanger: deps.api.addr_canonicalize(&rcv_msg.sender)?,

        exchange_time: env.block.time,
    };

    EXCHANGES.save(deps.storage, &id, &exchange)?;

    Ok(Response::new()
        .add_attribute("method", "exchange_initiate")
        .add_attribute("id", id)
        .add_attribute("exchanger", exchange.exchanger.to_string())
        .add_attribute("receiver", msg.receiver)
        .add_attribute("exchanger_token_id", &exchange.exchanger_token_id)
        .add_attribute("receiver_token_id", exchange.receiver_token_id)
        .add_attribute(
            "receiver_contract_addr",
            exchange.receiver_token_contract_addr.to_string(),
        )
        .add_attribute(
            "exchanger_contract_addr",
            exchange.exchanger_token_contract_addr.to_string(),
        ))
}

pub fn try_receiver_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: CounterOfferNft = from_binary(&rcv_msg.msg)?;

    let exchange_id = msg.exchange_id.clone();

    let exchange = EXCHANGES.load(deps.storage, &exchange_id)?;

    if exchange.receiver == exchange.exchanger {
        return Err(ContractError::SenderIsReceiver {})
    }

    if exchange.receiver_token_contract_addr
        != deps.api.addr_canonicalize(&info.sender.to_string())?
    {
        return Err(ContractError::WrongTokenCollection {});
    }

    if exchange.receiver_token_id != rcv_msg.token_id {
        return Err(ContractError::WrongToken {});
    }

    if exchange.receiver != deps.api.addr_canonicalize(&rcv_msg.sender.to_string())? {
        return Err(ContractError::Unauthorized {});
    }

    let counter_offer = CounterOffer {
        sender: rcv_msg.sender,

        exchange_id: exchange_id.clone(),

        contract_addr: deps
            .api
            .addr_humanize(&exchange.receiver_token_contract_addr)?.to_string(),

        token_id: exchange.receiver_token_id,

        counter_offer_time: env.block.time,
    };

    COUNTER_OFFERS.save(deps.storage, &exchange_id, &counter_offer)?;

    Ok(Response::new()
        .add_attribute("method", "counter_offer_receive")
        .add_attribute("counter_offer_id", &exchange_id)
        .add_attribute("token_id", &rcv_msg.token_id))
}

pub fn transfer_nft(
    deps: &DepsMut,
    contract_addr: CanonicalAddr,
    recipient: CanonicalAddr,
    token_id: String,
) -> Result<WasmMsg, ContractError> {
    let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
        recipient: deps.api.addr_humanize(&recipient)?.into_string(),
        token_id: token_id.clone(),
    };

    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&contract_addr)?.into_string(),
        msg: to_binary(&transfer_cw721_msg)?,
        funds: vec![],
    };

    Ok(exec_cw721_transfer)
}

pub fn try_exchange(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    exchange_id: String,
) -> Result<Response, ContractError> {
    let exchange = EXCHANGES.load(deps.storage, &exchange_id)?;

    let counter_offer = COUNTER_OFFERS.load(deps.storage, &exchange_id)?;

    if exchange.exchanger == deps.api.addr_canonicalize(info.sender.as_str())?
        || counter_offer.sender.as_str() == info.sender.as_str()
    {
        let exec_exchanger_cw721_transfer = transfer_nft(
            &deps,
            exchange.exchanger_token_contract_addr,
            exchange.receiver,
            exchange.exchanger_token_id,
        )?;

        let cw721_transfer_exchanger_cosmos_msg: Vec<CosmosMsg> =
            vec![exec_exchanger_cw721_transfer.into()];

        let exec_receiver_cw721_transfer = transfer_nft(
            &deps,
            exchange.receiver_token_contract_addr,
            exchange.exchanger,
            exchange.receiver_token_id,
        )?;

        let cw721_transfer_receivers_cosmos_msg: Vec<CosmosMsg> =
            vec![exec_receiver_cw721_transfer.into()];

        // get generated id
        let completed_exchanged_id = increment_completed_exchanges(deps.storage)?.to_string();

        // load completed exchange and save completed exchange. exchange variable is not used because of borrowing.
        let completed_exchange = EXCHANGES.load(deps.storage, &exchange_id)?;

        // save to completed exchanges
        COMPLETED_EXCHANGES.save(deps.storage, &completed_exchanged_id, &completed_exchange)?;

        // remove counter offers
        COUNTER_OFFERS.remove(deps.storage, &exchange_id);

        // remove old exchange from contract
        EXCHANGES.remove(deps.storage, &exchange_id);

        return Ok(Response::new()
            .add_messages(cw721_transfer_exchanger_cosmos_msg)
            .add_messages(cw721_transfer_receivers_cosmos_msg)
            .add_attribute("method", "exchange_tokens"));
    }

    Err(ContractError::CannotClaim {})
}

pub fn try_exchanger_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    exchange_id: String,
) -> Result<Response, ContractError> {
    let counter_offer_exists = COUNTER_OFFERS.load(deps.storage, &exchange_id).is_ok();

    // Do not allow withdrawals. Exchanger can only call exchange in that case. Or counter offer must be withdrawn by receiver.
    if counter_offer_exists {
        return Err(ContractError::CantWithdraw {});
    }

    let exchange = EXCHANGES.load(deps.storage, &exchange_id)?;
    if exchange.exchanger == deps.api.addr_canonicalize(&info.sender.as_str())? {
        let exchanger_withdraw = transfer_nft(
            &deps,
            exchange.exchanger_token_contract_addr,
            exchange.exchanger,
            exchange.exchanger_token_id,
        )?;

        let cw721_transfer_cosmos_msg: Vec<CosmosMsg> = vec![exchanger_withdraw.into()];

        // remove exchange from list
        EXCHANGES.remove(deps.storage, &exchange_id);

        return Ok(Response::new()
            .add_messages(cw721_transfer_cosmos_msg)
            .add_attribute("method", "exchanger_withdraw")
            .add_attribute("exchanger", info.sender)
            .add_attribute("exchange_id", exchange_id));
    }
    Err(ContractError::Unauthorized {})
}

pub fn try_receiver_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    exchange_id: String,
) -> Result<Response, ContractError> {
    let counter_offer = COUNTER_OFFERS.load(deps.storage, &exchange_id)?;

    if counter_offer.sender == info.sender.as_str() {
        let receiver_withdraw = transfer_nft(
            &deps,
            deps.api.addr_canonicalize(&counter_offer.contract_addr)?,
            deps.api.addr_canonicalize(&counter_offer.sender)?,
            counter_offer.token_id,
        )?;

        let cw721_transfer_cosmos_msg: Vec<CosmosMsg> = vec![receiver_withdraw.into()];

        COUNTER_OFFERS.remove(deps.storage, &exchange_id);

        return Ok(Response::new()
            .add_messages(cw721_transfer_cosmos_msg)
            .add_attribute("method", "receiver_withdraw")
            .add_attribute("receiver", info.sender)
            .add_attribute("exchange_id", exchange_id));
    }
    Err(ContractError::Unauthorized {})
}

pub fn try_withdraw_all_exchangers_nfts(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut response = Response::new().add_attribute("action", "withdraw_all_exchangers");

    if config.owner == deps.api.addr_canonicalize(&info.sender.as_str())? {
        let exchanges: StdResult<Vec<_>> = EXCHANGES
            .range(deps.storage, None, None, Order::Ascending)
            .collect();

        let exchanges_cloned = exchanges.unwrap().into_iter().clone();

        // Iterate trough exchanges and transfer back all tokens to owners.
        for (key, _value) in exchanges_cloned {
            let id = from_utf8(&key)?;

            let exchange = EXCHANGES.load(deps.storage, id)?;

            let exchanger_withdraw = transfer_nft(
                &deps,
                exchange.exchanger_token_contract_addr,
                exchange.exchanger,
                exchange.exchanger_token_id,
            )?;

            let cw721_transfer_cosmos_msg: Vec<CosmosMsg> = vec![exchanger_withdraw.into()];

            // remove exchange from list
            EXCHANGES.remove(deps.storage, id);

            response = response.add_messages(cw721_transfer_cosmos_msg);
        }

        return Ok(response);
    }

    Err(ContractError::Unauthorized {})
}

pub fn try_withdraw_all_receivers_nfts(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut response = Response::new().add_attribute("action", "withdraw_all_receivers");

    if config.owner == deps.api.addr_canonicalize(&info.sender.as_str())? {
        let counter_offers: StdResult<Vec<_>> = COUNTER_OFFERS
            .range(deps.storage, None, None, Order::Ascending)
            .collect();

        let counter_offers_cloned = counter_offers.unwrap().into_iter().clone();

        // Iterate trough exchanges and transfer back all tokens to owners.
        for (key, _value) in counter_offers_cloned {
            let id = from_utf8(&key)?;

            let counter_offer = COUNTER_OFFERS.load(deps.storage, id)?;

            let exchanger_withdraw = transfer_nft(
                &deps,
                deps.api.addr_canonicalize(&counter_offer.contract_addr)?,
                deps.api.addr_canonicalize(&counter_offer.sender)?,
                counter_offer.token_id,
            )?;

            let cw721_transfer_cosmos_msg: Vec<CosmosMsg> = vec![exchanger_withdraw.into()];

            // remove exchange from list
            COUNTER_OFFERS.remove(deps.storage, id);

            response = response.add_messages(cw721_transfer_cosmos_msg);
        }

        return Ok(response);
    }

    Err(ContractError::Unauthorized {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetExchanges {
            exchanger,
            receiver,
        } => to_binary(&query_exchanges(deps, exchanger, receiver)?),
        QueryMsg::GetCompletedExchanges {
            exchanger,
            receiver,
        } => to_binary(&query_completed_exchanges(deps, exchanger, receiver)?),
        QueryMsg::GetCounterOffers { receiver } => {
            to_binary(&query_counter_offers(deps, receiver)?)
        }
    }
}

fn query_completed_exchanges(
    deps: Deps,
    exchanger: Option<String>,
    receiver: Option<String>,
) -> StdResult<ExchangesResponse> {
    let res: StdResult<Vec<QueryExchangesResult>> = COMPLETED_EXCHANGES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|kv_item| parse_exchanges(deps.api, kv_item))
        .collect();

    let res_filtered = res?
        .into_iter()
        .filter(|a| {
            let exchanger_clone = exchanger.clone();
            let receiver_clone = receiver.clone();

            if exchanger_clone.is_some() || receiver_clone.is_some() {
                return a.exchanger == exchanger_clone.unwrap_or_default()
                    || a.receiver == receiver_clone.unwrap_or_default();
            }

            return true;
        })
        .collect();

    return Ok(ExchangesResponse {
        exchanges: res_filtered,
    });
}

fn query_counter_offers(deps: Deps, sender: String) -> StdResult<CounterOffersResponse> {
    let res: StdResult<Vec<QueryCounterOffersResult>> = COUNTER_OFFERS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|kv_item| parse_counter_offers(deps.api, kv_item))
        .collect();

    let res_filtered = res?
        .into_iter()
        .filter(|co| {
            return co.sender == sender;
        })
        .collect();

    return Ok(CounterOffersResponse {
        counter_offers: res_filtered,
    });
}

fn query_exchanges(
    deps: Deps,
    exchanger: Option<String>,
    receiver: Option<String>,
) -> StdResult<ExchangesResponse> {
    let res: StdResult<Vec<QueryExchangesResult>> = EXCHANGES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|kv_item| parse_exchanges(deps.api, kv_item))
        .collect();

    let res_filtered = res?
        .into_iter()
        .filter(|a| {
            // filter with optional parameters, if exchanger is defined or receiver use it as filter param else return all.
            let exchanger_clone = exchanger.clone();
            let receiver_clone = receiver.clone();

            if exchanger_clone.is_some() || receiver_clone.is_some() {
                return a.exchanger == exchanger_clone.unwrap_or_default()
                    || a.receiver == receiver_clone.unwrap_or_default();
            }

            return true;
        })
        .collect();

    return Ok(ExchangesResponse {
        exchanges: res_filtered,
    });
}

// Parse exchanges to human readable addresses and values.
fn parse_exchanges(
    api: &dyn Api,
    item: StdResult<Pair<Exchange>>,
) -> StdResult<QueryExchangesResult> {
    item.and_then(|(k, exchange)| {
        let id = from_utf8(&k)?;
        Ok(QueryExchangesResult {
            id: id.to_string(),
            exchanger_token_id: exchange.exchanger_token_id,
            receiver_token_id: exchange.receiver_token_id,
            exchanger_contract_addr: api
                .addr_humanize(&exchange.exchanger_token_contract_addr)?
                .to_string(),
            receiver_contract_addr: api
                .addr_humanize(&exchange.receiver_token_contract_addr)?
                .to_string(),
            exchanger: api.addr_humanize(&exchange.exchanger)?.to_string(),
            receiver: api.addr_humanize(&exchange.receiver)?.to_string(),
            exchange_time: exchange.exchange_time,
        })
    })
}

// Parse exchanges to human readable addresses and values.
fn parse_counter_offers(
    _api: &dyn Api,
    item: StdResult<Pair<CounterOffer>>,
) -> StdResult<QueryCounterOffersResult> {
    item.and_then(|(k, counter_offer)| {
        let id = from_utf8(&k)?;
        Ok(QueryCounterOffersResult {
            id: id.to_string(),
            token_id: counter_offer.token_id,
            contract_addr: counter_offer.contract_addr,
            sender: counter_offer.sender,
            exchange_id: id.to_string(),
            counter_offer_time: counter_offer.counter_offer_time,
        })
    })
}
