mod v0;

use crate::drive::batch::{DriveOperation, GroveDbOpBatch};
use crate::drive::credit_pools::pending_epoch_refunds::pending_epoch_refunds_path_vec;
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use dpp::version::drive_versions::DriveVersion;
use grovedb::Element;

impl Drive {
    /// Adds GroveDB batch operations to update pending epoch storage pool updates
    ///
    /// # Arguments
    ///
    /// * `batch` - A mutable reference to a Vec of DriveOperations.
    /// * `refunds_per_epoch` - A CreditsPerEpoch instance.
    /// * `drive_version` - A DriveVersion instance representing the version of the drive.
    ///
    /// # Returns
    ///
    /// A Result containing a boolean indicating whether the proposer's tree is empty, or an Error.
    pub(crate) fn add_update_pending_epoch_refunds_operations(
        batch: &mut Vec<DriveOperation>,
        refunds_per_epoch: CreditsPerEpoch,
        drive_version: &DriveVersion,
    ) -> Result<(), Error> {
        match drive_version
            .methods
            .credit_pools
            .epochs
            .add_update_pending_epoch_refunds_operations
        {
            0 => Self::add_update_pending_epoch_refunds_operations_v0(batch, refunds_per_epoch),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "add_update_pending_epoch_refunds_operations".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}
