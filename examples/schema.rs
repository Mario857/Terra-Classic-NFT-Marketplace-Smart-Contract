use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use cw_one2one_nft::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ExchangeNft, CounterOfferNft};
use cw_one2one_nft::package::{ExchangesResponse, ContractInfoResponse, QueryExchangesResult, QueryCounterOffersResult};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ExchangeNft), &out_dir);
    export_schema(&schema_for!(CounterOfferNft), &out_dir);
    export_schema(&schema_for!(ExchangesResponse), &out_dir);
    export_schema(&schema_for!(ContractInfoResponse), &out_dir);
    export_schema(&schema_for!(QueryExchangesResult), &out_dir);
    export_schema(&schema_for!(QueryCounterOffersResult), &out_dir);
}
