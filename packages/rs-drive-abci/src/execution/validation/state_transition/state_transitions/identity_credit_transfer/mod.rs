mod identity_and_signatures;
mod state;
mod structure;

use dpp::identity::state_transition::identity_credit_transfer_transition::IdentityCreditTransferTransition;

use dpp::identity::PartialIdentity;
use dpp::state_transition::identity_credit_transfer_transition::IdentityCreditTransferTransition;
use dpp::version::PlatformVersion;
use dpp::{
    state_transition::StateTransitionAction,
    validation::{ConsensusValidationResult, SimpleConsensusValidationResult},
};
use dpp::state_transition_action::StateTransitionAction;

use drive::drive::Drive;
use drive::grovedb::TransactionArg;

use crate::error::execution::ExecutionError;
use crate::error::Error;
use crate::platform_types::platform::PlatformRef;
use crate::rpc::core::CoreRPCLike;

use crate::execution::validation::state_transition::identity_credit_transfer::identity_and_signatures::v0::StateTransitionIdentityAndSignaturesValidationV0;
use crate::execution::validation::state_transition::identity_credit_transfer::state::v0::StateTransitionStateValidationV0;
use crate::execution::validation::state_transition::identity_credit_transfer::structure::v0::StateTransitionStructureValidationV0;
use crate::execution::validation::state_transition::processor::v0::StateTransitionValidationV0;
use crate::execution::validation::state_transition::transformer::StateTransitionActionTransformerV0;
use crate::platform_types::platform_state::v0::PlatformStateV0Methods;

impl StateTransitionActionTransformerV0 for IdentityCreditTransferTransition {
    fn transform_into_action<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        _tx: TransactionArg,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error> {
        let platform_version =
            PlatformVersion::get(platform.state.current_protocol_version_in_consensus())?;
        match platform_version
            .drive_abci
            .validation_and_processing
            .state_transitions
            .identity_credit_transfer_state_transition
            .transform_into_action
        {
            0 => self.transform_into_action_v0(),
            version => Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                method: "identity credit transfer transition: transform_into_action".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}

impl StateTransitionValidationV0 for IdentityCreditTransferTransition {
    fn validate_structure(
        &self,
        _drive: &Drive,
        protocol_version: u32,
        _tx: TransactionArg,
    ) -> Result<SimpleConsensusValidationResult, Error> {
        let platform_version = PlatformVersion::get(protocol_version)?;
        match platform_version
            .drive_abci
            .validation_and_processing
            .state_transitions
            .identity_credit_transfer_state_transition
            .structure
        {
            0 => self.validate_structure_v0(),
            version => Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                method: "identity credit transfer transition: validate_structure".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }

    fn validate_identity_and_signatures(
        &self,
        drive: &Drive,
        protocol_version: u32,
        tx: TransactionArg,
    ) -> Result<ConsensusValidationResult<Option<PartialIdentity>>, Error> {
        let platform_version = PlatformVersion::get(protocol_version)?;
        match platform_version
            .drive_abci
            .validation_and_processing
            .state_transitions
            .identity_credit_transfer_state_transition
            .state
        {
            0 => self.validate_identity_and_signatures_v0(drive, tx),
            version => Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                method: "identity credit transfer transition: validate_identity_and_signatures"
                    .to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }

    fn validate_state<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        tx: TransactionArg,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error> {
        let platform_version =
            PlatformVersion::get(platform.state.current_protocol_version_in_consensus())?;
        match platform_version
            .drive_abci
            .validation_and_processing
            .state_transitions
            .identity_credit_transfer_state_transition
            .state
        {
            0 => self.validate_state_v0(platform, tx),
            version => Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                method: "identity credit transfer transition: validate_state".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}
