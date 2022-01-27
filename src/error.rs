use std::str::Utf8Error;

use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Utf(#[from] Utf8Error),

    #[error("No data in ReceiveMsg")]
    NoData {},

    #[error("Sender cannot be receiver!")]
    SenderIsReceiver {},

    #[error("Withdrawal not available")]
    CantWithdraw {},

    #[error("Wrong token collection")]
    WrongTokenCollection {},

    #[error("Wrong token received")]
    WrongToken {},

    #[error("Cannot claim!")]
    CannotClaim {},

    #[error("Wrong Action")]
    WrongAction {},

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Insufficient funds")]
    InsufficientFunds {},
}
