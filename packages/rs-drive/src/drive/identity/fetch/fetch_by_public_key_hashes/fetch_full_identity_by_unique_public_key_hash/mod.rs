mod v0;

use crate::drive::Drive;
use crate::error::{drive::DriveError, Error};
use crate::fee::op::LowLevelDriveOperation;
use dpp::identity::Identity;
use dpp::version::drive_versions::DriveVersion;
use grovedb::TransactionArg;

impl Drive {
    /// Fetches an identity with all its related information from storage based on a unique public key hash.
    ///
    /// This function leverages the versioning system to direct the fetch operation to the appropriate handler based on the `DriveVersion` provided.
    ///
    /// # Arguments
    ///
    /// * `public_key_hash` - A unique public key hash corresponding to the identity to be fetched.
    /// * `transaction` - Transaction arguments.
    /// * `drive_version` - A reference to the drive version.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an `Option` of the `Identity` if it exists, otherwise an `Error` if the fetch operation fails or the version is not supported.
    pub fn fetch_full_identity_by_unique_public_key_hash(
        &self,
        public_key_hash: [u8; 20],
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<Option<Identity>, Error> {
        match drive_version
            .methods
            .identity
            .fetch
            .public_key_hashes
            .fetch_full_identity_by_unique_public_key_hash
        {
            0 => self.fetch_full_identity_by_unique_public_key_hash_v0(
                public_key_hash,
                transaction,
                drive_version,
            ),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "fetch_full_identity_by_unique_public_key_hash".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }

    /// Fetches an identity and its flags from storage, based on a unique public key hash. This function also logs drive operations.
    ///
    /// This function leverages the versioning system to direct the fetch operation to the appropriate handler based on the `DriveVersion` provided.
    ///
    /// # Arguments
    ///
    /// * `public_key_hash` - A unique public key hash corresponding to the identity to be fetched.
    /// * `transaction` - Transaction arguments.
    /// * `drive_operations` - A mutable reference to a vector of drive operations.
    /// * `drive_version` - A reference to the drive version.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an `Option` of the `Identity` if it exists, otherwise an `Error` if the fetch operation fails or the version is not supported.
    pub(crate) fn fetch_full_identity_by_unique_public_key_hash_operations(
        &self,
        public_key_hash: [u8; 20],
        transaction: TransactionArg,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
        drive_version: &DriveVersion,
    ) -> Result<Option<Identity>, Error> {
        match drive_version
            .methods
            .identity
            .fetch
            .public_key_hashes
            .fetch_full_identity_by_unique_public_key_hash
        {
            0 => self.fetch_full_identity_by_unique_public_key_hash_operations_v0(
                public_key_hash,
                transaction,
                drive_operations,
                drive_version,
            ),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "fetch_full_identity_by_unique_public_key_hash_operations".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}
