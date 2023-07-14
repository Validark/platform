use crate::drive::Drive;

use crate::error::Error;

use dpp::version::drive_versions::DriveVersion;
use grovedb::TransactionArg;

impl Drive {
    /// Proves identity ids against public key hashes.
    pub(super) fn prove_identity_ids_by_unique_public_key_hashes_v0(
        &self,
        public_key_hashes: &[[u8; 20]],
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<Vec<u8>, Error> {
        let path_query = Self::identity_ids_by_unique_public_key_hash_query(public_key_hashes);
        self.grove_get_proved_path_query(
            &path_query,
            false,
            transaction,
            &mut vec![],
            drive_version,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::drive::Drive;
    use crate::tests::helpers::setup::setup_drive_with_initial_state_structure;
    use dpp::block::extended_block_info::BlockInfo;
    use dpp::identity::Identity;
    use dpp::version::drive_versions::DriveVersion;
    use std::collections::BTreeMap;

    #[test]
    fn should_prove_multiple_identity_ids() {
        let drive = setup_drive_with_initial_state_structure();
        let drive_version = DriveVersion::latest();

        let identities: BTreeMap<[u8; 32], Identity> =
            Identity::random_identities(None, 10, 3, Some(14))
                .into_iter()
                .map(|identity| (identity.id.to_buffer(), identity))
                .collect();

        for identity in identities.values() {
            drive
                .add_new_identity(identity.clone(), &BlockInfo::default(), true, None)
                .expect("expected to add an identity");
        }

        let key_hashes_to_identity_ids = identities
            .values()
            .flat_map(|identity| {
                identity
                    .public_keys
                    .values()
                    .filter(|public_key| public_key.key_type.is_unique_key_type())
                    .map(move |public_key| {
                        (
                            public_key
                                .hash()
                                .expect("expected to hash data")
                                .try_into()
                                .expect("expected to be 20 bytes"),
                            Some(identity.id.to_buffer()),
                        )
                    })
            })
            .collect::<BTreeMap<[u8; 20], Option<[u8; 32]>>>();

        let key_hashes = key_hashes_to_identity_ids
            .keys()
            .copied()
            .collect::<Vec<[u8; 20]>>();

        let proof = drive
            .prove_identity_ids_by_unique_public_key_hashes(&key_hashes, None, &drive_version)
            .expect("should not error when proving an identity");

        let (_, proved_identity_id): ([u8; 32], BTreeMap<[u8; 20], Option<[u8; 32]>>) =
            Drive::verify_identity_ids_by_public_key_hashes(proof.as_slice(), false, &key_hashes)
                .expect("expect that this be verified");

        assert_eq!(proved_identity_id, key_hashes_to_identity_ids);
    }
}
