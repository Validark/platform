use dpp::identity::state_transition::identity_credit_withdrawal_transition::IdentityCreditWithdrawalTransition;

use super::ValidateStateTransition;

impl ValidateStateTransition for IdentityCreditWithdrawalTransition {
    fn validate_type(&self) -> Result<dpp::validation::SimpleValidationResult, dpp::ProtocolError> {
        todo!()
    }

    fn validate_signature(
        &self,
    ) -> Result<dpp::validation::SimpleValidationResult, dpp::ProtocolError> {
        todo!()
    }

    fn validate_key_signature(
        &self,
    ) -> Result<dpp::validation::SimpleValidationResult, dpp::ProtocolError> {
        todo!()
    }

    fn validate_state(
        &self,
        drive: &drive::drive::Drive,
    ) -> Result<
        dpp::validation::ValidationResult<dpp::state_transition::StateTransitionAction>,
        dpp::ProtocolError,
    > {
        todo!()
    }
}
