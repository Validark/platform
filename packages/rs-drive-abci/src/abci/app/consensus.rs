use crate::abci::app::{BlockExecutionApplication, PlatformApplication, StateSyncApplication, TransactionalApplication};
use crate::abci::{AbciError, handler};
use crate::abci::handler::error::error_into_exception;
use crate::error::execution::ExecutionError;
use crate::error::Error;
use crate::execution::types::block_execution_context::BlockExecutionContext;
use crate::platform_types::platform::Platform;
use crate::rpc::core::CoreRPCLike;
use dpp::version::PlatformVersion;
use drive::grovedb::Transaction;
use std::fmt::Debug;
use std::sync::RwLock;
use tenderdash_abci::proto::abci as proto;
use drive::grovedb::replication::MultiStateSyncSession;
use crate::platform_types::snapshot::{SnapshotFetchingSession, SnapshotManager};

/// AbciApp is an implementation of ABCI Application, as defined by Tenderdash.
///
/// AbciApp implements logic that should be triggered when Tenderdash performs various operations, like
/// creating new proposal or finalizing new block.
/// 'p: 'tx, means that Platform must outlive the transaction
pub struct ConsensusAbciApplication<'p: 'tx, 'tx, C> {
    /// Platform
    platform: &'p Platform<C>,
    /// The current GroveDb transaction
    transaction: RwLock<Option<Transaction<'tx>>>,
    /// The current block execution context
    block_execution_context: RwLock<Option<BlockExecutionContext>>,
    snapshot_fetching_session: RwLock<Option<SnapshotFetchingSession<'tx>>>,
    snapshot_manager: SnapshotManager,
}

impl<'p, 'tx, C> ConsensusAbciApplication<'p, 'tx, C> {
    /// Create new ABCI app
    pub fn new(platform: &'p Platform<C>) -> Self {
        Self {
            platform,
            transaction: Default::default(),
            block_execution_context: Default::default(),
            snapshot_fetching_session: Default::default(),
            snapshot_manager: Default::default(),
        }
    }
}

impl<'p, 'tx, C> PlatformApplication<C> for ConsensusAbciApplication<'p, 'tx, C> {
    fn platform(&self) -> &Platform<C> {
        self.platform
    }
}

impl<'p, 'tx, C> StateSyncApplication<'tx> for ConsensusAbciApplication<'p, 'tx, C> {
    fn snapshot_fetching_session(&self) -> &RwLock<Option<SnapshotFetchingSession<'tx>>> {
        &self.snapshot_fetching_session
    }
}

impl<'p, 'tx, C> BlockExecutionApplication for ConsensusAbciApplication<'p, 'tx, C> {
    fn block_execution_context(&self) -> &RwLock<Option<BlockExecutionContext>> {
        &self.block_execution_context
    }
}

impl<'p, 'tx, C> TransactionalApplication<'tx> for ConsensusAbciApplication<'p, 'tx, C> {
    /// create and store a new transaction
    fn start_transaction(&self) {
        let transaction = self.platform.drive.grove.start_transaction();
        self.transaction.write().unwrap().replace(transaction);
    }

    fn transaction(&self) -> &RwLock<Option<Transaction<'tx>>> {
        &self.transaction
    }

    /// Commit a transaction
    fn commit_transaction(&self, platform_version: &PlatformVersion) -> Result<(), Error> {
        let transaction = self
            .transaction
            .write()
            .unwrap()
            .take()
            .ok_or(Error::Execution(ExecutionError::NotInTransaction(
                "trying to commit a transaction, but we are not in one",
            )))?;

        self.platform
            .drive
            .commit_transaction(transaction, &platform_version.drive)
            .map_err(Error::Drive)
    }
}

impl<'p, 'tx, C> Debug for ConsensusAbciApplication<'p, 'tx, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<ConsensusAbciApplication>")
    }
}

impl<'p, 'tx, C> tenderdash_abci::Application for ConsensusAbciApplication<'p, 'tx, C>
where
    C: CoreRPCLike,
{
    fn info(
        &self,
        request: proto::RequestInfo,
    ) -> Result<proto::ResponseInfo, proto::ResponseException> {
        handler::info(self, request).map_err(error_into_exception)
    }

    fn init_chain(
        &self,
        request: proto::RequestInitChain,
    ) -> Result<proto::ResponseInitChain, proto::ResponseException> {
        handler::init_chain(self, request).map_err(error_into_exception)
    }

    fn query(
        &self,
        _request: proto::RequestQuery,
    ) -> Result<proto::ResponseQuery, proto::ResponseException> {
        unreachable!("query is not implemented for consensus ABCI application")
    }

    fn check_tx(
        &self,
        _request: proto::RequestCheckTx,
    ) -> Result<proto::ResponseCheckTx, proto::ResponseException> {
        unreachable!("check_tx is not implemented for consensus ABCI application")
    }

    fn extend_vote(
        &self,
        request: proto::RequestExtendVote,
    ) -> Result<proto::ResponseExtendVote, proto::ResponseException> {
        handler::extend_vote(self, request).map_err(error_into_exception)
    }

    fn finalize_block(
        &self,
        request: proto::RequestFinalizeBlock,
    ) -> Result<proto::ResponseFinalizeBlock, proto::ResponseException> {
        handler::finalize_block(self, request).map_err(error_into_exception)
    }

    fn prepare_proposal(
        &self,
        request: proto::RequestPrepareProposal,
    ) -> Result<proto::ResponsePrepareProposal, proto::ResponseException> {
        handler::prepare_proposal(self, request).map_err(error_into_exception)
    }

    fn process_proposal(
        &self,
        request: proto::RequestProcessProposal,
    ) -> Result<proto::ResponseProcessProposal, proto::ResponseException> {
        handler::process_proposal(self, request).map_err(error_into_exception)
    }

    fn verify_vote_extension(
        &self,
        request: proto::RequestVerifyVoteExtension,
    ) -> Result<proto::ResponseVerifyVoteExtension, proto::ResponseException> {
        handler::verify_vote_extension(self, request).map_err(error_into_exception)
    }

    fn offer_snapshot(
        &self,
        request: proto::RequestOfferSnapshot,
    ) -> Result<proto::ResponseOfferSnapshot, proto::ResponseException> {
        match request.snapshot {
            None => {
                Err(error_into_exception(Error::Abci(AbciError::BadRequest("offer_snapshot missing snapshot".to_string()))))
            }
            Some(offered_snapshot) => {
                match self.snapshot_fetching_session.write() {
                    Ok(mut session_write) => {
                        // Now `session_write` is a mutable reference to the inner data
                        match *session_write {
                            Some(ref mut session) => {
                                // Access and modify `session` here
                                // Example: session.modify_some_field();
                                match &session.snapshot {
                                    None => {},
                                    Some(already_offered_snapshot) => {
                                        if offered_snapshot.height <= already_offered_snapshot.height {
                                            return Err(error_into_exception(Error::Abci(AbciError::BadRequest("offer_snapshot already syncing newest height".to_string()))))
                                        }
                                        /*
                                        if offered_snapshot.version != already_offered_snapshot.version {
                                            return Err(error_into_exception(Error::Abci(AbciError::BadRequest("fd".to_string()))))
                                        }
                                        */
                                    }
                                }


                                match self.platform.drive.grove.wipe() {
                                    Ok(_) => {
                                        let mut response = proto::ResponseOfferSnapshot::default();

                                        let state_sync_info = self.platform.drive.grove.start_new_session();

                                        session.snapshot = Option::from(offered_snapshot);
                                        session.app_hash = request.app_hash;
                                        session.state_sync_info = state_sync_info;

                                        Ok(response)
                                    }
                                    Err(e) => {
                                        Err(error_into_exception(Error::Abci(AbciError::BadRequest(format!("offer_snapshot unable to wipe grovedb:{}", e)))))
                                    }
                                }
                            },
                            None => {
                                // Handle the case where the Option is None
                                Err(error_into_exception(Error::Abci(AbciError::BadRequest("offer_snapshot unable to lock session".to_string()))))
                            }
                        }
                    },
                    Err(_poisoned) => {
                        // Handle the case where the lock is poisoned
                        Err(error_into_exception(Error::Abci(AbciError::BadRequest("offer_snapshot unable to lock session (poisoned)".to_string()))))
                    }
                }
            }
        }
    }

    fn apply_snapshot_chunk(
        &self,
        request: proto::RequestApplySnapshotChunk,
    ) -> Result<proto::ResponseApplySnapshotChunk, proto::ResponseException> {
        handler::apply_snapshot_chunk(self, request).map_err(error_into_exception)
    }
}

fn with_transaction_ref<'a,'b, C, F, R>(app: &'a ConsensusAbciApplication<'a,'b, C>, f: F) -> Result<R, Error>
    where
        F: FnOnce(&Transaction<'a>) -> R,
{
    let transaction_guard = app.transaction.read().unwrap();
    let transaction = transaction_guard.as_ref().ok_or(Error::Execution(ExecutionError::NotInTransaction(
        "trying to finalize block without a current transaction",
    )))?;
    Ok(f(transaction))
}
