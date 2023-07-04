mod v0;

use grovedb::TransactionArg;
use dpp::block::block_info::BlockInfo;
use crate::fee::calculate_fee;
use dpp::state_transition::fee::fee_result::FeeResult;
use dpp::version::drive_versions::DriveVersion;
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;

impl Drive {
    /// Fetches the Identity's balance from the backing store.
    /// If the balance is 0, then also provide debt, respecting drive versioning.
    ///
    /// # Arguments
    ///
    /// * `identity_id` - The ID of the Identity whose balance is to be fetched.
    /// * `transaction` - The current transaction.
    /// * `drive_version` - The drive version.
    ///
    /// # Returns
    ///
    /// * `Result<Option<SignedCredits>, Error>` - The balance of the Identity if successful, or an error.
    pub fn fetch_identity_balance_include_debt(
        &self,
        identity_id: [u8; 32],
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<Option<SignedCredits>, Error> {
        match drive_version.methods.identity.fetch.attributes.balance {
            0 => self.fetch_identity_balance_include_debt_v0(identity_id, transaction, drive_version),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "fetch_identity_balance_include_debt".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }

    /// Fetches the Identity's balance from the backing store, including the estimated cost.
    /// If the balance is 0, then also provide debt, respecting drive versioning.
    ///
    /// # Arguments
    ///
    /// * `identity_id` - The ID of the Identity whose balance is to be fetched.
    /// * `block_info` - The information about the current block.
    /// * `apply` - Whether to get the estimated cost or the actual balance.
    /// * `transaction` - The current transaction.
    /// * `drive_version` - The drive version.
    ///
    /// # Returns
    ///
    /// * `Result<(Option<SignedCredits>, FeeResult), Error>` - The balance of the Identity and the fee if successful, or an error.
    pub fn fetch_identity_balance_include_debt_with_costs(
        &self,
        identity_id: [u8; 32],
        block_info: &BlockInfo,
        apply: bool,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<(Option<SignedCredits>, FeeResult), Error> {
        match drive_version.methods.identity.fetch.attributes.balance {
            0 => self.fetch_identity_balance_include_debt_with_costs_v0(identity_id, block_info, apply, transaction, drive_version),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "fetch_identity_balance_include_debt_with_costs".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }

    /// Fetches the Identity's balance from the backing store.
    /// If the balance is 0, then also provide debt.
    /// Operations are created based on the 'apply' flag (stateful vs stateless).
    ///
    /// # Arguments
    ///
    /// * `identity_id` - The ID of the Identity whose balance is to be fetched.
    /// * `apply` - Whether to create stateful or stateless operations.
    /// * `transaction` - The current transaction.
    /// * `drive_operations` - The drive operations to be updated.
    /// * `drive_version` - The drive version.
    ///
    /// # Returns
    ///
    /// * `Result<Option<SignedCredits>, Error>` - The balance of the Identity if successful, or an error.
    pub(crate) fn fetch_identity_balance_include_debt_operations(
        &self,
        identity_id: [u8; 32],
        apply: bool,
        transaction: TransactionArg,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
        drive_version: &DriveVersion,
    ) -> Result<Option<SignedCredits>, Error> {
        match drive_version.methods.identity.fetch.attributes.balance {
            0 => self.fetch_identity_balance_include_debt_operations_v0(identity_id, apply, transaction, drive_operations, drive_version),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "fetch_identity_balance_include_debt_operations".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}