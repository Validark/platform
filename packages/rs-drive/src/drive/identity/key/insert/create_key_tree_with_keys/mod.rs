mod v0;

use crate::drive::identity::identity_path;
use crate::drive::identity::IdentityRootStructure::{IdentityTreeKeyReferences, IdentityTreeKeys};
use crate::drive::operation::LowLevelDriveOperation;
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use dpp::identity::IdentityPublicKey;
use dpp::version::drive_versions::DriveVersion;
use grovedb::batch::KeyInfoPath;
use grovedb::{EstimatedLayerInformation, TransactionArg};
use std::collections::HashMap;

impl Drive {
    /// Generates a vector of operations for creating a new identity key tree with the given keys.
    ///
    /// # Arguments
    ///
    /// * `identity_id` - An array of bytes representing the identity id.
    /// * `keys` - A vector of `IdentityPublicKey` objects to be inserted.
    /// * `estimated_costs_only_with_layer_info` - A mutable reference to an optional `HashMap` for estimated layer information.
    /// * `transaction` - A `TransactionArg` object representing the transaction for the operation.
    /// * `drive_version` - The version of the drive.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<LowLevelDriveOperation>, Error>` - If successful, returns a vector of `LowLevelDriveOperation` objects. If an error occurs during the operation, returns an `Error`.
    ///
    /// # Errors
    ///
    /// This function may return an `Error` if the operation creation process fails or if the drive version does not match any of the implemented method versions.
    pub(crate) fn create_key_tree_with_keys_operations(
        &self,
        identity_id: [u8; 32],
        keys: Vec<IdentityPublicKey>,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<Vec<LowLevelDriveOperation>, Error> {
        match drive_version
            .methods
            .identity
            .keys
            .insert
            .create_key_tree_with_keys
        {
            0 => self.create_key_tree_with_keys_operations_v0(
                identity_id,
                keys,
                estimated_costs_only_with_layer_info,
                transaction,
                drive_version,
            ),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "create_key_tree_with_keys_operations".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}
