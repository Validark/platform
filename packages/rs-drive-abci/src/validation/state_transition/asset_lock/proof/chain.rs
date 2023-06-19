use crate::error::Error;
use crate::platform::PlatformRef;
use dpp::consensus::basic::identity::{
    IdentityAssetLockTransactionOutPointAlreadyExistsError,
    InvalidAssetLockProofCoreChainHeightError,
};
use dpp::dashcore::OutPoint;
use dpp::identity::state_transition::asset_lock_proof::chain::ChainAssetLockProof;
use dpp::identity::state_transition::asset_lock_proof::CHAIN_ASSET_LOCK_PROOF_SCHEMA_VALIDATOR;
use dpp::prelude::ConsensusValidationResult;
use dpp::validation::SimpleConsensusValidationResult;
use drive::query::TransactionArg;

/// Validate the structure of the chain asset lock proof
pub fn validate_structure(
    chain_asset_lock_proof: &ChainAssetLockProof,
) -> Result<SimpleConsensusValidationResult, Error> {
    let result = CHAIN_ASSET_LOCK_PROOF_SCHEMA_VALIDATOR
        .validate(
            &(chain_asset_lock_proof
                .to_cleaned_object()
                .expect("we don't hold unserializable structs")
                .try_into_validating_json()
                .expect("TODO")),
        )
        .expect("TODO: how jsonschema validation will ever fail?");

    Ok(result)
}

/// Validate the state of the chain asset lock proof
pub fn validate_state<C>(
    chain_asset_lock_proof: &ChainAssetLockProof,
    platform_ref: &PlatformRef<C>,
    transaction: TransactionArg,
) -> Result<SimpleConsensusValidationResult, Error> {
    let mut result = ConsensusValidationResult::default();

    if platform_ref.block_info.core_height < chain_asset_lock_proof.core_chain_locked_height {
        result.add_error(InvalidAssetLockProofCoreChainHeightError::new(
            chain_asset_lock_proof.core_chain_locked_height,
            platform_ref.block_info.core_height,
        ));

        return Ok(result);
    }

    // Make sure that asset lock isn't spent yet

    let is_already_spent = platform_ref
        .drive
        .has_asset_lock_outpoint(&chain_asset_lock_proof.out_point, transaction)?;

    if is_already_spent {
        let outpoint = OutPoint::from(chain_asset_lock_proof.out_point.to_buffer());

        result.add_error(IdentityAssetLockTransactionOutPointAlreadyExistsError::new(
            outpoint.txid,
            outpoint.vout as usize,
        ))
    }

    Ok(result)
}
