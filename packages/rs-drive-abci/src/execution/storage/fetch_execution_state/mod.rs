use crate::error::execution::ExecutionError;
use crate::error::Error;
use crate::execution::storage::EXECUTION_STORAGE_PATH;
use crate::platform_types::platform_state::PlatformState;
use dpp::version::PlatformVersion;
use drive::drive::Drive;
use drive::query::TransactionArg;

mod v0;

/// Fetches execution state from grovedb storage
pub fn fetch_execution_state(
    drive: &Drive,
    transaction: TransactionArg,
    platform_version: &PlatformVersion,
) -> Result<Option<PlatformState>, Error> {
    match platform_version
        .drive_abci
        .methods
        .execution_state_storage
        .fetch_execution_state
    {
        0 => v0::fetch_execution_state_v0(drive, transaction, platform_version),
        version => Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
            method: "fetch_execution_state".to_string(),
            known_versions: vec![0],
            received: version,
        })),
    }
}
