use crate::abci::app::PlatformApplication;
use crate::abci::handler;
use crate::error::Error;
use crate::platform_types::platform::Platform;
use crate::rpc::core::CoreRPCLike;
use std::fmt::Debug;
use tenderdash_abci::proto::abci as proto;

/// AbciApp is an implementation of ABCI Application, as defined by Tenderdash.
///
/// AbciApp implements logic that should be triggered when Tenderdash performs various operations, like
/// creating new proposal or finalizing new block.
pub struct QueryAbciApplication<'a, C> {
    /// Platform
    pub platform: &'a Platform<C>,
}

impl<'a, C> PlatformApplication<C> for QueryAbciApplication<'a, C> {
    fn platform(&self) -> &Platform<C> {
        self.platform
    }
}

impl<'a, C> QueryAbciApplication<'a, C> {
    /// Create new ABCI app
    pub fn new(platform: &'a Platform<C>) -> Result<QueryAbciApplication<'a, C>, Error> {
        let app = QueryAbciApplication { platform };

        Ok(app)
    }
}

impl<'a, C> Debug for QueryAbciApplication<'a, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<ReadOnlyAbciApp>")
    }
}

impl<'a, C> tenderdash_abci::Application for QueryAbciApplication<'a, C>
where
    C: CoreRPCLike,
{
    fn info(
        &self,
        _request: proto::RequestInfo,
    ) -> Result<proto::ResponseInfo, proto::ResponseException> {
        unreachable!("info is not implemented for read-only ABCI application")
    }

    fn init_chain(
        &self,
        _request: proto::RequestInitChain,
    ) -> Result<proto::ResponseInitChain, proto::ResponseException> {
        unreachable!("init_chain is not implemented for read-only ABCI application")
    }

    fn query(
        &self,
        request: proto::RequestQuery,
    ) -> Result<proto::ResponseQuery, proto::ResponseException> {
        handler::query(self, request)
    }

    fn check_tx(
        &self,
        request: proto::RequestCheckTx,
    ) -> Result<proto::ResponseCheckTx, proto::ResponseException> {
        handler::check_tx(self, request)
    }

    fn extend_vote(
        &self,
        _request: proto::RequestExtendVote,
    ) -> Result<proto::ResponseExtendVote, proto::ResponseException> {
        unreachable!("extend_vote is not implemented for read-only ABCI application")
    }

    fn finalize_block(
        &self,
        _request: proto::RequestFinalizeBlock,
    ) -> Result<proto::ResponseFinalizeBlock, proto::ResponseException> {
        // TODO: We don't need this

        self.platform
            .drive
            .grove
            .try_to_catch_up_from_primary()
            .expect("failed to catch up");

        Ok(proto::ResponseFinalizeBlock {
            events: vec![],
            retain_height: 0,
        })
    }

    fn prepare_proposal(
        &self,
        _request: proto::RequestPrepareProposal,
    ) -> Result<proto::ResponsePrepareProposal, proto::ResponseException> {
        unreachable!("prepare_proposal is not implemented for read-only ABCI application")
    }

    fn process_proposal(
        &self,
        _request: proto::RequestProcessProposal,
    ) -> Result<proto::ResponseProcessProposal, proto::ResponseException> {
        unreachable!("process_proposal is not implemented for read-only ABCI application")
    }

    fn verify_vote_extension(
        &self,
        _request: proto::RequestVerifyVoteExtension,
    ) -> Result<proto::ResponseVerifyVoteExtension, proto::ResponseException> {
        unreachable!("verify_vote_extension is not implemented for read-only ABCI application")
    }
}
