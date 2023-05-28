use crate::error::execution::ExecutionError;
use crate::error::Error;
use crate::platform::Platform;
use crate::rpc::core::CoreRPCLike;
use anyhow::{anyhow, bail, Result as AnyResult};
use dashcore_rpc::dashcore::anyhow::Result;
use dashcore_rpc::dashcore::hashes::Hash;
use dashcore_rpc::dashcore::{InstantLock, ProTxHash};
use dpp::async_trait::async_trait;
use dpp::block::block_info::BlockInfo;
use dpp::block::epoch::Epoch;
use dpp::dashcore::hashes::hex::FromHex;
use dpp::dashcore::Txid;
use dpp::data_contract::DataContract;
use dpp::document::{Document, ExtendedDocument};
use dpp::identifier::Identifier;
use dpp::identity::{Identity, IdentityPublicKey, KeyID};
use dpp::platform_value::{Bytes36, Value};
use dpp::prelude::{Revision, TimestampMillis};
use dpp::state_repository::{FetchTransactionResponse, StateRepositoryLike};
use dpp::state_transition::state_transition_execution_context::StateTransitionExecutionContext;
use drive::query::DriveQuery;
use itertools::Itertools;
use serde::Deserialize;
use std::convert::Infallible;

#[async_trait(?Send)]
impl<C> StateRepositoryLike for Platform<C>
where
    C: CoreRPCLike,
{
    type ConversionError = Infallible;
    type FetchDataContract = DataContract;
    type FetchDocument = Document;
    type FetchExtendedDocument = ExtendedDocument;
    type FetchIdentity = Identity;
    type FetchTransaction = FetchTransactionResponse;

    fn fetch_data_contract<'c>(
        &self,
        data_contract_id: &Identifier,
        execution_context: Option<&'c StateTransitionExecutionContext>,
    ) -> AnyResult<Option<Self::FetchDataContract>> {
        //todo: deal with execution context
        let Some(contract_fetch_info) = self.drive.get_contract_with_fetch_info(
            data_contract_id.to_buffer(),
            true, // todo
            None, // todo
        )? else {
            return Ok(None);
        };

        Ok(Some(contract_fetch_info.contract.clone())) // TODO Other ways rather than clone?
    }

    fn create_data_contract(
        &self,
        data_contract: DataContract,
        execution_context: Option<&StateTransitionExecutionContext>,
    ) -> AnyResult<()> {
        unreachable!()
    }

    fn update_data_contract(
        &self,
        data_contract: DataContract,
        execution_context: Option<&StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn fetch_documents<'c>(
        &self,
        contract_id: &Identifier,
        data_contract_type: &str,
        where_query: Value,
        execution_context: Option<&'c StateTransitionExecutionContext>,
    ) -> AnyResult<Vec<Self::FetchDocument>> {
        let guarded_block_execution_context = self.block_execution_context.read().unwrap();
        let block_execution_context =
            guarded_block_execution_context
                .as_ref()
                .ok_or(anyhow!("there should be an execution context when calling fetch_data_contract from dpp via state repository"))?;

        let contract_fetch_info = self
            .drive
            .get_contract_with_fetch_info(
                contract_id.to_buffer(),
                true, // todo
                None, // todo
            )?
            .ok_or(anyhow!("the contract should exist when fetching documents"))?;

        let contract = &contract_fetch_info.contract;
        let document_type = contract
            .document_type_for_name(data_contract_type)
            .map_err(|_| {
                anyhow!("the contract document type should exist when fetching documents")
            })?;

        let drive_query =
            DriveQuery::from_value(where_query, contract, document_type, &self.drive.config)?;

        //todo: deal with fees
        let epoch = Epoch::new(block_execution_context.epoch_info.current_epoch_index)?;

        let documents = self.drive.query_documents(
            drive_query,
            Some(&epoch),
            false, // todo
            None,  // todo
        )?;

        Ok(documents.documents)
    }

    fn fetch_extended_documents<'a>(
        &self,
        contract_id: &Identifier,
        data_contract_type: &str,
        where_query: Value,
        execution_context: Option<&'a StateTransitionExecutionContext>,
    ) -> AnyResult<Vec<Self::FetchExtendedDocument>> {
        let state = self.state.read().unwrap();
        let guarded_block_execution_context = self.block_execution_context.read().unwrap();
        let block_execution_context =
            guarded_block_execution_context
                .as_ref()
                .ok_or(anyhow!("there should be an execution context when calling fetch_data_contract from dpp via state repository"))?;

        let contract_fetch_info = self
            .drive
            .get_contract_with_fetch_info(
                contract_id.to_buffer(),
                true, // todo
                None, // todo
            )?
            .ok_or(anyhow!("the contract should exist when fetching documents"))?;

        let contract = &contract_fetch_info.contract;
        let document_type = contract
            .document_type_for_name(data_contract_type)
            .map_err(|_| {
                anyhow!("the contract document type should exist when fetching documents")
            })?;

        let drive_query =
            DriveQuery::from_value(where_query, contract, document_type, &self.drive.config)?;

        //todo: deal with fees
        let epoch = Epoch::new(block_execution_context.epoch_info.current_epoch_index)?;

        let documents = self.drive.query_documents(
            drive_query,
            Some(&epoch),
            false, // todo
            None,  // todo
        )?;

        let extended_documents = documents
            .documents
            .into_iter()
            .map(|document| {
                ExtendedDocument::from_document_with_additional_info(
                    document,
                    contract.clone(),
                    data_contract_type.to_string(),
                    state.current_protocol_version_in_consensus,
                )
            })
            .collect();

        Ok(extended_documents)
    }

    fn create_document<'a>(
        &self,
        document: &ExtendedDocument,
        execution_context: Option<&'a StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn update_document<'a>(
        &self,
        document: &ExtendedDocument,
        execution_context: Option<&'a StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn remove_document<'a>(
        &self,
        data_contract: &DataContract,
        data_contract_type: &str,
        document_id: &Identifier,
        execution_context: Option<&'a StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn fetch_transaction<'c>(
        &self,
        id: &str,
        execution_context: Option<&'c StateTransitionExecutionContext>,
    ) -> AnyResult<Self::FetchTransaction> {
        let tx_id = Txid::from_hex(id)?;

        // TODO: we need to handle errors (not found, etc.)

        self.core_rpc
            .get_transaction_extended_info(&tx_id)
            .map(|tx_result| FetchTransactionResponse {
                height: tx_result.blockindex,
                data: Some(tx_result.hex),
            })
            .map_err(|e| anyhow!("error fetching transaction: {}", e))
    }

    fn fetch_identity<'c>(
        &self,
        id: &Identifier,
        execution_context: Option<&'c StateTransitionExecutionContext>,
    ) -> AnyResult<Option<Self::FetchIdentity>> {
        self.drive
            .fetch_full_identity(
                id.to_buffer(),
                None, // todo
            )
            .map_err(Into::into)
    }

    fn create_identity<'a>(
        &self,
        identity: &Identity,
        execution_context: Option<&'a StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn add_keys_to_identity<'a>(
        &self,
        identity_id: &Identifier,
        keys: &[IdentityPublicKey],
        execution_context: Option<&'a StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn disable_identity_keys<'a>(
        &self,
        identity_id: &Identifier,
        keys: &[KeyID],
        disable_at: TimestampMillis,
        execution_context: Option<&'a StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn update_identity_revision<'a>(
        &self,
        identity_id: &Identifier,
        revision: Revision,
        execution_context: Option<&'a StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn fetch_identity_balance<'c>(
        &self,
        identity_id: &Identifier,
        execution_context: Option<&'c StateTransitionExecutionContext>,
    ) -> AnyResult<Option<u64>> {
        let guarded_block_execution_context = self.block_execution_context.read().unwrap();
        let block_execution_context =
            guarded_block_execution_context
                .as_ref()
                .ok_or(anyhow!("there should be an execution context when calling fetch_data_contract from dpp via state repository"))?;

        let epoch = Epoch::new(block_execution_context.epoch_info.current_epoch_index)?;

        let block_info = block_execution_context
            .block_state_info
            .to_block_info(epoch);

        let (maybe_balance, fee_result) = self
            .drive
            .fetch_identity_balance_with_costs(
                identity_id.to_buffer(),
                &block_info,
                true, // todo
                None, // todo
            )
            .map_err(|e| anyhow!("error fetching identity balance: {}", e))?;

        Ok(maybe_balance)
    }

    fn fetch_identity_balance_with_debt<'c>(
        &self,
        identity_id: &Identifier,
        execution_context: Option<&'c StateTransitionExecutionContext>,
    ) -> AnyResult<Option<i64>> {
        let guarded_block_execution_context = self.block_execution_context.read().unwrap();
        let block_execution_context =
            guarded_block_execution_context
                .as_ref()
                .ok_or(anyhow!("there should be an execution context when calling fetch_data_contract from dpp via state repository"))?;

        let epoch = Epoch::new(block_execution_context.epoch_info.current_epoch_index)?;

        let block_info = block_execution_context
            .block_state_info
            .to_block_info(epoch);

        let (maybe_balance, fee_result) = self
            .drive
            .fetch_identity_balance_include_debt_with_costs(
                identity_id.to_buffer(),
                &block_info,
                true, // todo
                None, // todo
            )
            .map_err(|e| anyhow!("error fetching identity balance with debt: {}", e))?;

        Ok(maybe_balance)
    }

    fn add_to_identity_balance<'c>(
        &self,
        identity_id: &Identifier,
        amount: u64,
        execution_context: Option<&'c StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn remove_from_identity_balance<'a>(
        &self,
        identity_id: &Identifier,
        amount: u64,
        execution_context: Option<&'a StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn add_to_system_credits(
        &self,
        amount: u64,
        execution_context: Option<&StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn remove_from_system_credits(
        &self,
        amount: u64,
        execution_context: Option<&StateTransitionExecutionContext>,
    ) -> Result<()> {
        unreachable!()
    }

    fn fetch_latest_platform_block_header(&self) -> AnyResult<Vec<u8>> {
        unreachable!(
            "fetch_latest_platform_block_header is deprecated a long ago and shoudn't be used"
        )
    }

    fn verify_instant_lock(
        &self,
        instant_lock: &InstantLock,
        execution_context: Option<&StateTransitionExecutionContext>,
    ) -> AnyResult<bool> {
        // TODO: Implement verify_instant_lock in Core RPC

        Ok(true)
    }

    fn is_asset_lock_transaction_out_point_already_used(
        &self,
        out_point_buffer: &[u8],
        execution_context: Option<&StateTransitionExecutionContext>,
    ) -> AnyResult<bool> {
        let bytes: [u8; 36] = out_point_buffer
            .try_into()
            .map_err(|_| anyhow!("invalid out_point_buffer"))?;

        self.drive
            .has_asset_lock_outpoint(&Bytes36(bytes), None) // todo
            .map_err(Into::into)
    }

    fn mark_asset_lock_transaction_out_point_as_used(
        &self,
        out_point_buffer: &[u8],
        execution_context: Option<&StateTransitionExecutionContext>,
    ) -> AnyResult<()> {
        unreachable!()
    }

    fn fetch_sml_store<T>(&self) -> AnyResult<T>
    where
        T: for<'de> Deserialize<'de> + 'static,
    {
        unreachable!("fetch_sml_store is deprecated a long ago and shouldn't be used")
    }

    fn is_in_the_valid_master_nodes_list(&self, out_point_buffer: [u8; 32]) -> AnyResult<bool> {
        let state = self.state.read().unwrap();

        let pro_tx_hash = ProTxHash::from_inner(out_point_buffer);

        Ok(state.hpmn_masternode_list.contains_key(&pro_tx_hash))
    }

    fn fetch_latest_withdrawal_transaction_index(&self) -> AnyResult<u64> {
        unreachable!(
            "fetch_latest_withdrawal_transaction_index is deprecated a long ago and shouldn't be used"
        )
    }

    fn fetch_latest_platform_core_chain_locked_height(&self) -> AnyResult<Option<u32>> {
        let guarded_block_execution_context = self.block_execution_context.read().unwrap();
        let block_execution_context =
            guarded_block_execution_context
                .as_ref()
                .ok_or(anyhow!("there should be an execution context when calling fetch_data_contract from dpp via state repository"))?;

        Ok(Some(
            block_execution_context
                .block_state_info
                .core_chain_locked_height,
        ))
    }

    fn enqueue_withdrawal_transaction(
        &self,
        index: u64,
        transaction_bytes: Vec<u8>,
    ) -> AnyResult<()> {
        unreachable!(
            "enqueue_withdrawal_transaction is deprecated a long ago and shouldn't be used"
        )
    }

    fn fetch_latest_platform_block_time(&self) -> AnyResult<u64> {
        let guarded_block_execution_context = self.block_execution_context.read().unwrap();
        let block_execution_context =
            guarded_block_execution_context
                .as_ref()
                .ok_or(anyhow!("there should be an execution context when calling fetch_data_contract from dpp via state repository"))?;

        Ok(block_execution_context.block_state_info.block_time_ms)
    }

    fn fetch_latest_platform_block_height(&self) -> AnyResult<u64> {
        let guarded_block_execution_context = self.block_execution_context.read().unwrap();
        let block_execution_context =
            guarded_block_execution_context
                .as_ref()
                .ok_or(anyhow!("there should be an execution context when calling fetch_data_contract from dpp via state repository"))?;

        Ok(block_execution_context.block_state_info.height)
    }
}
