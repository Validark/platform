use dashcore_rpc::json::{ProTxHash, QuorumMasternodeListItem};
use dpp::consensus::basic::identity::IdentityInsufficientBalanceError;
use dpp::consensus::ConsensusError;
use dpp::identity::factory::IDENTITY_PROTOCOL_VERSION;
use dpp::identity::{Identity, KeyType, Purpose, SecurityLevel};
use dpp::platform_value::BinaryData;
use dpp::prelude::{Identifier, IdentityPublicKey};
use dpp::state_transition::StateTransition;
use dpp::validation::{
    ConsensusValidationResult, SimpleConsensusValidationResult, SimpleValidationResult,
    ValidationResult,
};
use drive::drive::block_info::BlockInfo;
use drive::error::Error::GroveDB;
use drive::fee::result::FeeResult;
use drive::grovedb::{Transaction, TransactionArg};
use std::collections::BTreeMap;
use tenderdash_abci::proto::abci::{ExecTxResult, RequestFinalizeBlock};

use crate::abci::AbciError;
use crate::block::{BlockExecutionContext, BlockStateInfo};
use crate::error::execution::ExecutionError;
use crate::error::Error;
use crate::execution::block_proposal::BlockProposal;
use crate::execution::execution_event::ExecutionResult::{
    ConsensusExecutionError, SuccessfulFreeExecution, SuccessfulPaidExecution,
};
use crate::execution::execution_event::{ExecutionEvent, ExecutionResult};
use crate::execution::fee_pools::epoch::EpochInfo;
use crate::platform::Platform;
use crate::rpc::core::CoreRPCLike;
use crate::validation::state_transition::validate_state_transition;

/// The outcome of the block execution, either by prepare proposal, or process proposal
pub struct BlockExecutionOutcome {
    /// The app hash, also known as the commit hash, this is the root hash of grovedb
    /// after the block has been executed
    pub app_hash: [u8; 32],
    /// The results of the execution of each transaction
    pub tx_results: Vec<ExecTxResult>,
}

/// The outcome of the finalization of the block
pub struct BlockFinalizationOutcome {
    /// The validation result of the finalization of the block.
    /// Errors here can happen if the block that we receive to be finalized isn't actually
    /// the one we expect, this could be a replay attack or some other kind of attack.
    pub validation_result: SimpleValidationResult<AbciError>,
}

impl From<SimpleValidationResult<AbciError>> for BlockFinalizationOutcome {
    fn from(validation_result: SimpleValidationResult<AbciError>) -> Self {
        BlockFinalizationOutcome { validation_result }
    }
}

impl<C> Platform<C>
where
    C: CoreRPCLike,
{
    pub(crate) fn validate_fees_of_event(
        &self,
        event: &ExecutionEvent,
        block_info: &BlockInfo,
        transaction: TransactionArg,
    ) -> Result<ConsensusValidationResult<FeeResult>, Error> {
        match event {
            ExecutionEvent::PaidDriveEvent {
                identity,
                operations,
            } => {
                let balance = identity.balance.ok_or(Error::Execution(
                    ExecutionError::CorruptedCodeExecution("partial identity info with no balance"),
                ))?;
                let estimated_fee_result = self
                    .drive
                    .apply_drive_operations(operations.clone(), false, block_info, transaction)
                    .map_err(Error::Drive)?;

                // TODO: Should take into account refunds as well
                if balance >= estimated_fee_result.total_base_fee() {
                    Ok(ConsensusValidationResult::new_with_data(
                        estimated_fee_result,
                    ))
                } else {
                    Ok(ConsensusValidationResult::new_with_data_and_errors(
                        estimated_fee_result,
                        vec![ConsensusError::IdentityInsufficientBalanceError(
                            IdentityInsufficientBalanceError {
                                identity_id: identity.id,
                                balance,
                            },
                        )],
                    ))
                }
            }
            ExecutionEvent::FreeDriveEvent { operations: _ } => Ok(
                ConsensusValidationResult::new_with_data(FeeResult::default()),
            ),
        }
    }

    pub(crate) fn execute_event(
        &self,
        event: ExecutionEvent,
        block_info: &BlockInfo,
        transaction: &Transaction,
    ) -> Result<ExecutionResult, Error> {
        //todo: we need to split out errors
        //  between failed execution and internal errors
        let validation_result =
            self.validate_fees_of_event(&event, block_info, Some(&transaction))?;
        match event {
            ExecutionEvent::PaidDriveEvent {
                identity,
                operations,
            } => {
                if validation_result.is_valid_with_data() {
                    let individual_fee_result = self
                        .drive
                        .apply_drive_operations(operations, true, block_info, Some(transaction))
                        .map_err(Error::Drive)?;

                    let balance_change =
                        individual_fee_result.into_balance_change(identity.id.to_buffer());

                    let outcome = self.drive.apply_balance_change_from_fee_to_identity(
                        balance_change.clone(),
                        Some(transaction),
                    )?;

                    // println!("State transition fees {:#?}", outcome.actual_fee_paid);
                    //
                    // println!(
                    //     "Identity balance {:?} changed {:#?}",
                    //     identity.balance,
                    //     balance_change.change()
                    // );

                    Ok(SuccessfulPaidExecution(
                        validation_result.into_data()?,
                        outcome.actual_fee_paid,
                    ))
                } else {
                    Ok(ConsensusExecutionError(
                        SimpleConsensusValidationResult::new_with_errors(validation_result.errors),
                    ))
                }
            }
            ExecutionEvent::FreeDriveEvent { operations } => {
                self.drive
                    .apply_drive_operations(operations, true, block_info, Some(transaction))
                    .map_err(Error::Drive)?;
                Ok(SuccessfulFreeExecution)
            }
        }
    }

    pub(crate) fn process_raw_state_transitions(
        &self,
        raw_state_transitions: &Vec<Vec<u8>>,
        block_info: &BlockInfo,
        transaction: &Transaction,
    ) -> Result<(FeeResult, Vec<ExecTxResult>), Error> {
        let state_transitions = StateTransition::deserialize_many(raw_state_transitions)?;

        let mut aggregate_fee_result = FeeResult::default();
        let exec_tx_results = state_transitions
            .into_iter()
            .map(|state_transition| {
                let state_transition_execution_event =
                    validate_state_transition(self, state_transition)?;

                let execution_result = if state_transition_execution_event.is_valid() {
                    let execution_event = state_transition_execution_event.into_data()?;
                    self.execute_event(execution_event, block_info, transaction)?
                } else {
                    ConsensusExecutionError(SimpleConsensusValidationResult::new_with_errors(
                        state_transition_execution_event.errors,
                    ))
                };
                if let SuccessfulPaidExecution(_, fee_result) = &execution_result {
                    aggregate_fee_result.checked_add_assign(fee_result.clone())?;
                }

                Ok(execution_result.into())
            })
            .collect::<Result<Vec<ExecTxResult>, Error>>()?;
        Ok((aggregate_fee_result, exec_tx_results))
    }

    /// Run a block proposal, either from process proposal, or prepare proposal
    pub fn run_block_proposal(
        &self,
        block_proposal: BlockProposal,
        transaction: &Transaction,
    ) -> Result<BlockExecutionOutcome, Error> {
        let BlockProposal {
            height,
            round,
            core_chain_locked_height,
            proposed_app_version,
            proposer_pro_tx_hash,
            validator_set_quorum_hash,
            block_time_ms,
            raw_state_transitions,
        } = block_proposal;
        // We start by getting the epoch we are in
        let genesis_time_ms = self.get_genesis_time(height, block_time_ms, &transaction)?;

        let state = self.state.read().unwrap();
        let previous_core_height = state.core_height();
        let previous_block_time_ms = state
            .last_committed_block_info
            .as_ref()
            .map(|block_info| block_info.time_ms);
        drop(state);
        // Init block execution context
        let block_state_info =
            BlockStateInfo::from_block_proposal(&block_proposal, previous_block_time_ms);

        let epoch_info =
            EpochInfo::from_genesis_time_and_block_info(genesis_time_ms, &block_state_info)?;

        //
        self.drive
            .update_validator_proposed_app_version(
                proposer_pro_tx_hash,
                proposed_app_version as u32,
                Some(transaction),
            )
            .map_err(|e| {
                Error::Execution(ExecutionError::UpdateValidatorProposedAppVersionError(e))
            })?;

        let block_info = block_state_info.to_block_info(epoch_info.current_epoch_index);
        // FIXME: we need to calculate total hpmns based on masternode list (or remove hpmn_count if not needed)
        let total_hpmns = self.config.quorum_size as u32;
        let mut block_execution_context = BlockExecutionContext {
            block_state_info,
            epoch_info: epoch_info.clone(),
            hpmn_count: total_hpmns,
        };

        // If last synced Core block height is not set instead of scanning
        // number of blocks for asset unlock transactions scan only one
        // on Core chain locked height by setting last_synced_core_height to the same value
        // FIXME: re-enable and implement
        // let last_synced_core_height = if request.last_synced_core_height == 0 {
        //     block_execution_context.block_info.core_chain_locked_height
        // } else {
        //     request.last_synced_core_height
        // };
        let last_synced_core_height = block_execution_context
            .block_state_info
            .core_chain_locked_height;

        self.update_broadcasted_withdrawal_transaction_statuses(
            last_synced_core_height,
            &block_execution_context,
            &transaction,
        )?;

        let unsigned_withdrawal_transaction_bytes = self
            .fetch_and_prepare_unsigned_withdrawal_transactions(
                validator_set_quorum_hash,
                &block_execution_context,
                &transaction,
            )?;

        let (block_fees, tx_results) =
            self.process_raw_state_transitions(&raw_state_transitions, &block_info, transaction)?;

        self.pool_withdrawals_into_transactions_queue(&block_execution_context, transaction)?;

        // while we have the state transitions executed, we now need to process the block fees

        // Process fees
        let process_block_fees_outcome = self.process_block_fees(
            &block_execution_context.block_state_info,
            &epoch_info,
            block_fees.into(),
            transaction,
        )?;

        // TODO: re-enable
        // self.update_masternode_identities(previous_core_height, core_chain_locked_height)?;

        let root_hash = self
            .drive
            .grove
            .root_hash(Some(transaction))
            .unwrap()
            .map_err(|e| Error::Drive(GroveDB(e)))?;

        block_execution_context.block_state_info.commit_hash = Some(root_hash);

        self.block_execution_context
            .write()
            .unwrap()
            .replace(block_execution_context);

        Ok(BlockExecutionOutcome {
            app_hash: root_hash,
            tx_results,
        })
    }

    /// Update the current quorums if the core_height changes
    pub fn update_state_cache_and_quorums(&self, block_info: BlockInfo) {
        let mut state_cache = self.state.write().unwrap();

        state_cache.last_committed_block_info = Some(block_info.clone());
    }

    /// Finalize the block, this first involves validating it, then if valid
    /// it is committed to the state
    pub fn finalize_block_proposal(
        &self,
        request_finalize_block: RequestFinalizeBlock,
        transaction: &Transaction,
    ) -> Result<BlockFinalizationOutcome, Error> {
        let RequestFinalizeBlock {
            commit,
            misbehavior,
            hash,
            height,
            round,
            block,
            block_id,
        } = request_finalize_block;

        // Retrieve block execution context before we do anything else
        let mut guarded_block_execution_context = self.block_execution_context.write().unwrap();
        let block_execution_context =
            guarded_block_execution_context
                .as_ref()
                .ok_or(Error::Execution(ExecutionError::CorruptedCodeExecution(
                    "block execution context must be set in block begin handler",
                )))?;

        let BlockExecutionContext {
            block_state_info,
            epoch_info,
            hpmn_count,
        } = &block_execution_context;

        let mut validation_result = SimpleValidationResult::<AbciError>::new_with_errors(vec![]);

        //// Verification that commit is for our current executed block
        // When receiving the finalized block, we need to make sure that info matches our current block

        // First let's check the basics, height, round and hash
        if !block_state_info.matches(height as u64, round as u32, hash)? {
            // we are on the wrong height or round
            validation_result.add_error(AbciError::WrongFinalizeBlockReceived(format!(
                "received a block for h: {} r: {}, expected h: {} r: {}",
                height, round, block_state_info.height, block_state_info.round
            )));
            return Ok(validation_result.into());
        }

        // Next we need to verify that the signature returned from the quorum is valid

        // todo: verify commit

        // Next let's check that the hash received is the same as the hash we expect

        if height == self.config.abci.genesis_height {
            self.drive.set_genesis_time(block_state_info.block_time_ms);
        }

        // Determine a new protocol version if enough proposers voted
        let changed_protocol_version = if epoch_info.is_epoch_change_but_not_genesis() {
            let mut state = self.state.write().unwrap();
            // Set current protocol version to the version from upcoming epoch
            state.current_protocol_version_in_consensus = state.next_epoch_protocol_version;

            // Determine new protocol version based on votes for the next epoch
            let maybe_new_protocol_version =
                self.check_for_desired_protocol_upgrade(*hpmn_count, &state, transaction)?;
            if let Some(new_protocol_version) = maybe_new_protocol_version {
                state.next_epoch_protocol_version = new_protocol_version;
            } else {
                state.next_epoch_protocol_version = state.current_protocol_version_in_consensus;
            }

            Some(state.current_protocol_version_in_consensus)
        } else {
            None
        };

        // At the end we update the state cache
        let block_info = block_state_info.to_block_info(epoch_info.current_epoch_index);

        self.update_state_cache_and_quorums(block_info);

        let mut drive_cache = self.drive.cache.write().unwrap();

        drive_cache.cached_contracts.clear_block_cache();

        Ok(validation_result.into())
    }

    /// Check a state transition to see if it should be added to mempool,
    /// This executes a few checks.
    /// It does validation on the state transition, and checks that the user is able to pay
    /// for the it. It can be wrong is rare cases, so the proposer needs to check transactions
    /// again before proposing his block.
    pub fn check_tx(
        &self,
        raw_tx: Vec<u8>,
    ) -> Result<ValidationResult<FeeResult, ConsensusError>, Error> {
        let state_transition =
            StateTransition::deserialize(raw_tx.as_slice()).map_err(Error::Protocol)?;
        let execution_event = validate_state_transition(&self, state_transition)?;

        // We should run the execution event in dry run to see if we would have enough fees for the transaction

        // We need the approximate block info
        let block_info_guard = self.state.read().unwrap();
        if let Some(block_info) = block_info_guard.last_committed_block_info.as_ref() {
            // We do not put the transaction, because this event happens outside of a block
            execution_event.and_then_borrowed_validation(|execution_event| {
                self.validate_fees_of_event(&execution_event, block_info, None)
            })
        } else {
            execution_event.and_then_borrowed_validation(|execution_event| {
                self.validate_fees_of_event(&execution_event, &BlockInfo::default(), None)
            })
        }
    }
}
