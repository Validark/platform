use crate::error::Error;
use crate::platform_types::platform::Platform;
use crate::rpc::core::CoreRPCLike;
use dpp::block::block_info::BlockInfo;
use drive::grovedb::TransactionArg;
use platform_version::version::PlatformVersion;

impl<C> Platform<C>
where
    C: CoreRPCLike,
{
    /// Checks for ended vote polls
    #[inline(always)]
    pub(in crate::execution) fn check_for_ended_vote_polls_v0(
        &self,
        block_info: &BlockInfo,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<(), Error> {
        // Only contested vote polls for v0
        self.check_for_ended_contested_resource_vote_polls(
            block_info,
            transaction,
            platform_version,
        )
    }
}
