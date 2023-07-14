use crate::consensus::ConsensusError;
use crate::state_transition::identity_topup_transition::IdentityTopUpTransition;
use crate::state_transition_action::identity::identity_topup::IdentityTopUpTransitionAction;
use crate::state_transition_action::identity::identity_topup::v0::IdentityTopUpTransitionActionV0;

impl IdentityTopUpTransitionAction {
    pub fn try_from(
        value: IdentityTopUpTransition,
        top_up_balance_amount: u64,
    ) -> Result<Self, ConsensusError> {
        match value {
            IdentityTopUpTransition::V0(v0) => {
                IdentityTopUpTransitionActionV0::try_from(v0, top_up_balance_amount).into()
            }
        }
    }

    pub fn try_from_borrowed(
        value: &IdentityTopUpTransition,
        top_up_balance_amount: u64,
    ) -> Result<Self, ConsensusError> {
        match value {
            IdentityTopUpTransition::V0(v0) => {
                IdentityTopUpTransitionActionV0::try_from_borrowed(v0, top_up_balance_amount).into()
            }
        }
    }
}