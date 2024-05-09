mod data_contract_based_queries;
mod document_query;
mod identity_based_queries;
mod proofs;
mod response_metadata;
mod service;
mod system;
mod voting;
mod prefunded_specialized_balances;

use crate::error::query::QueryError;

use dpp::validation::ValidationResult;

pub use service::QueryService;

/// A query validation result
pub type QueryValidationResult<TData> = ValidationResult<TData, QueryError>;

#[cfg(test)]
mod tests {
    use crate::error::query::QueryError;
    use crate::platform_types::platform::Platform;
    use crate::platform_types::platform_state::PlatformState;
    use crate::query::QueryValidationResult;
    use crate::rpc::core::MockCoreRPCLike;
    use crate::test::helpers::setup::{TempPlatform, TestPlatformBuilder};
    use dpp::block::block_info::BlockInfo;
    use dpp::data_contract::DataContract;
    use drive::drive::batch::DataContractOperationType;
    use drive::drive::batch::DriveOperation::DataContractOperation;
    use platform_version::version::PlatformVersion;
    use std::borrow::Cow;
    use std::sync::Arc;

    pub fn setup_platform<'a>(
        with_genesis_state: bool,
    ) -> (
        TempPlatform<MockCoreRPCLike>,
        Arc<PlatformState>,
        &'a PlatformVersion,
    ) {
        let platform = if with_genesis_state {
            TestPlatformBuilder::new()
                .build_with_mock_rpc()
                .set_genesis_state()
        } else {
            TestPlatformBuilder::new()
                .build_with_mock_rpc()
                .set_initial_state_structure()
        };

        // We can't return a reference to Arc (`load` method) so we clone Arc (`load_full`).
        // This is a bit slower but we don't care since we are in test environment
        let platform_state = platform.platform.state.load_full();

        let platform_version = platform_state.current_platform_version().unwrap();

        (platform, platform_state, platform_version)
    }

    pub fn store_data_contract(
        platform: &Platform<MockCoreRPCLike>,
        data_contract: &DataContract,
        platform_version: &PlatformVersion,
    ) {
        let operation = DataContractOperation(DataContractOperationType::ApplyContract {
            contract: Cow::Owned(data_contract.to_owned()),
            storage_flags: None,
        });

        let block_info = BlockInfo::genesis();

        platform
            .drive
            .apply_drive_operations(vec![operation], true, &block_info, None, platform_version)
            .expect("expected to apply drive operations");
    }

    pub fn assert_invalid_identifier<TData: Clone>(
        validation_result: QueryValidationResult<TData>,
    ) {
        assert!(matches!(
            validation_result.errors.as_slice(),
            [QueryError::InvalidArgument(msg)] if msg.contains("id must be a valid identifier (32 bytes long)")
        ));
    }
}
