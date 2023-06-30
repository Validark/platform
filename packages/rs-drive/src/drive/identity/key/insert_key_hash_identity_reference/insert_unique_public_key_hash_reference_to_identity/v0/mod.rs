
use crate::drive::defaults::{
    DEFAULT_HASH_160_SIZE_U8, DEFAULT_HASH_SIZE_U32, DEFAULT_HASH_SIZE_U8,
    ESTIMATED_NON_UNIQUE_KEY_DUPLICATES,
};

use crate::drive::object_size_info::DriveKeyInfo::KeyRef;
use crate::drive::object_size_info::PathKeyElementInfo::PathKeyRefElement;

use crate::drive::{
    non_unique_key_hashes_sub_tree_path_vec, non_unique_key_hashes_tree_path,
    non_unique_key_hashes_tree_path_vec, unique_key_hashes_tree_path_vec, Drive,
};
use crate::error::drive::DriveError;
use crate::error::identity::IdentityError;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation::FunctionOperation;
use crate::fee::op::{FunctionOp, HashFunction, LowLevelDriveOperation};
use dpp::identity::IdentityPublicKey;
use grovedb::batch::KeyInfoPath;
use grovedb::EstimatedLayerCount::{ApproximateElements, PotentiallyAtMaxElements};
use grovedb::EstimatedLayerSizes::{AllItems, AllSubtrees};
use grovedb::EstimatedSumTrees::NoSumTrees;
use grovedb::{Element, EstimatedLayerInformation, TransactionArg};
use std::collections::HashMap;
use dpp::version::drive_versions::DriveVersion;


impl Drive {
    /// Insert a unique public key hash reference that contains an identity id
    /// Contrary to the name this is not a reference but an Item containing the identity
    /// identifier
    pub(super) fn insert_unique_public_key_hash_reference_to_identity_operations_v0(
        &self,
        identity_id: [u8; 32],
        public_key_hash: [u8; 20],
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<Vec<LowLevelDriveOperation>, Error> {
        let mut drive_operations = vec![];

        let already_exists = self.has_unique_public_key_hash_operations(
            public_key_hash,
            transaction,
            &mut drive_operations,
            drive_version,
        )?;

        if already_exists {
            return Err(Error::Identity(IdentityError::UniqueKeyAlreadyExists(
                "the key already exists in the unique set",
            )));
        }

        let already_exists = self.has_non_unique_public_key_hash_operations(
            public_key_hash,
            transaction,
            &mut drive_operations,
            drive_version,
        )?;

        if already_exists {
            return Err(Error::Identity(IdentityError::UniqueKeyAlreadyExists(
                "the key already exists in the non unique set",
            )));
        }

        let unique_key_hashes_path = unique_key_hashes_tree_path_vec();

        if let Some(estimated_costs_only_with_layer_info) = estimated_costs_only_with_layer_info {
            Self::add_estimation_costs_for_insert_unique_public_key_hash_reference(
                estimated_costs_only_with_layer_info,
                drive_version,
            )?;
        }

        // We insert the identity tree
        self.batch_insert::<0>(
            PathKeyRefElement((
                unique_key_hashes_path,
                public_key_hash.as_slice(),
                Element::Item(identity_id.to_vec(), None),
            )),
            &mut drive_operations,
            drive_version,
        )?;

        Ok(drive_operations)
    }
}