use crate::drive::batch::grovedb_op_batch::GroveDbOpBatchV0Methods;
use crate::drive::Drive;

use crate::error::Error;

use crate::fee::op::LowLevelDriveOperation;

use dpp::util::deserializer::ProtocolVersion;

use dpp::version::PlatformVersion;
use dpp::ProtocolError;

use grovedb::TransactionArg;

impl Drive {
    /// Clear all version information from the backing store, this is done on epoch change in
    /// execution logic
    pub(super) fn change_to_new_version_and_clear_version_information_v0(
        &self,
        current_version: ProtocolVersion,
        next_version: ProtocolVersion,
        transaction: TransactionArg,
    ) -> Result<(), Error> {
        let platform_version = PlatformVersion::get(current_version)
            .map_err(|a| ProtocolError::PlatformVersionError(a))?;
        let mut batch_operations: Vec<LowLevelDriveOperation> = vec![];
        self.clear_version_information_operations_v0(
            transaction,
            &mut batch_operations,
            &platform_version.drive,
        )?;
        self.set_current_protocol_version_operations(
            current_version,
            transaction,
            &mut batch_operations,
            &platform_version.drive,
        )?;
        self.set_next_protocol_version_operations(
            next_version,
            transaction,
            &mut batch_operations,
            &platform_version.drive,
        )?;
        let grove_db_operations =
            LowLevelDriveOperation::grovedb_operations_batch(&batch_operations);
        if !grove_db_operations.is_empty() {
            self.apply_batch_grovedb_operations(
                None,
                transaction,
                grove_db_operations,
                &mut vec![],
                &platform_version.drive,
            )?;
        }
        Ok(())
    }
}
