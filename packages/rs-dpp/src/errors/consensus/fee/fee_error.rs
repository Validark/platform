use crate::errors::consensus::fee::balance_is_not_enough_error::BalanceIsNotEnoughError;
use crate::errors::consensus::ConsensusError;
use thiserror::Error;

use serde::{Deserialize, Serialize};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum FeeError {
    /*

    DO NOT CHANGE ORDER OF VARIANTS WITHOUT INTRODUCING OF NEW VERSION

    */
    #[error(transparent)]
    BalanceIsNotEnoughError(BalanceIsNotEnoughError),
}

impl From<FeeError> for ConsensusError {
    fn from(error: FeeError) -> Self {
        Self::FeeError(error)
    }
}
