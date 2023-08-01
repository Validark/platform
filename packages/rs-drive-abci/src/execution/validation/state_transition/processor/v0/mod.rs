use crate::error::Error;
use crate::execution::types::execution_event::ExecutionEvent;
use crate::execution::validation::state_transition::transformer::StateTransitionActionTransformerV0;
use crate::platform_types::platform::PlatformRef;
use crate::platform_types::platform_state::v0::PlatformStateV0Methods;
use crate::rpc::core::CoreRPCLike;
use dpp::identity::PartialIdentity;
use dpp::prelude::ConsensusValidationResult;
use dpp::serialization::Signable;
use dpp::state_transition::StateTransition;
use dpp::state_transition_action::StateTransitionAction;
use dpp::validation::SimpleConsensusValidationResult;
use dpp::version::{DefaultForPlatformVersion, PlatformVersion, TryIntoPlatformVersioned};
use drive::drive::Drive;
use drive::grovedb::TransactionArg;
use crate::error::execution::ExecutionError;
use crate::execution::types::state_transition_execution_context::StateTransitionExecutionContext;
use crate::execution::validation::state_transition::common::validate_state_transition_identity_signed::ValidateStateTransitionIdentitySignature;
use crate::execution::validation::state_transition::state_transitions::identity_update::identity_and_signatures::v0::IdentityUpdateStateTransitionIdentityAndSignaturesValidationV0;
use crate::execution::validation::state_transition::state_transitions::identity_create::identity_and_signatures::v0::IdentityCreateStateTransitionIdentityAndSignaturesValidationV0;
use crate::execution::validation::state_transition::state_transitions::identity_top_up::identity_retrieval::v0::IdentityTopUpStateTransitionIdentityRetrievalV0;

pub(in crate::execution) fn process_state_transition_v0<'a, C: CoreRPCLike>(
    platform: &'a PlatformRef<C>,
    state_transition: StateTransition,
    transaction: TransactionArg,
    platform_version: &PlatformVersion,
) -> Result<ConsensusValidationResult<ExecutionEvent<'a>>, Error> {
    let mut state_transition_execution_context =
        StateTransitionExecutionContext::default_for_platform_version(platform_version)?;
    // Validating structure
    let result = state_transition.validate_structure(
        platform.drive,
        platform.state.current_protocol_version_in_consensus(),
        transaction,
    )?;
    if !result.is_valid() {
        return Ok(ConsensusValidationResult::<ExecutionEvent>::new_with_errors(result.errors));
    }

    // Validating signatures
    let result = state_transition.validate_identity_and_signatures(
        platform.drive,
        transaction,
        &mut state_transition_execution_context,
        platform_version,
    )?;
    if !result.is_valid() {
        return Ok(ConsensusValidationResult::<ExecutionEvent>::new_with_errors(result.errors));
    }
    let maybe_identity = result.into_data()?;

    // Validating state
    let result = state_transition.validate_state(platform, transaction)?;

    result.map_result(|action| {
        ExecutionEvent::create_from_state_transition_action(action, maybe_identity, platform.state.epoch_ref(), platform_version)
    })
}

/// A trait for validating state transitions within a blockchain.
pub(crate) trait StateTransitionSignatureValidationV0 {
    /// Validates the identity and signatures of a transaction to ensure its authenticity.
    ///
    /// # Arguments
    ///
    /// * `drive` - A reference to the drive containing the transaction data.
    /// * `tx` - The transaction argument to be authenticated.
    /// * `execution_context` - A mutable reference to the StateTransitionExecutionContext that provides the context for validation.
    /// * `platform_version` - A reference to the PlatformVersion to be used for validation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with either:
    /// - `Ok(ConsensusValidationResult<Option<PartialIdentity>>)`: Indicates that the transaction has passed authentication, and the result contains an optional `PartialIdentity`.
    /// - `Err(Error)`: Indicates that the transaction failed authentication, and the result contains an `Error` indicating the reason for failure.
    ///
    fn validate_identity_and_signatures(
        &self,
        drive: &Drive,
        tx: TransactionArg,
        execution_context: &mut StateTransitionExecutionContext,
        platform_version: &PlatformVersion,
    ) -> Result<ConsensusValidationResult<Option<PartialIdentity>>, Error>;
}

/// A trait for validating state transitions within a blockchain.
pub(crate) trait StateTransitionStructureValidationV0 {
    /// Validates the structure of a transaction by checking its basic elements.
    ///
    /// # Arguments
    ///
    /// * `drive` - A reference to the drive containing the transaction data.
    /// * `tx` - The transaction argument to be checked.
    ///
    /// # Returns
    ///
    /// * `Result<SimpleConsensusValidationResult, Error>` - A result with either a SimpleConsensusValidationResult or an Error.
    fn validate_structure(
        &self,
        drive: &Drive,
        protocol_version: u32,
        tx: TransactionArg,
    ) -> Result<SimpleConsensusValidationResult, Error>;
}

/// A trait for validating state transitions within a blockchain.
pub(crate) trait StateTransitionStateValidationV0:
    StateTransitionActionTransformerV0
{
    /// Validates the state transition by analyzing the changes in the platform state after applying the transaction.
    ///
    /// # Arguments
    ///
    /// * `platform` - A reference to the platform containing the state data.
    /// * `tx` - The transaction argument to be applied.
    ///
    /// # Type Parameters
    ///
    /// * `C: CoreRPCLike` - A type constraint indicating that C should implement `CoreRPCLike`.
    ///
    /// # Returns
    ///
    /// * `Result<ConsensusValidationResult<StateTransitionAction>, Error>` - A result with either a ConsensusValidationResult containing a StateTransitionAction or an Error.
    fn validate_state<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        tx: TransactionArg,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error>;
}

impl StateTransitionStructureValidationV0 for StateTransition {
    fn validate_structure(
        &self,
        drive: &Drive,
        protocol_version: u32,
        tx: TransactionArg,
    ) -> Result<SimpleConsensusValidationResult, Error> {
        match self {
            StateTransition::DataContractCreate(st) => {
                st.validate_structure(drive, protocol_version, tx)
            }
            StateTransition::DataContractUpdate(st) => {
                st.validate_structure(drive, protocol_version, tx)
            }
            StateTransition::IdentityCreate(st) => {
                st.validate_structure(drive, protocol_version, tx)
            }
            StateTransition::IdentityUpdate(st) => {
                st.validate_structure(drive, protocol_version, tx)
            }
            StateTransition::IdentityTopUp(st) => {
                st.validate_structure(drive, protocol_version, tx)
            }
            StateTransition::IdentityCreditWithdrawal(st) => {
                st.validate_structure(drive, protocol_version, tx)
            }
            StateTransition::DocumentsBatch(st) => {
                st.validate_structure(drive, protocol_version, tx)
            }
            StateTransition::IdentityCreditTransfer(st) => {
                st.validate_structure(drive, protocol_version, tx)
            }
        }
    }
}

impl StateTransitionSignatureValidationV0 for StateTransition {
    fn validate_identity_and_signatures(
        &self,
        drive: &Drive,
        tx: TransactionArg,
        execution_context: &mut StateTransitionExecutionContext,
        platform_version: &PlatformVersion,
    ) -> Result<ConsensusValidationResult<Option<PartialIdentity>>, Error> {
        match self {
            StateTransition::DataContractCreate(_)
            | StateTransition::DataContractUpdate(_)
            | StateTransition::IdentityCreditWithdrawal(_)
            | StateTransition::DocumentsBatch(_)
            | StateTransition::IdentityCreditTransfer(_) => {
                //Basic signature verification
                Ok(self
                    .validate_state_transition_identity_signed(
                        drive,
                        false,
                        tx,
                        execution_context,
                        platform_version,
                    )?
                    .map(Some))
            }
            StateTransition::IdentityUpdate(st) => {
                match platform_version
                    .drive_abci
                    .validation_and_processing
                    .state_transitions
                    .identity_update_state_transition
                    .identity_signatures
                {
                    Some(0) => {
                        let signable_bytes: Vec<u8> = self.signable_bytes()?;
                        let mut validation_result = self
                            .validate_state_transition_identity_signed(
                                drive,
                                false,
                                tx,
                                execution_context,
                                platform_version,
                            )?;
                        let partial_identity = validation_result.data_as_borrowed()?;
                        let result = st.validate_identity_update_state_transition_signatures_v0(
                            signable_bytes,
                            partial_identity,
                        )?;
                        validation_result.merge(result);
                        Ok(validation_result.map(Some))
                    }
                    None => Err(Error::Execution(ExecutionError::VersionNotActive {
                        method: "identity update transition: validate_identity_and_signatures"
                            .to_string(),
                        known_versions: vec![0],
                    })),
                    Some(version) => {
                        Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                            method: "identity update transition: validate_identity_and_signatures"
                                .to_string(),
                            known_versions: vec![0],
                            received: version,
                        }))
                    }
                }
            }
            StateTransition::IdentityCreate(st) => {
                match platform_version
                    .drive_abci
                    .validation_and_processing
                    .state_transitions
                    .identity_create_state_transition
                    .identity_signatures
                {
                    Some(0) => {
                        let signable_bytes: Vec<u8> = self.signable_bytes()?;
                        Ok(st
                            .validate_identity_create_state_transition_signatures_v0(
                                signable_bytes,
                            )?
                            .map(|_| None))
                    }
                    None => Err(Error::Execution(ExecutionError::VersionNotActive {
                        method: "identity create transition: validate_identity_and_signatures"
                            .to_string(),
                        known_versions: vec![0],
                    })),
                    Some(version) => {
                        Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                            method: "identity create transition: validate_identity_and_signatures"
                                .to_string(),
                            known_versions: vec![0],
                            received: version,
                        }))
                    }
                }
            }
            StateTransition::IdentityTopUp(st) => {
                match platform_version
                    .drive_abci
                    .validation_and_processing
                    .state_transitions
                    .identity_top_up_state_transition
                    .identity_signatures
                {
                    // The validation of the signature happens on the state level
                    Some(0) => Ok(st
                        .retrieve_topped_up_identity(drive, tx, platform_version)?
                        .map(Some)),
                    None => Err(Error::Execution(ExecutionError::VersionNotActive {
                        method: "identity top up transition: validate_identity_and_signatures"
                            .to_string(),
                        known_versions: vec![0],
                    })),
                    Some(version) => {
                        Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                            method: "identity top up transition: validate_identity_and_signatures"
                                .to_string(),
                            known_versions: vec![0],
                            received: version,
                        }))
                    }
                }
            }
        }
    }
}

impl StateTransitionStateValidationV0 for StateTransition {
    fn validate_state<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        tx: TransactionArg,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error> {
        match self {
            StateTransition::DataContractCreate(st) => st.validate_state(platform, tx),
            StateTransition::DataContractUpdate(st) => st.validate_state(platform, tx),
            StateTransition::IdentityCreate(st) => st.validate_state(platform, tx),
            StateTransition::IdentityUpdate(st) => st.validate_state(platform, tx),
            StateTransition::IdentityTopUp(st) => st.validate_state(platform, tx),
            StateTransition::IdentityCreditWithdrawal(st) => st.validate_state(platform, tx),
            StateTransition::DocumentsBatch(st) => st.validate_state(platform, tx),
            StateTransition::IdentityCreditTransfer(st) => st.validate_state(platform, tx),
        }
    }
}
