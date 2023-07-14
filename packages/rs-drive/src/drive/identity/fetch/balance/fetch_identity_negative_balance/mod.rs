mod v0;

use crate::drive::grove_operations::DirectQueryType;
use crate::drive::grove_operations::QueryTarget::QueryTargetValue;
use crate::drive::identity::{identity_path, IdentityRootStructure};
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use dpp::fee::Credits;
use dpp::version::drive_versions::DriveVersion;
use grovedb::Element::Item;
use grovedb::TransactionArg;

impl Drive {
    /// Fetches the Identity's negative balance operations from the backing store.
    /// This function is version controlled.
    ///
    /// # Arguments
    ///
    /// * `identity_id` - The ID of the Identity whose negative balance operations are to be fetched.
    /// * `apply` - Whether to get the estimated cost or the actual balance.
    /// * `transaction` - The current transaction.
    /// * `drive_operations` - The drive operations to be updated.
    /// * `drive_version` - The drive version.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Credits>, Error>` - The balance of the Identity if successful, or an error.
    pub(crate) fn fetch_identity_negative_balance_operations(
        &self,
        identity_id: [u8; 32],
        apply: bool,
        transaction: TransactionArg,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
        drive_version: &DriveVersion,
    ) -> Result<Option<Credits>, Error> {
        match drive_version
            .methods
            .identity
            .fetch
            .attributes
            .negative_balance
        {
            0 => self.fetch_identity_negative_balance_operations_v0(
                identity_id,
                apply,
                transaction,
                drive_operations,
                drive_version,
            ),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "fetch_identity_negative_balance_operations".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}
