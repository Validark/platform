use crate::drive::batch::drive_op_batch::DriveLowLevelOperationConverter;
use crate::drive::batch::{DriveOperation, GroveDbOpBatch};
use crate::drive::Drive;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use dpp::block::block_info::BlockInfo;
use dpp::version::drive_versions::DriveVersion;
use grovedb::batch::GroveDbOp;
use grovedb::TransactionArg;
use itertools::Itertools;

impl Drive {
    /// Convert a batch of drive operations to a batch of grove database operations.
    ///
    /// This function takes drive operations and converts them into grove database operations by
    /// processing each operation in the `drive_batch_operations` vector, transforming them to low-level
    /// drive operations and finally, into grove database operations. The resulting operations are
    /// returned as a `GroveDbOpBatch`.
    ///
    /// # Arguments
    ///
    /// * `drive_batch_operations` - A vector of high-level drive operations to be converted.
    /// * `block_info` - A reference to the block information associated with these operations.
    /// * `transaction` - A transaction argument to be used during processing.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `GroveDbOpBatch` with transformed grove database operations,
    /// or an error if any step in the conversion process fails.
    pub(super) fn convert_drive_operations_to_grove_operations_v0(
        &self,
        drive_batch_operations: Vec<DriveOperation>,
        block_info: &BlockInfo,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<GroveDbOpBatch, Error> {
        let ops = drive_batch_operations
            .into_iter()
            .map(|drive_op| {
                let inner_drive_operations = drive_op.into_low_level_drive_operations(
                    self,
                    &mut None,
                    block_info,
                    transaction,
                    drive_version,
                )?;
                Ok(LowLevelDriveOperation::grovedb_operations_consume(
                    inner_drive_operations,
                ))
            })
            .flatten_ok()
            .collect::<Result<Vec<GroveDbOp>, Error>>()?;
        Ok(GroveDbOpBatch::from_operations(ops))
    }
}
