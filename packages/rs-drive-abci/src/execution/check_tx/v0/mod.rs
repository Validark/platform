use crate::error::Error;
#[cfg(test)]
use crate::execution::types::execution_result::ExecutionResult;
#[cfg(test)]
use crate::execution::types::execution_result::ExecutionResult::ConsensusExecutionError;
use crate::execution::validation::state_transition::processor::process_state_transition;
use crate::platform_types::platform::{Platform, PlatformRef};
use crate::platform_types::platform_state::v0::PlatformStateV0Methods;
use crate::rpc::core::CoreRPCLike;
use dpp::block::block_info::BlockInfo;
use dpp::block::extended_block_info::v0::ExtendedBlockInfoV0Getters;
use dpp::consensus::ConsensusError;
use dpp::fee::fee_result::FeeResult;
use dpp::serialization::PlatformDeserializable;
use dpp::state_transition::StateTransition;
#[cfg(test)]
use dpp::validation::SimpleConsensusValidationResult;
use dpp::validation::ValidationResult;
#[cfg(test)]
use drive::grovedb::Transaction;

impl<C> Platform<C>
where
    C: CoreRPCLike,
{
    #[cfg(test)]
    pub(in crate::execution) fn execute_tx(
        &self,
        raw_tx: Vec<u8>,
        block_info: &BlockInfo,
        transaction: &Transaction,
    ) -> Result<ExecutionResult, Error> {
        let state_transition =
            StateTransition::deserialize(raw_tx.as_slice()).map_err(Error::Protocol)?;
        let state_read_guard = self.state.read().unwrap();
        let platform_ref = PlatformRef {
            drive: &self.drive,
            state: &state_read_guard,
            config: &self.config,
            core_rpc: &self.core_rpc,
        };

        let state_transition_execution_event =
            process_state_transition(&platform_ref, state_transition, Some(transaction))?;

        if state_transition_execution_event.is_valid() {
            let platform_version = platform_ref.state.current_platform_version()?;
            let execution_event = state_transition_execution_event.into_data()?;
            self.execute_event(execution_event, block_info, transaction, platform_version)
        } else {
            Ok(ConsensusExecutionError(
                SimpleConsensusValidationResult::new_with_errors(
                    state_transition_execution_event.errors,
                ),
            ))
        }
    }

    /// Checks a state transition to determine if it should be added to the mempool.
    ///
    /// This function performs a few checks, including validating the state transition and ensuring that the
    /// user can pay for it. It may be inaccurate in rare cases, so the proposer needs to re-check transactions
    /// before proposing a block.
    ///
    /// # Arguments
    ///
    /// * `raw_tx` - A raw transaction represented as a vector of bytes.
    ///
    /// # Returns
    ///
    /// * `Result<ValidationResult<FeeResult, ConsensusError>, Error>` - If the state transition passes all
    ///   checks, it returns a `ValidationResult` with fee information. If any check fails, it returns an `Error`.
    pub(super) fn check_tx_v0(
        &self,
        raw_tx: &[u8],
    ) -> Result<ValidationResult<FeeResult, ConsensusError>, Error> {
        let state_transition = StateTransition::deserialize(raw_tx).map_err(Error::Protocol)?;
        let state_read_guard = self.state.read().unwrap();
        let platform_ref = PlatformRef {
            drive: &self.drive,
            state: &state_read_guard,
            config: &self.config,
            core_rpc: &self.core_rpc,
        };
        let execution_event = process_state_transition(&platform_ref, state_transition, None)?;

        let platform_version = platform_ref.state.current_platform_version()?;

        // We should run the execution event in dry run to see if we would have enough fees for the transaction

        // We need the approximate block info
        if let Some(block_info) = state_read_guard.last_committed_block_info().as_ref() {
            // We do not put the transaction, because this event happens outside of a block
            execution_event.and_then_borrowed_validation(|execution_event| {
                self.validate_fees_of_event(
                    execution_event,
                    &block_info.basic_info(),
                    None,
                    platform_version,
                )
            })
        } else {
            execution_event.and_then_borrowed_validation(|execution_event| {
                self.validate_fees_of_event(
                    execution_event,
                    &BlockInfo::default(),
                    None,
                    platform_version,
                )
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::PlatformConfig;
    use crate::execution::types::execution_result::ExecutionResult::SuccessfulPaidExecution;
    use crate::platform_types::platform_state::v0::PlatformStateV0Methods;
    use crate::platform_types::system_identity_public_keys::v0::SystemIdentityPublicKeysV0;
    use crate::test::helpers::setup::TestPlatformBuilder;
    use crate::test::helpers::signer::SimpleSigner;
    use dpp::block::block_info::BlockInfo;
    use dpp::consensus::basic::BasicError;
    use dpp::consensus::signature::SignatureError;
    use dpp::consensus::state::state_error::StateError;
    use dpp::consensus::ConsensusError;
    use dpp::dashcore::secp256k1::Secp256k1;
    use dpp::dashcore::{signer, KeyPair, Network, PrivateKey};
    use dpp::data_contract::base::DataContractBaseMethodsV0;
    use dpp::data_contract::document_type::random_document::CreateRandomDocument;
    use dpp::data_contracts::dpns_contract;
    use dpp::document::document_methods::DocumentMethodsV0;
    use dpp::document::{DocumentV0Getters, DocumentV0Setters};
    use dpp::identity::accessors::IdentityGettersV0;
    use dpp::identity::state_transition::asset_lock_proof;
    use dpp::identity::KeyType::ECDSA_SECP256K1;
    use dpp::identity::{Identity, IdentityV0, KeyType, Purpose, SecurityLevel};
    use dpp::prelude::{Identifier, IdentityPublicKey};
    use dpp::serialization::{PlatformSerializable, Signable};
    use dpp::state_transition::data_contract_create_transition::methods::DataContractCreateTransitionMethodsV0;
    use dpp::state_transition::data_contract_create_transition::DataContractCreateTransition;
    use dpp::state_transition::documents_batch_transition::methods::v0::DocumentsBatchTransitionMethodsV0;
    use dpp::state_transition::documents_batch_transition::DocumentsBatchTransition;
    use dpp::state_transition::identity_create_transition::methods::IdentityCreateTransitionMethodsV0;
    use dpp::state_transition::identity_create_transition::IdentityCreateTransition;
    use dpp::state_transition::identity_update_transition::v0::IdentityUpdateTransitionV0;
    use dpp::state_transition::public_key_in_creation::v0::IdentityPublicKeyInCreationV0;
    use dpp::state_transition::public_key_in_creation::IdentityPublicKeyInCreation;
    use dpp::state_transition::StateTransition;
    use dpp::tests::fixtures::{get_dashpay_contract_fixture, instant_asset_lock_proof_fixture};
    use dpp::version::PlatformVersion;
    use dpp::NativeBlsModule;
    use drive::drive::contract::DataContractFetchInfo;
    use platform_version::TryIntoPlatformVersioned;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::BTreeMap;

    #[test]
    fn data_contract_create_check_tx() {
        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();
        let state = platform.state.read().unwrap();
        let protocol_version = state.current_protocol_version_in_consensus();
        let platform_version = PlatformVersion::get(protocol_version).unwrap();

        let (key, private_key) = IdentityPublicKey::random_ecdsa_critical_level_authentication_key(
            1,
            Some(1),
            platform_version,
        )
        .expect("expected to get key pair");

        platform
            .drive
            .create_initial_state_structure(None, platform_version)
            .expect("expected to create state structure");
        let identity: Identity = IdentityV0 {
            id: Identifier::new([
                158, 113, 180, 126, 91, 83, 62, 44, 83, 54, 97, 88, 240, 215, 84, 139, 167, 156,
                166, 203, 222, 4, 64, 31, 215, 199, 149, 151, 190, 246, 251, 44,
            ]),
            public_keys: BTreeMap::from([(1, key.clone())]),
            balance: 1000000000,
            revision: 0,
        }
        .into();

        let dashpay = get_dashpay_contract_fixture(Some(identity.id()), protocol_version);
        let mut create_contract_state_transition: StateTransition = dashpay
            .try_into_platform_versioned(platform_version)
            .expect("expected a state transition");
        create_contract_state_transition
            .sign(&key, private_key.as_slice(), &NativeBlsModule::default())
            .expect("expected to sign transition");
        let serialized = create_contract_state_transition
            .serialize()
            .expect("serialized state transition");
        platform
            .drive
            .add_new_identity(
                identity,
                &BlockInfo::default(),
                true,
                None,
                platform_version,
            )
            .expect("expected to insert identity");

        let validation_result = platform
            .check_tx(serialized.as_slice())
            .expect("expected to check tx");

        assert!(validation_result.errors.is_empty());
    }

    #[test]
    fn document_update_check_tx() {
        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        let platform_state = platform.state.read().unwrap();
        let platform_version = platform_state.current_platform_version().unwrap();

        let mut signer = SimpleSigner::default();

        let mut rng = StdRng::seed_from_u64(567);

        let (key, private_key) = IdentityPublicKey::random_ecdsa_critical_level_authentication_key(
            1,
            Some(19),
            platform_version,
        )
        .expect("expected to get key pair");

        signer.add_key(key.clone(), private_key.clone());

        let (_, pk) = ECDSA_SECP256K1
            .random_public_and_private_key_data(&mut rng, platform_version)
            .unwrap();

        let asset_lock_proof = instant_asset_lock_proof_fixture(Some(
            PrivateKey::from_slice(pk.as_slice(), Network::Testnet).unwrap(),
        ));

        let identifier = asset_lock_proof
            .create_identifier()
            .expect("expected an identifier");

        let identity: Identity = IdentityV0 {
            id: identifier,
            public_keys: BTreeMap::from([(1, key.clone())]),
            balance: 1000000000,
            revision: 0,
        }
        .into();

        let identity_create_transition: StateTransition =
            IdentityCreateTransition::try_from_identity_with_signer(
                identity.clone(),
                asset_lock_proof,
                pk.as_slice(),
                &signer,
                &NativeBlsModule::default(),
                platform_version,
            )
            .expect("expected an identity create transition")
            .into();

        let identity_create_serialized_transition = identity_create_transition
            .serialize()
            .expect("serialized state transition");

        let dashpay =
            get_dashpay_contract_fixture(Some(identity.id()), platform_version.protocol_version);
        let dashpay_contract = dashpay.data_contract().clone();
        let mut create_contract_state_transition: StateTransition = dashpay
            .try_into_platform_versioned(platform_version)
            .expect("expected a state transition");
        create_contract_state_transition
            .sign(&key, private_key.as_slice(), &NativeBlsModule::default())
            .expect("expected to sign transition");
        let data_contract_create_serialized_transition = create_contract_state_transition
            .serialize()
            .expect("expected data contract create serialized state transition");

        let profile = dashpay_contract
            .document_type_for_name("profile")
            .expect("expected a profile document type");

        let mut document = profile
            .random_document_with_rng(&mut rng, platform_version)
            .expect("expected a random document");

        document.set_owner_id(identifier);

        let mut altered_document = document.clone();

        altered_document.increment_revision().unwrap();
        altered_document.set("displayName", "Samuel".into());

        let documents_batch_create_transition =
            DocumentsBatchTransition::new_document_creation_transition_from_document(
                document,
                profile,
                [1; 32],
                &key,
                &signer,
                platform_version,
                None,
                None,
                None,
            )
            .expect("expect to create documents batch transition");

        let documents_batch_create_serialized_transition = documents_batch_create_transition
            .serialize()
            .expect("expected documents batch serialized state transition");

        let documents_batch_update_transition =
            DocumentsBatchTransition::new_document_replacement_transition_from_document(
                altered_document,
                profile,
                &key,
                &signer,
                platform_version,
                None,
                None,
                None,
            )
            .expect("expect to create documents batch transition");

        let documents_batch_update_serialized_transition = documents_batch_update_transition
            .serialize()
            .expect("expected documents batch serialized state transition");

        platform
            .drive
            .create_initial_state_structure(None, platform_version)
            .expect("expected to create state structure");

        let transaction = platform.drive.grove.start_transaction();

        let validation_result = platform
            .execute_tx(
                identity_create_serialized_transition,
                &BlockInfo::default(),
                &transaction,
            )
            .expect("expected to execute identity_create tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        let validation_result = platform
            .execute_tx(
                data_contract_create_serialized_transition,
                &BlockInfo::default(),
                &transaction,
            )
            .expect("expected to execute data_contract_create tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));
        let validation_result = platform
            .execute_tx(
                documents_batch_create_serialized_transition,
                &BlockInfo::default(),
                &transaction,
            )
            .expect("expected to execute document_create tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");

        let validation_result = platform
            .check_tx(documents_batch_update_serialized_transition.as_slice())
            .expect("expected to check tx");

        assert!(validation_result.errors.is_empty());
    }

    #[test]
    fn identity_top_up_check_tx() {
        let identity_top_up = hex::decode("04030000c601018c047719bb8b287e33b788671131b16b1f355d1b3ba6c4917396d0d7bf41e681000000007f1df760772c7ab48c042c01319bd553b7a635936e9a06fa382eb5037638e6ba077a524aa82c6b20e7b8dcadafa46f8ecc59b2dea8c3d6269a24cd5cad74b712ae5a460d11242bd345e168028b3e8442439a63847aa736057a6cd587ae9f7bca1f59f3045566233566142cbca5a7b525085bf96c621ba39f838d6c5c31b116e756753177aa303a8ea712e17ad1ff5dfb0b1504c03d5c225c5cbdb1ee8f6636f0df03000000018c047719bb8b287e33b788671131b16b1f355d1b3ba6c4917396d0d7bf41e681000000006b483045022100d71b565e319a0b85725d1eca250da27d846c6b015e601254e3f8aeb11c0feab60220381c92a46467d6c5270d424b666b989e444e72955f3d5b77d8be9965335b43bd01210222150e3b66410341308b646234bff9c203172c6720b2ecc838c71d94f670066affffffff02e093040000000000166a144cf5fee3ebdce0f51540a3504091c0dccb0f7d343832963b000000001976a914f3b05a1dda565b0013cb9857e708d840bcd47bef88ac00000000003012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac014120d56826c39c07eaea7157b8b717fdcef73fbc99cc680e34f695e0c763d79531691d8ea117cd4623e96a25cbf673e5b1da6e43a96d5bb2a65fe82c2efd4dc2c6dc").expect("expected to decode");

        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        let platform_state = platform.state.read().unwrap();
        let platform_version = platform_state.current_platform_version().unwrap();

        let genesis_time = 0;

        let system_identity_public_keys_v0: SystemIdentityPublicKeysV0 =
            platform.config.abci.keys.clone().into();

        platform
            .create_genesis_state_v0(
                genesis_time,
                system_identity_public_keys_v0.into(),
                None,
                platform_version,
            )
            .expect("expected to create genesis state");

        let validation_result = platform
            .check_tx_v0(identity_top_up.as_slice())
            .expect("expected to check tx");

        assert!(validation_result.errors.is_empty());

        let transaction = platform.drive.grove.start_transaction();

        let validation_result = platform
            .execute_tx(identity_top_up, &BlockInfo::default(), &transaction)
            .expect("expected to execute identity top up tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");
    }

    #[test]
    fn identity_cant_double_top_up() {
        let identity_top_up = hex::decode("04030000c601018c047719bb8b287e33b788671131b16b1f355d1b3ba6c4917396d0d7bf41e681000000007f1df760772c7ab48c042c01319bd553b7a635936e9a06fa382eb5037638e6ba077a524aa82c6b20e7b8dcadafa46f8ecc59b2dea8c3d6269a24cd5cad74b712ae5a460d11242bd345e168028b3e8442439a63847aa736057a6cd587ae9f7bca1f59f3045566233566142cbca5a7b525085bf96c621ba39f838d6c5c31b116e756753177aa303a8ea712e17ad1ff5dfb0b1504c03d5c225c5cbdb1ee8f6636f0df03000000018c047719bb8b287e33b788671131b16b1f355d1b3ba6c4917396d0d7bf41e681000000006b483045022100d71b565e319a0b85725d1eca250da27d846c6b015e601254e3f8aeb11c0feab60220381c92a46467d6c5270d424b666b989e444e72955f3d5b77d8be9965335b43bd01210222150e3b66410341308b646234bff9c203172c6720b2ecc838c71d94f670066affffffff02e093040000000000166a144cf5fee3ebdce0f51540a3504091c0dccb0f7d343832963b000000001976a914f3b05a1dda565b0013cb9857e708d840bcd47bef88ac00000000003012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac014120d56826c39c07eaea7157b8b717fdcef73fbc99cc680e34f695e0c763d79531691d8ea117cd4623e96a25cbf673e5b1da6e43a96d5bb2a65fe82c2efd4dc2c6dc").expect("expected to decode");

        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        let platform_state = platform.state.read().unwrap();
        let platform_version = platform_state.current_platform_version().unwrap();

        let genesis_time = 0;

        let system_identity_public_keys_v0: SystemIdentityPublicKeysV0 =
            platform.config.abci.keys.clone().into();

        platform
            .create_genesis_state_v0(
                genesis_time,
                system_identity_public_keys_v0.into(),
                None,
                platform_version,
            )
            .expect("expected to create genesis state");

        let transaction = platform.drive.grove.start_transaction();

        let validation_result = platform
            .execute_tx(identity_top_up.clone(), &BlockInfo::default(), &transaction)
            .expect("expected to execute identity top up tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");

        let validation_result = platform
            .check_tx_v0(identity_top_up.as_slice())
            .expect("expected to check tx");

        assert!(matches!(
            validation_result.errors.first().expect("expected an error"),
            ConsensusError::BasicError(
                BasicError::IdentityAssetLockTransactionOutPointAlreadyExistsError(_)
            )
        ));
    }

    #[test]
    fn identity_update_doesnt_panic() {
        let identity_top_up = hex::decode("0601054e683919ac96d2e9b099162d845f7540fb1e776eadaca5d84b28235e298d9224020101000002002103a106d1b2fbe4f47c0f9a6cf89b7ed625b5f5972798c9af73475fb179bcb047364120db77e92f250ff1c1114b26355d0a186ab439cbd26ac18ed89c7c63e32b3aea4b339b10feeb2dffd7efa1bdb3e48332a6cdea1951071fb41ef30011a267eb6bbb000000411f56f03e48506fef87be778167838128eb06edc541667c7f010344bb69e54ba1df2c2818db073cc1f7c3966d1d99f0aa1c5e4e1d21959da7f4b89e6c19c123a8b9").expect("expected to decode");

        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        let platform_state = platform.state.read().unwrap();
        let platform_version = platform_state.current_platform_version().unwrap();

        let genesis_time = 0;

        let system_identity_public_keys_v0: SystemIdentityPublicKeysV0 =
            platform.config.abci.keys.clone().into();

        platform
            .create_genesis_state_v0(
                genesis_time,
                system_identity_public_keys_v0.into(),
                None,
                platform_version,
            )
            .expect("expected to create genesis state");

        let validation_result = platform
            .check_tx_v0(identity_top_up.as_slice())
            .expect("expected to check tx");

        assert!(matches!(
            validation_result.errors.first().expect("expected an error"),
            ConsensusError::SignatureError(SignatureError::IdentityNotFoundError(_))
        ));
    }

    #[test]
    fn identity_cant_create_with_used_outpoint() {
        let identity_create = hex::decode("03020200000000002102b60f8631519ee2245dfbd7ff107540d378b9d9ca3e8356d1f791703b3027d71b412039feb0213146906bbe7b5c3edd127d567780258f11fd33475af3d2a48679c05e22c446a312d047ed61e94664a787bcac3aae2ab82417232e4a8edae519c525c4010000020021035f06136d7de4240dcfaa885f0c8236d752f906b3e1e8eb157cb816b5a58d6cd141202a6be48392e482a283380fc4679c0147589a898767897e958b3b85abf5703ef734c2041e172794199ecac63ee37f2744e13b6fc9e7d8265b9f2d0ea5a05e9be30000c601013f4fdb109bcd46a4f8eaf72c8bfca482b028e51a8e136519107c7b2c525a5f4100000000de33c4662f152d963eb1ee779cd11891d77890009e4aeabbfce29af45c402b846cc532593e62741065732e91762ac89822b1a6295c36c82863baf59bc0a0ad0b82f3a8f5d956f005dd505ad7e27cbb75c0196fa92f45f504079edb0ba5b64c2e93830f13a63cf9b7cba5a82836a979c1016378237e48859057bd6d2a09c47f78f0aa56f259aba31f205f68adc9f7b5931ea8929806dcd497c23fca262898a163de03000000013f4fdb109bcd46a4f8eaf72c8bfca482b028e51a8e136519107c7b2c525a5f41000000006a47304402207e065274128f612325de5c4e8332e8d3f49891ef2cd1ce4e57da7520484af7290220095d40dfa4490ea336155e82e969ee0f4b59d12126bec6ec7bfe1fee4cbf27d6012102faf354bb3b1487f939e7d7e2e25ec71ad6e3d9804f649e3765ea143d65042bdfffffffff02a086010000000000166a1445399c0424ad422c9dd1f1798ac570666a23f519d8230900000000001976a9143f10001d7dfcdbe673eccecd0cc5dabf208eb78388ac000000000001411f656fd575598f79545029603db7978810292c4b6d923898a09aaeaee2276c32da66f46fbb9ce3230c2485ae803ed98d3bf415fc31f4f1d8f17c44413580c8f7b6563829f5f8f22a6d3dba3143ac38cff96c1fcef4c18b2dbb7da0a2be80757955").expect("expected to decode");

        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        let platform_state = platform.state.read().unwrap();
        let platform_version = platform_state.current_platform_version().unwrap();

        let genesis_time = 0;

        let system_identity_public_keys_v0: SystemIdentityPublicKeysV0 =
            platform.config.abci.keys.clone().into();

        platform
            .create_genesis_state_v0(
                genesis_time,
                system_identity_public_keys_v0.into(),
                None,
                platform_version,
            )
            .expect("expected to create genesis state");

        let transaction = platform.drive.grove.start_transaction();

        let validation_result = platform
            .execute_tx(identity_create.clone(), &BlockInfo::default(), &transaction)
            .expect("expected to execute identity create tx");

        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");

        let validation_result = platform
            .check_tx_v0(identity_create.as_slice())
            .expect("expected to check tx");

        assert!(matches!(
            validation_result.errors.first().expect("expected an error"),
            ConsensusError::StateError(StateError::IdentityAlreadyExistsError(_))
        ));
    }

    #[test]
    fn dpns_owner_top_up() {
        let identity_top_up_first = hex::decode("04030000c60101ba15a200ad72279b3a5e060305660091412babfe6f4bed3c17af33da920baa93000000000460721b022a879c9afcf005fe36136e436f966a0700adf48aa7cf5b45be712e18cf6fd4c5423912f637a99e7cdc2723a42b6ee6d2ea450521a030319bcef330870eb1f0f6918d30e3ab3cfb653b5c087acd52af99efc1c997cd9999401cd323c47858b726f62ef02cf6672a05c7d4b308a59904256d5d1aed723eea5e6666622986750c8011a78b91ddfdccd519af3cb154b90bff6de975b9b3e38c3d9845d5df0300000001ba15a200ad72279b3a5e060305660091412babfe6f4bed3c17af33da920baa93000000006b4830450221008d13e35776dfc5f3dae9427dad1f205e069cac1f3681bc0bca55c434fd7782b002207712496a6eb693527f47f60d39c5cfd5ba99ff9df1ac4e9fe1a5f40c42e5645f0121032eb006412f801abfbfa25e3387cf15583b553d3b89cd30823579232ac021fc33ffffffff02e093040000000000166a14fec95dcf766d232571c45eb86cffda4605e891403832963b000000001976a914cf9e180ad4289a61cc0af0fb376ff7f9a04b8b2888ac00000000003012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac0141207ec67cdad799fcbb072b0dd8734b46e1ee1b3cf07c41ba32f29b745ae0ef89532348fee9ec7fa1e4a6224fcd54d9671452cb0a35b8fd0a5fdaba0447567b84d7").expect("expected to decode");
        let identity_top_up_second = hex::decode("04030000c601018fc4202f1b0a147582ceaefcf622929ae52d9e8a12644d44e59bf198189cbb7300000000c16956695f3911e34b560533ea7b2d1bc2e37e4b9e3c483c27d1038c7072604618cf6fd4c5423912f637a99e7cdc2723a42b6ee6d2ea450521a030319bcef330805526988577953190e7eba053d9ecee1a55e26c2a4fe6783d339cd215ae6dc98e43ed69c86cd601ebdcddccf51349e10e2b09a59a883660aac5ddbe02ea8cee5df6e67115fc040e23ae2316a7c632a662879656a8f61b72daf651898a740a18df03000000018fc4202f1b0a147582ceaefcf622929ae52d9e8a12644d44e59bf198189cbb73000000006b483045022100b421af2c60117484db254c4030a10bddde04a476f0cda484f41357a0074609aa0220328a17b66dff1d8a70ba1c5d4c445c21d45a7548cd09368ccfad3acccbf0f590012102a7dd6bf8188bf9d2aed01dd4c06981e6f8e7df94f79c6dcfa23d56c3787bf228ffffffff02f401000000000000166a14fc7a4f5bdd8d74f31309f1a4d7857484a2ac2ae834210000000000001976a914e13db56a35ee5cef6e40c7608d2fa1372f0af5b188ac00000000003012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac01411f8f5a83ada0b784e352bfc0b7030a4aea966c8cff62c07b9763928added5d72ac0d1cb965e6afcf478be7532ecd42ff87111f3e79d802becf7838aca79acc63ac").expect("expected to decode");
        let dpns_preorder_document = hex::decode("0201013012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac0100a027b57dde6f5ab32aea94cca60a5fbf72a9eb85b7514b3f4a1a23b86563bf74087072656f7264657200e668c659af66aee1e72c186dde7b5b7e0a1d712a09c40d5721f622bf53c5315501e668c659af66aee1e72c186dde7b5b7e0a1d712a09c40d5721f622bf53c5315500000000013468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374013012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac0206646f6d61696e160612047479706512066f626a6563741207696e64696365731503160312046e616d651212706172656e744e616d65416e644c6162656c120a70726f7065727469657315021601121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d6512036173631601120f6e6f726d616c697a65644c6162656c12036173631206756e697175651301160312046e616d65120e646173684964656e746974794964120a70726f7065727469657315011601121c7265636f7264732e64617368556e697175654964656e74697479496412036173631206756e697175651301160212046e616d65120964617368416c696173120a70726f7065727469657315011601121b7265636f7264732e64617368416c6961734964656e7469747949641203617363120a70726f70657274696573160612056c6162656c16051204747970651206737472696e6712077061747465726e122a5e5b612d7a412d5a302d395d5b612d7a412d5a302d392d5d7b302c36317d5b612d7a412d5a302d395d2412096d696e4c656e677468020312096d61784c656e677468023f120b6465736372697074696f6e1219446f6d61696e206c6162656c2e20652e672e2027426f62272e120f6e6f726d616c697a65644c6162656c16051204747970651206737472696e6712077061747465726e12215e5b612d7a302d395d5b612d7a302d392d5d7b302c36317d5b612d7a302d395d2412096d61784c656e677468023f120b6465736372697074696f6e1250446f6d61696e206c6162656c20696e206c6f7765726361736520666f7220636173652d696e73656e73697469766520756e697175656e6573732076616c69646174696f6e2e20652e672e2027626f6227120824636f6d6d656e7412694d75737420626520657175616c20746f20746865206c6162656c20696e206c6f776572636173652e20546869732070726f70657274792077696c6c20626520646570726563617465642064756520746f206361736520696e73656e73697469766520696e6469636573121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d6516061204747970651206737472696e6712077061747465726e12265e247c5e5b612d7a302d395d5b612d7a302d392d5c2e5d7b302c36317d5b612d7a302d395d2412096d696e4c656e677468020012096d61784c656e677468023f120b6465736372697074696f6e125e412066756c6c20706172656e7420646f6d61696e206e616d6520696e206c6f7765726361736520666f7220636173652d696e73656e73697469766520756e697175656e6573732076616c69646174696f6e2e20652e672e20276461736827120824636f6d6d656e74128c4d7573742065697468657220626520657175616c20746f20616e206578697374696e6720646f6d61696e206f7220656d70747920746f20637265617465206120746f70206c6576656c20646f6d61696e2e204f6e6c7920746865206461746120636f6e7472616374206f776e65722063616e2063726561746520746f70206c6576656c20646f6d61696e732e120c7072656f7264657253616c741605120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d730220120b6465736372697074696f6e122253616c74207573656420696e20746865207072656f7264657220646f63756d656e7412077265636f726473160612047479706512066f626a656374120a70726f706572746965731602121464617368556e697175654964656e7469747949641607120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d7302201210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572120b6465736372697074696f6e123e4964656e7469747920494420746f206265207573656420746f2063726561746520746865207072696d617279206e616d6520746865204964656e74697479120824636f6d6d656e7412234d75737420626520657175616c20746f2074686520646f63756d656e74206f776e6572121364617368416c6961734964656e7469747949641607120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d7302201210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572120b6465736372697074696f6e123d4964656e7469747920494420746f206265207573656420746f2063726561746520616c696173206e616d657320666f7220746865204964656e74697479120824636f6d6d656e7412234d75737420626520657175616c20746f2074686520646f63756d656e74206f776e6572120824636f6d6d656e741290436f6e73747261696e742077697468206d617820616e64206d696e2070726f7065727469657320656e737572652074686174206f6e6c79206f6e65206964656e74697479207265636f72642069732075736564202d206569746865722061206064617368556e697175654964656e74697479496460206f722061206064617368416c6961734964656e74697479496460120d6d696e50726f706572746965730201120d6d617850726f70657274696573020112146164646974696f6e616c50726f706572746965731300120e737562646f6d61696e52756c6573160512047479706512066f626a656374120a70726f706572746965731601120f616c6c6f77537562646f6d61696e7316031204747970651207626f6f6c65616e120b6465736372697074696f6e125b54686973206f7074696f6e20646566696e65732077686f2063616e2063726561746520737562646f6d61696e733a2074727565202d20616e796f6e653b2066616c7365202d206f6e6c792074686520646f6d61696e206f776e6572120824636f6d6d656e74124f4f6e6c792074686520646f6d61696e206f776e657220697320616c6c6f77656420746f2063726561746520737562646f6d61696e7320666f72206e6f6e20746f702d6c6576656c20646f6d61696e73120b6465736372697074696f6e1242537562646f6d61696e2072756c657320616c6c6f7720646f6d61696e206f776e65727320746f20646566696e652072756c657320666f7220737562646f6d61696e7312146164646974696f6e616c50726f706572746965731300120872657175697265641501120f616c6c6f77537562646f6d61696e7312087265717569726564150612056c6162656c120f6e6f726d616c697a65644c6162656c121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d65120c7072656f7264657253616c7412077265636f726473120e737562646f6d61696e52756c657312146164646974696f6e616c50726f706572746965731300120824636f6d6d656e7412fb0137496e206f7264657220746f207265676973746572206120646f6d61696e20796f75206e65656420746f206372656174652061207072656f726465722e20546865207072656f726465722073746570206973206e656564656420746f2070726576656e74206d616e2d696e2d7468652d6d6964646c652061747461636b732e206e6f726d616c697a65644c6162656c202b20272e27202b206e6f726d616c697a6564506172656e74446f6d61696e206d757374206e6f74206265206c6f6e676572207468616e20323533206368617273206c656e67746820617320646566696e65642062792052464320313033352e20446f6d61696e20646f63756d656e74732061726520696d6d757461626c653a206d6f64696669636174696f6e20616e642064656c6574696f6e206172652072657374726963746564087072656f72646572160612047479706512066f626a6563741207696e64696365731501160312046e616d65120a73616c74656448617368120a70726f7065727469657315011601121073616c746564446f6d61696e4861736812036173631206756e697175651301120a70726f706572746965731601121073616c746564446f6d61696e486173681605120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d730220120b6465736372697074696f6e1259446f75626c65207368612d323536206f662074686520636f6e636174656e6174696f6e206f66206120333220627974652072616e646f6d2073616c7420616e642061206e6f726d616c697a656420646f6d61696e206e616d65120872657175697265641501121073616c746564446f6d61696e4861736812146164646974696f6e616c50726f706572746965731300120824636f6d6d656e74124a5072656f7264657220646f63756d656e74732061726520696d6d757461626c653a206d6f64696669636174696f6e20616e642064656c6574696f6e20617265207265737472696374656400d802aae1a3f029af9ef35dd1f4388fab0f04aa459131fbf4561101302010d71c000001011073616c746564446f6d61696e486173680a2012e6b5cacd027fab24ddfb4755e76f8cf840072b44c8e850fec213472bdd217d010101411f49a8787acb91c42e6b32fbceae5191505398cdfe3a2c730cf14f7b8002f3c8bb4dbb0cf4db23f858f34e69b145f59d10056782dc404c1005349a34bb04f7cce6").expect("expected to decode");
        let dpns_domain_document = hex::decode("0201013012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac01005cfbbbd4226c16dafbbf5c614701bc2502603c5662a661ef38a8ae7b14ac6ead06646f6d61696e00e668c659af66aee1e72c186dde7b5b7e0a1d712a09c40d5721f622bf53c5315501e668c659af66aee1e72c186dde7b5b7e0a1d712a09c40d5721f622bf53c5315500000000013468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374013012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac0206646f6d61696e160612047479706512066f626a6563741207696e64696365731503160312046e616d651212706172656e744e616d65416e644c6162656c120a70726f7065727469657315021601121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d6512036173631601120f6e6f726d616c697a65644c6162656c12036173631206756e697175651301160312046e616d65120e646173684964656e746974794964120a70726f7065727469657315011601121c7265636f7264732e64617368556e697175654964656e74697479496412036173631206756e697175651301160212046e616d65120964617368416c696173120a70726f7065727469657315011601121b7265636f7264732e64617368416c6961734964656e7469747949641203617363120a70726f70657274696573160612056c6162656c16051204747970651206737472696e6712077061747465726e122a5e5b612d7a412d5a302d395d5b612d7a412d5a302d392d5d7b302c36317d5b612d7a412d5a302d395d2412096d696e4c656e677468020312096d61784c656e677468023f120b6465736372697074696f6e1219446f6d61696e206c6162656c2e20652e672e2027426f62272e120f6e6f726d616c697a65644c6162656c16051204747970651206737472696e6712077061747465726e12215e5b612d7a302d395d5b612d7a302d392d5d7b302c36317d5b612d7a302d395d2412096d61784c656e677468023f120b6465736372697074696f6e1250446f6d61696e206c6162656c20696e206c6f7765726361736520666f7220636173652d696e73656e73697469766520756e697175656e6573732076616c69646174696f6e2e20652e672e2027626f6227120824636f6d6d656e7412694d75737420626520657175616c20746f20746865206c6162656c20696e206c6f776572636173652e20546869732070726f70657274792077696c6c20626520646570726563617465642064756520746f206361736520696e73656e73697469766520696e6469636573121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d6516061204747970651206737472696e6712077061747465726e12265e247c5e5b612d7a302d395d5b612d7a302d392d5c2e5d7b302c36317d5b612d7a302d395d2412096d696e4c656e677468020012096d61784c656e677468023f120b6465736372697074696f6e125e412066756c6c20706172656e7420646f6d61696e206e616d6520696e206c6f7765726361736520666f7220636173652d696e73656e73697469766520756e697175656e6573732076616c69646174696f6e2e20652e672e20276461736827120824636f6d6d656e74128c4d7573742065697468657220626520657175616c20746f20616e206578697374696e6720646f6d61696e206f7220656d70747920746f20637265617465206120746f70206c6576656c20646f6d61696e2e204f6e6c7920746865206461746120636f6e7472616374206f776e65722063616e2063726561746520746f70206c6576656c20646f6d61696e732e120c7072656f7264657253616c741605120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d730220120b6465736372697074696f6e122253616c74207573656420696e20746865207072656f7264657220646f63756d656e7412077265636f726473160612047479706512066f626a656374120a70726f706572746965731602121464617368556e697175654964656e7469747949641607120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d7302201210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572120b6465736372697074696f6e123e4964656e7469747920494420746f206265207573656420746f2063726561746520746865207072696d617279206e616d6520746865204964656e74697479120824636f6d6d656e7412234d75737420626520657175616c20746f2074686520646f63756d656e74206f776e6572121364617368416c6961734964656e7469747949641607120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d7302201210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572120b6465736372697074696f6e123d4964656e7469747920494420746f206265207573656420746f2063726561746520616c696173206e616d657320666f7220746865204964656e74697479120824636f6d6d656e7412234d75737420626520657175616c20746f2074686520646f63756d656e74206f776e6572120824636f6d6d656e741290436f6e73747261696e742077697468206d617820616e64206d696e2070726f7065727469657320656e737572652074686174206f6e6c79206f6e65206964656e74697479207265636f72642069732075736564202d206569746865722061206064617368556e697175654964656e74697479496460206f722061206064617368416c6961734964656e74697479496460120d6d696e50726f706572746965730201120d6d617850726f70657274696573020112146164646974696f6e616c50726f706572746965731300120e737562646f6d61696e52756c6573160512047479706512066f626a656374120a70726f706572746965731601120f616c6c6f77537562646f6d61696e7316031204747970651207626f6f6c65616e120b6465736372697074696f6e125b54686973206f7074696f6e20646566696e65732077686f2063616e2063726561746520737562646f6d61696e733a2074727565202d20616e796f6e653b2066616c7365202d206f6e6c792074686520646f6d61696e206f776e6572120824636f6d6d656e74124f4f6e6c792074686520646f6d61696e206f776e657220697320616c6c6f77656420746f2063726561746520737562646f6d61696e7320666f72206e6f6e20746f702d6c6576656c20646f6d61696e73120b6465736372697074696f6e1242537562646f6d61696e2072756c657320616c6c6f7720646f6d61696e206f776e65727320746f20646566696e652072756c657320666f7220737562646f6d61696e7312146164646974696f6e616c50726f706572746965731300120872657175697265641501120f616c6c6f77537562646f6d61696e7312087265717569726564150612056c6162656c120f6e6f726d616c697a65644c6162656c121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d65120c7072656f7264657253616c7412077265636f726473120e737562646f6d61696e52756c657312146164646974696f6e616c50726f706572746965731300120824636f6d6d656e7412fb0137496e206f7264657220746f207265676973746572206120646f6d61696e20796f75206e65656420746f206372656174652061207072656f726465722e20546865207072656f726465722073746570206973206e656564656420746f2070726576656e74206d616e2d696e2d7468652d6d6964646c652061747461636b732e206e6f726d616c697a65644c6162656c202b20272e27202b206e6f726d616c697a6564506172656e74446f6d61696e206d757374206e6f74206265206c6f6e676572207468616e20323533206368617273206c656e67746820617320646566696e65642062792052464320313033352e20446f6d61696e20646f63756d656e74732061726520696d6d757461626c653a206d6f64696669636174696f6e20616e642064656c6574696f6e206172652072657374726963746564087072656f72646572160612047479706512066f626a6563741207696e64696365731501160312046e616d65120a73616c74656448617368120a70726f7065727469657315011601121073616c746564446f6d61696e4861736812036173631206756e697175651301120a70726f706572746965731601121073616c746564446f6d61696e486173681605120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d730220120b6465736372697074696f6e1259446f75626c65207368612d323536206f662074686520636f6e636174656e6174696f6e206f66206120333220627974652072616e646f6d2073616c7420616e642061206e6f726d616c697a656420646f6d61696e206e616d65120872657175697265641501121073616c746564446f6d61696e4861736812146164646974696f6e616c50726f706572746965731300120824636f6d6d656e74124a5072656f7264657220646f63756d656e74732061726520696d6d757461626c653a206d6f64696669636174696f6e20616e642064656c6574696f6e206172652072657374726963746564000e3fc8950e82fb0ccbd0ed86e593bab85212bcf9bc012f92980bb4aa2532aafb00000106056c6162656c121437396661333531626562343362303736653538380f6e6f726d616c697a65644c6162656c121437396661333531626562343362303736653538381a6e6f726d616c697a6564506172656e74446f6d61696e4e616d6512000c7072656f7264657253616c740a2004e085d9967c0081bb616ef2df8d993c9d315853ef49dee44ea5367e604870f4077265636f7264731601121364617368416c6961734964656e746974794964103012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac0e737562646f6d61696e52756c65731601120f616c6c6f77537562646f6d61696e731301010101411fb015b8f9a09a4b9199fd1eea97ff210be521794c4d42b08fe2369fc298977acf3887ac514c4688ae1fd1d5277c7164e3a4758e4d4c3e91759e81a8d4261140b4").expect("expected to decode");

        let mut config = PlatformConfig::default();

        config.abci.keys.dpns_master_public_key =
            base64::decode("A08spLsoYnvWMLhBiaK+G4pybyu+5NmJefh3MdMfqkg/")
                .expect("expected to decode");
        config.abci.keys.dpns_second_public_key =
            base64::decode("A47zP5s3jmW7dPBzmzEXieUNnM34YPtDR1FpbqMaAVFR")
                .expect("expected to decode");

        let platform = TestPlatformBuilder::new()
            .with_config(config)
            .build_with_mock_rpc();

        let platform_state = platform.state.read().unwrap();
        let platform_version = platform_state.current_platform_version().unwrap();

        let genesis_time = 0;

        let system_identity_public_keys_v0: SystemIdentityPublicKeysV0 =
            platform.config.abci.keys.clone().into();

        platform
            .create_genesis_state_v0(
                genesis_time,
                system_identity_public_keys_v0.into(),
                None,
                platform_version,
            )
            .expect("expected to create genesis state");

        let transaction = platform.drive.grove.start_transaction();

        let validation_result = platform
            .execute_tx(identity_top_up_first, &BlockInfo::default(), &transaction)
            .expect("expected to execute identity_top_up_first tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        let validation_result = platform
            .execute_tx(identity_top_up_second, &BlockInfo::default(), &transaction)
            .expect("expected to execute identity_top_up_second tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        let validation_result = platform
            .execute_tx(dpns_preorder_document, &BlockInfo::default(), &transaction)
            .expect("expected to execute identity_create tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");

        let validation_result = platform
            .check_tx_v0(dpns_domain_document.as_slice())
            .expect("expected to check tx");

        assert!(validation_result.errors.is_empty());

        let transaction = platform.drive.grove.start_transaction();

        let validation_result = platform
            .execute_tx(dpns_domain_document, &BlockInfo::default(), &transaction)
            .expect("expected to execute identity top up tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");
    }

    #[test]
    fn identity_update_with_non_master_key_check_tx() {
        let mut config = PlatformConfig::default();

        let mut rng = StdRng::seed_from_u64(1);

        let secp = Secp256k1::new();

        let master_key_pair = KeyPair::new(&secp, &mut rng);

        let _master_secret_key = master_key_pair.secret_key();

        let master_public_key = master_key_pair.public_key();

        config.abci.keys.dpns_master_public_key = master_public_key.serialize().to_vec();

        let high_key_pair = KeyPair::new(&secp, &mut rng);

        let high_secret_key = high_key_pair.secret_key();

        let high_public_key = high_key_pair.public_key();

        config.abci.keys.dpns_second_public_key = high_public_key.serialize().to_vec();

        let platform = TestPlatformBuilder::new()
            .with_config(config)
            .build_with_mock_rpc();

        let platform_state = platform.state.read().unwrap();
        let platform_version = platform_state.current_platform_version().unwrap();

        let genesis_time = 0;

        let system_identity_public_keys_v0: SystemIdentityPublicKeysV0 =
            platform.config.abci.keys.clone().into();

        platform
            .create_genesis_state_v0(
                genesis_time,
                system_identity_public_keys_v0.into(),
                None,
                platform_version,
            )
            .expect("expected to create genesis state");

        let new_key_pair = KeyPair::new(&secp, &mut rng);

        let mut new_key = IdentityPublicKeyInCreationV0 {
            id: 2,
            purpose: Purpose::AUTHENTICATION,
            security_level: SecurityLevel::HIGH,
            key_type: KeyType::ECDSA_SECP256K1,
            read_only: false,
            data: new_key_pair.public_key().serialize().to_vec().into(),
            signature: Default::default(),
        };

        let signable_bytes = new_key
            .signable_bytes()
            .expect("expected to get signable bytes");
        let secret = new_key_pair.secret_key();
        let signature =
            signer::sign(&signable_bytes, &secret.secret_bytes()).expect("expected to sign");

        new_key.signature = signature.to_vec().into();

        let mut update_transition = IdentityUpdateTransitionV0 {
            identity_id: dpns_contract::OWNER_ID_BYTES.into(),
            revision: 0,
            add_public_keys: vec![IdentityPublicKeyInCreation::V0(new_key)],
            disable_public_keys: vec![],
            public_keys_disabled_at: None,
            signature_public_key_id: 1,
            signature: Default::default(),
        };

        let signature = signer::sign(
            &update_transition
                .signable_bytes()
                .expect("expected signable bytes"),
            &high_secret_key.secret_bytes(),
        )
        .expect("expected to sign");

        update_transition.signature = signature.to_vec().into();

        let transition: StateTransition = update_transition.into();

        let update_transition_bytes = transition.serialize().expect("expected to serialize");

        let validation_result = platform
            .check_tx_v0(update_transition_bytes.as_slice())
            .expect("expected to execute identity top up tx");

        // Only master keys can sign an update

        validation_result.errors.first().expect("expected an error");
    }
}
