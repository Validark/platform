use crate::consensus::basic::BasicError;
use crate::consensus::basic::identity::IdentityAssetLockTransactionOutputNotFoundError;
use crate::consensus::ConsensusError;
use crate::state_transition::identity_topup_transition::IdentityTopUpTransitionV0;
use crate::state_transition_action::identity::identity_topup::v0::IdentityTopUpTransitionActionV0;

impl IdentityTopUpTransitionActionV0 {
    pub fn try_from(
        value: IdentityTopUpTransitionV0,
        top_up_balance_amount: u64,
    ) -> Result<Self, ConsensusError> {
        let IdentityTopUpTransitionV0 {
            identity_id,
            asset_lock_proof,
            ..
        } = value;
        let asset_lock_outpoint = asset_lock_proof
            .out_point()
            .ok_or(ConsensusError::BasicError(
                BasicError::IdentityAssetLockTransactionOutputNotFoundError(
                    IdentityAssetLockTransactionOutputNotFoundError::new(
                        asset_lock_proof.instant_lock_output_index().unwrap(),
                    ),
                ),
            ))?
            .into();
        Ok(IdentityTopUpTransitionActionV0 {
            top_up_balance_amount,
            identity_id,
            asset_lock_outpoint,
        })
    }

    pub fn try_from_borrowed(
        value: &IdentityTopUpTransitionV0,
        top_up_balance_amount: u64,
    ) -> Result<Self, ConsensusError> {
        let IdentityTopUpTransitionV0 {
            identity_id,
            asset_lock_proof,
            ..
        } = value;
        let asset_lock_outpoint = asset_lock_proof
            .out_point()
            .ok_or(ConsensusError::BasicError(
                BasicError::IdentityAssetLockTransactionOutputNotFoundError(
                    IdentityAssetLockTransactionOutputNotFoundError::new(
                        asset_lock_proof.instant_lock_output_index().unwrap(),
                    ),
                ),
            ))?
            .into();

        Ok(IdentityTopUpTransitionActionV0 {
            top_up_balance_amount,
            identity_id: *identity_id,
            asset_lock_outpoint,
        })
    }
}