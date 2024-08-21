#[cfg(test)]
mod refund_tests {
    use once_cell::sync::Lazy;
use std::borrow::Cow;
    use std::collections::BTreeMap;
    use bs58;
use dpp::platform_value;
use crate::execution::validation::state_transition::tests::{
        fetch_expected_identity_balance, process_state_transitions,
        setup_identity_with_system_credits,
    };
    use crate::platform_types::platform_state::v0::PlatformStateV0Methods;
    use crate::rpc::core::MockCoreRPCLike;
    use crate::test::helpers::fast_forward_to_block::fast_forward_to_block;
    use crate::test::helpers::setup::{TempPlatform, TestPlatformBuilder};
    use dpp::block::block_info::BlockInfo;
    use dpp::dash_to_credits;
    use dpp::data_contract::accessors::v0::DataContractV0Getters;
    use dpp::data_contract::conversion::value::v0::DataContractValueConversionMethodsV0;
    use dpp::data_contract::document_type::accessors::DocumentTypeV0Getters;
    use dpp::data_contract::document_type::random_document::{
        CreateRandomDocument, DocumentFieldFillSize, DocumentFieldFillType,
    };
    use dpp::data_contract::document_type::DocumentTypeRef;
    use dpp::document::document_methods::DocumentMethodsV0;
    use dpp::document::serialization_traits::DocumentPlatformConversionMethodsV0;
    use dpp::document::{Document, DocumentV0Getters, DocumentV0Setters};
    use dpp::fee::fee_result::FeeResult;
    use dpp::fee::Credits;
    use dpp::identity::accessors::IdentityGettersV0;
    use dpp::identity::{Identity, IdentityPublicKey};
    use dpp::platform_value::{platform_value, Identifier, Value, Bytes32};
    use dpp::state_transition::documents_batch_transition::methods::v0::DocumentsBatchTransitionMethodsV0;
    use dpp::state_transition::documents_batch_transition::DocumentsBatchTransition;
    use drive::util::test_helpers::setup_contract;
    use platform_version::version::PlatformVersion;
    use rand::prelude::StdRng;
    use rand::SeedableRng;
    use simple_signer::signer::SimpleSigner;
    use std::ops::Deref;
    use dpp::block::epoch::EpochIndex;
    use dpp::dashcore::Network;
    use dpp::data_contract::DataContract;
    use dpp::prelude::CoreBlockHeight;
    use drive::util::storage_flags::StorageFlags;
    use dpp::document::serialization_traits::DocumentPlatformValueMethodsV0;
    use dpp::fee::default_costs::CachedEpochIndexFeeVersions;
    use drive::util::object_size_info::{DocumentAndContractInfo, OwnedDocumentInfo};
    use drive::util::object_size_info::DocumentInfo::DocumentOwnedInfo;

    static EPOCH_CHANGE_FEE_VERSION_TEST: Lazy<CachedEpochIndexFeeVersions> =
        Lazy::new(|| BTreeMap::from([(0, PlatformVersion::first().fee_version.clone())]));



    // There's a fee for the first document that a user creates on a contract as they add space
    // For the identity data contract nonce
    fn setup_join_contract_document<'a>(
        platform: &TempPlatform<MockCoreRPCLike>,
        profile: DocumentTypeRef,
        rng: &mut StdRng,
        identity: &'a Identity,
        key: &IdentityPublicKey,
        signer: &SimpleSigner,
    ) -> Credits {
        let platform_version = PlatformVersion::latest();

        let platform_state = platform.state.load();

        assert!(profile.documents_mutable());

        let entropy = Bytes32::random_with_rng(rng);

        let mut document = profile
            .random_document_with_identifier_and_entropy(
                rng,
                identity.id(),
                entropy,
                DocumentFieldFillType::FillIfNotRequired,
                DocumentFieldFillSize::AnyDocumentFillSize,
                platform_version,
            )
            .expect("expected a random document");

        document.set("avatarUrl", "http://test.com/ivan.jpg".into());

        let mut altered_document = document.clone();

        altered_document.increment_revision().unwrap();
        altered_document.set("displayName", "Ivan".into());
        altered_document.set("avatarUrl", "http://test.com/dog.jpg".into());

        let documents_batch_create_transition =
            DocumentsBatchTransition::new_document_creation_transition_from_document(
                document.clone(),
                profile,
                entropy.0,
                key,
                2,
                0,
                signer,
                platform_version,
                None,
                None,
                None,
            )
            .expect("expect to create documents batch transition");

        let (mut fee_results, processed_block_fee_outcome) = process_state_transitions(
            &platform,
            &vec![documents_batch_create_transition.clone()],
            BlockInfo::default(),
            &platform_state,
        );

        let fee_result = fee_results.remove(0);

        let credits_verified = platform
            .platform
            .drive
            .calculate_total_credits_balance(None, &platform_version.drive)
            .expect("expected to check sum trees");

        let balanced = credits_verified
            .ok()
            .expect("expected that credits will balance when we remove in same block");

        assert!(balanced, "platform should be balanced {}", credits_verified);

        assert_eq!(
            fee_result.storage_fee,
            processed_block_fee_outcome.fees_in_pools.storage_fees
        );

        assert_eq!(
            fee_result.processing_fee,
            processed_block_fee_outcome.fees_in_pools.processing_fees
        );

        let expected_user_balance_after_creation =
            dash_to_credits!(1) - fee_result.total_base_fee();

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            expected_user_balance_after_creation,
        );

        expected_user_balance_after_creation
    }

    fn setup_initial_document<'a>(
        platform: &TempPlatform<MockCoreRPCLike>,
        profile: DocumentTypeRef,
        rng: &mut StdRng,
        identity: &'a Identity,
        key: &IdentityPublicKey,
        signer: &SimpleSigner,
    ) -> (Document, FeeResult, Credits) {
        // Let's make another document first just so the operations of joining a contract are out of the way
        // (A user pays to add some data to the state on the first time they make their first document for a contract)
        let user_credits_left =
            setup_join_contract_document(platform, profile, rng, identity, key, signer);

        let platform_version = PlatformVersion::latest();

        let platform_state = platform.state.load();

        assert!(profile.documents_mutable());

        let entropy = Bytes32::random_with_rng(rng);

        let mut document = profile
            .random_document_with_identifier_and_entropy(
                rng,
                identity.id(),
                entropy,
                DocumentFieldFillType::FillIfNotRequired,
                DocumentFieldFillSize::AnyDocumentFillSize,
                platform_version,
            )
            .expect("expected a random document");

        document.set("avatarUrl", "http://test.com/bob.jpg".into());

        let mut altered_document = document.clone();

        altered_document.increment_revision().unwrap();
        altered_document.set("displayName", "Samuel".into());
        altered_document.set("avatarUrl", "http://test.com/cat.jpg".into());

        let serialized_len = document
            .serialize(profile, platform_version)
            .expect("expected to serialize")
            .len() as u64;

        assert_eq!(serialized_len, 173);

        let documents_batch_create_transition =
            DocumentsBatchTransition::new_document_creation_transition_from_document(
                document.clone(),
                profile,
                entropy.0,
                key,
                3,
                0,
                signer,
                platform_version,
                None,
                None,
                None,
            )
            .expect("expect to create documents batch transition");

        let (mut fee_results, _) = process_state_transitions(
            &platform,
            &vec![documents_batch_create_transition.clone()],
            BlockInfo::default(),
            &platform_state,
        );

        let fee_result = fee_results.remove(0);

        let credits_verified = platform
            .platform
            .drive
            .calculate_total_credits_balance(None, &platform_version.drive)
            .expect("expected to check sum trees");

        let balanced = credits_verified
            .ok()
            .expect("expected that credits will balance when we remove in same block");

        assert!(balanced, "platform should be balanced {}", credits_verified);

        assert_eq!(fee_result.storage_fee, 11124000);

        let added_bytes = fee_result.storage_fee
            / platform_version
                .fee_version
                .storage
                .storage_disk_usage_credit_per_byte;

        // Key -> 65 bytes
        // 32 bytes for the key prefix
        // 32 bytes for the unique id
        // 1 byte for key_size (required space for 64)

        // Value -> 279
        //   1 for the flag option with flags
        //   1 for the flags size
        //   35 for flags 32 + 1 + 2
        //   1 for the enum type
        //   1 for item
        //   173 for item serialized bytes (verified above)
        //   1 for Basic Merk
        // 32 for node hash
        // 32 for value hash
        // 2 byte for the value_size (required space for above 128)

        // Parent Hook -> 68
        // Key Bytes 32
        // Hash Size 32
        // Key Length 1
        // Child Heights 2
        // Basic Merk 1

        assert_eq!(added_bytes, 65 + 279 + 68);

        let expected_user_balance_after_creation = user_credits_left - fee_result.total_base_fee();

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            expected_user_balance_after_creation,
        );

        (document, fee_result, expected_user_balance_after_creation)
    }

    #[test]
    fn test_document_refund_immediate() {
        let platform_version = PlatformVersion::latest();
        let mut platform = TestPlatformBuilder::new()
            .build_with_mock_rpc()
            .set_genesis_state();

        let dashpay_contract_no_indexes = setup_contract(
            &platform.drive,
            "tests/supporting_files/contract/dashpay/dashpay-contract-no-indexes.json",
            None,
            None,
        );

        let profile = dashpay_contract_no_indexes
            .document_type_for_name("profile")
            .expect("expected a profile document type");

        let mut rng = StdRng::seed_from_u64(433);

        let platform_state = platform.state.load();

        let (identity, signer, key) =
            setup_identity_with_system_credits(&mut platform, 958, dash_to_credits!(1));

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            dash_to_credits!(1),
        );

        let (document, insertion_fee_result, current_user_balance) =
            setup_initial_document(&platform, profile, &mut rng, &identity, &key, &signer);

        let documents_batch_delete_transition =
            DocumentsBatchTransition::new_document_deletion_transition_from_document(
                document,
                profile,
                &key,
                4,
                0,
                &signer,
                platform_version,
                None,
                None,
                None,
            )
            .expect("expect to create documents batch transition");

        let (mut fee_results, _) = process_state_transitions(
            &platform,
            &vec![documents_batch_delete_transition.clone()],
            BlockInfo::default(),
            &platform_state,
        );

        let fee_result = fee_results.remove(0);

        let credits_verified = platform
            .platform
            .drive
            .calculate_total_credits_balance(None, &platform_version.drive)
            .expect("expected to check sum trees");

        let balanced = credits_verified
            .ok()
            .expect("expected that credits will balance when we remove in same block");

        assert!(balanced, "platform should be balanced {}", credits_verified);

        let refund_amount = fee_result
            .fee_refunds
            .calculate_refunds_amount_for_identity(identity.id())
            .expect("expected refunds for identity");

        // we should be refunding more than 99%
        let lower_bound = insertion_fee_result.storage_fee * 99 / 100;
        assert!(refund_amount > lower_bound, "expected the refund amount to be more than 99% of the storage cost, as it is for just one out of 2000 epochs");
        assert!(
            refund_amount < insertion_fee_result.storage_fee,
            "expected the refund amount to be less than the insertion cost"
        );

        assert_eq!(fee_result.storage_fee, 0);

        let expected_user_balance_after_deletion =
            current_user_balance - fee_result.total_base_fee() + refund_amount;

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            expected_user_balance_after_deletion,
        );
    }
    #[test]
    fn test_document_refund_after_an_epoch() {
        let platform_version = PlatformVersion::latest();
        let mut platform = TestPlatformBuilder::new()
            .build_with_mock_rpc()
            .set_genesis_state();

        let dashpay_contract_no_indexes = setup_contract(
            &platform.drive,
            "tests/supporting_files/contract/dashpay/dashpay-contract-no-indexes.json",
            None,
            None,
        );

        let profile = dashpay_contract_no_indexes
            .document_type_for_name("profile")
            .expect("expected a profile document type");

        let mut rng = StdRng::seed_from_u64(433);

        let (identity, signer, key) =
            setup_identity_with_system_credits(&mut platform, 958, dash_to_credits!(1));

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            dash_to_credits!(1),
        );

        let (document, insertion_fee_result, current_user_balance) =
            setup_initial_document(&platform, profile, &mut rng, &identity, &key, &signer);

        fast_forward_to_block(&platform, 1_200_000_000, 900, 42, 1, false); //next epoch

        let documents_batch_delete_transition =
            DocumentsBatchTransition::new_document_deletion_transition_from_document(
                document,
                profile,
                &key,
                4,
                0,
                &signer,
                platform_version,
                None,
                None,
                None,
            )
            .expect("expect to create documents batch transition");

        let platform_state = platform.state.load();

        let (mut fee_results, _) = process_state_transitions(
            &platform,
            &vec![documents_batch_delete_transition.clone()],
            *platform_state.last_block_info(),
            &platform_state,
        );

        let fee_result = fee_results.remove(0);

        let credits_verified = platform
            .platform
            .drive
            .calculate_total_credits_balance(None, &platform_version.drive)
            .expect("expected to check sum trees");

        let balanced = credits_verified
            .ok()
            .expect("expected that credits will balance when we remove in same block");

        assert!(balanced, "platform should be balanced {}", credits_verified);

        let refund_amount = fee_result
            .fee_refunds
            .calculate_refunds_amount_for_identity(identity.id())
            .expect("expected refunds for identity");

        // we should be refunding more than 99% still
        let lower_bound = insertion_fee_result.storage_fee * 99 / 100;
        assert!(refund_amount > lower_bound, "expected the refund amount to be more than 99% of the storage cost, as it is for just one out of 2000 epochs");
        assert!(
            refund_amount < insertion_fee_result.storage_fee,
            "expected the refund amount to be less than the insertion cost"
        );

        assert_eq!(fee_result.storage_fee, 0);

        let expected_user_balance_after_deletion =
            current_user_balance - fee_result.total_base_fee() + refund_amount;

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            expected_user_balance_after_deletion,
        );
    }

    #[test]
    fn test_document_refund_after_a_year() {
        let platform_version = PlatformVersion::latest();
        let mut platform = TestPlatformBuilder::new()
            .build_with_mock_rpc()
            .set_genesis_state();

        let dashpay_contract_no_indexes = setup_contract(
            &platform.drive,
            "tests/supporting_files/contract/dashpay/dashpay-contract-no-indexes.json",
            None,
            None,
        );

        let profile = dashpay_contract_no_indexes
            .document_type_for_name("profile")
            .expect("expected a profile document type");

        let mut rng = StdRng::seed_from_u64(433);

        let (identity, signer, key) =
            setup_identity_with_system_credits(&mut platform, 958, dash_to_credits!(1));

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            dash_to_credits!(1),
        );

        let (document, insertion_fee_result, current_user_balance) =
            setup_initial_document(&platform, profile, &mut rng, &identity, &key, &signer);

        fast_forward_to_block(&platform, 1_200_000_000, 900, 42, 40, false); //a year later

        let documents_batch_delete_transition =
            DocumentsBatchTransition::new_document_deletion_transition_from_document(
                document,
                profile,
                &key,
                4,
                0,
                &signer,
                platform_version,
                None,
                None,
                None,
            )
            .expect("expect to create documents batch transition");

        let platform_state = platform.state.load();

        let (mut fee_results, _) = process_state_transitions(
            &platform,
            &vec![documents_batch_delete_transition.clone()],
            *platform_state.last_block_info(),
            &platform_state,
        );

        let fee_result = fee_results.remove(0);

        let credits_verified = platform
            .platform
            .drive
            .calculate_total_credits_balance(None, &platform_version.drive)
            .expect("expected to check sum trees");

        let balanced = credits_verified
            .ok()
            .expect("expected that credits will balance when we remove in same block");

        assert!(balanced, "platform should be balanced {}", credits_verified);

        let refund_amount = fee_result
            .fee_refunds
            .calculate_refunds_amount_for_identity(identity.id())
            .expect("expected refunds for identity");

        // we should be refunding around 94% after a year.
        let refunded_percentage = refund_amount * 100 / insertion_fee_result.storage_fee;
        assert_eq!(refunded_percentage, 94);

        assert_eq!(fee_result.storage_fee, 0);

        let expected_user_balance_after_deletion =
            current_user_balance - fee_result.total_base_fee() + refund_amount;

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            expected_user_balance_after_deletion,
        );
    }

    #[test]
    fn test_document_refund_after_25_years() {
        let platform_version = PlatformVersion::latest();
        let mut platform = TestPlatformBuilder::new()
            .build_with_mock_rpc()
            .set_genesis_state();

        let dashpay_contract_no_indexes = setup_contract(
            &platform.drive,
            "tests/supporting_files/contract/dashpay/dashpay-contract-no-indexes.json",
            None,
            None,
        );

        let profile = dashpay_contract_no_indexes
            .document_type_for_name("profile")
            .expect("expected a profile document type");

        let mut rng = StdRng::seed_from_u64(433);

        let (identity, signer, key) =
            setup_identity_with_system_credits(&mut platform, 958, dash_to_credits!(1));

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            dash_to_credits!(1),
        );

        let (document, insertion_fee_result, current_user_balance) =
            setup_initial_document(&platform, profile, &mut rng, &identity, &key, &signer);

        fast_forward_to_block(&platform, 10_200_000_000, 9000, 42, 40 * 25, false); //25 years later

        let documents_batch_delete_transition =
            DocumentsBatchTransition::new_document_deletion_transition_from_document(
                document,
                profile,
                &key,
                4,
                0,
                &signer,
                platform_version,
                None,
                None,
                None,
            )
            .expect("expect to create documents batch transition");

        let platform_state = platform.state.load();

        let (mut fee_results, _) = process_state_transitions(
            &platform,
            &vec![documents_batch_delete_transition.clone()],
            *platform_state.last_block_info(),
            &platform_state,
        );

        let fee_result = fee_results.remove(0);

        let credits_verified = platform
            .platform
            .drive
            .calculate_total_credits_balance(None, &platform_version.drive)
            .expect("expected to check sum trees");

        let balanced = credits_verified
            .ok()
            .expect("expected that credits will balance when we remove in same block");

        assert!(balanced, "platform should be balanced {}", credits_verified);

        let refund_amount = fee_result
            .fee_refunds
            .calculate_refunds_amount_for_identity(identity.id())
            .expect("expected refunds for identity");

        // we should be refunding around 21% after 25 years.
        let refunded_percentage = refund_amount * 100 / insertion_fee_result.storage_fee;
        assert_eq!(refunded_percentage, 21);

        assert_eq!(fee_result.storage_fee, 0);

        let expected_user_balance_after_deletion =
            current_user_balance - fee_result.total_base_fee() + refund_amount;

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            expected_user_balance_after_deletion,
        );
    }

    #[test]
    fn test_document_refund_after_50_years() {
        let platform_version = PlatformVersion::latest();
        let mut platform = TestPlatformBuilder::new()
            .build_with_mock_rpc()
            .set_genesis_state();

        let dashpay_contract_no_indexes = setup_contract(
            &platform.drive,
            "tests/supporting_files/contract/dashpay/dashpay-contract-no-indexes.json",
            None,
            None,
        );

        let profile = dashpay_contract_no_indexes
            .document_type_for_name("profile")
            .expect("expected a profile document type");

        let mut rng = StdRng::seed_from_u64(433);

        let (identity, signer, key) =
            setup_identity_with_system_credits(&mut platform, 958, dash_to_credits!(1));

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            dash_to_credits!(1),
        );

        let (document, _, current_user_balance) =
            setup_initial_document(&platform, profile, &mut rng, &identity, &key, &signer);

        fast_forward_to_block(&platform, 10_200_000_000, 9000, 42, 40 * 50, true); //50 years later

        let documents_batch_delete_transition =
            DocumentsBatchTransition::new_document_deletion_transition_from_document(
                document,
                profile,
                &key,
                4,
                0,
                &signer,
                platform_version,
                None,
                None,
                None,
            )
            .expect("expect to create documents batch transition");

        let platform_state = platform.state.load();

        let (mut fee_results, _) = process_state_transitions(
            &platform,
            &vec![documents_batch_delete_transition.clone()],
            *platform_state.last_block_info(),
            &platform_state,
        );

        let fee_result = fee_results.remove(0);

        let credits_verified = platform
            .platform
            .drive
            .calculate_total_credits_balance(None, &platform_version.drive)
            .expect("expected to check sum trees");

        let balanced = credits_verified
            .ok()
            .expect("expected that credits will balance when we remove in same block");

        assert!(balanced, "platform should be balanced {}", credits_verified);

        let refund_amount = fee_result
            .fee_refunds
            .calculate_refunds_amount_for_identity(identity.id())
            .expect("expected refunds for identity");

        // we should be refunding nothing after 50 years.
        assert_eq!(refund_amount, 0);

        assert_eq!(fee_result.storage_fee, 0);

        let expected_user_balance_after_deletion =
            current_user_balance - fee_result.total_base_fee() + refund_amount;

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            expected_user_balance_after_deletion,
        );
    }

    #[test]
    fn test_document_refund_after_10_epochs_on_different_fee_version_increasing_fees() {
        let platform_version = PlatformVersion::latest();
        let platform_version_with_higher_fees = platform_version.clone();

        let mut platform = TestPlatformBuilder::new()
            .build_with_mock_rpc()
            .set_genesis_state();

        let dashpay_contract_no_indexes = setup_contract(
            &platform.drive,
            "tests/supporting_files/contract/dashpay/dashpay-contract-no-indexes.json",
            None,
            None,
        );

        let profile = dashpay_contract_no_indexes
            .document_type_for_name("profile")
            .expect("expected a profile document type");

        let mut rng = StdRng::seed_from_u64(433);

        let (identity, signer, key) =
            setup_identity_with_system_credits(&mut platform, 958, dash_to_credits!(1));

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            platform_version,
            dash_to_credits!(1),
        );

        let (document, insertion_fee_result, current_user_balance) =
            setup_initial_document(&platform, profile, &mut rng, &identity, &key, &signer);

        fast_forward_to_block(&platform, 1_200_000_000, 900, 42, 10, false); //next epoch

        let documents_batch_delete_transition =
            DocumentsBatchTransition::new_document_deletion_transition_from_document(
                document,
                profile,
                &key,
                4,
                0,
                &signer,
                &platform_version_with_higher_fees,
                None,
                None,
                None,
            )
            .expect("expect to create documents batch transition");

        let mut platform_state = platform.state.load().clone().deref().clone();

        platform_state
            .previous_fee_versions_mut()
            .insert(5, platform_version_with_higher_fees.fee_version.clone());

        let (mut fee_results, _) = process_state_transitions(
            &platform,
            &vec![documents_batch_delete_transition.clone()],
            *platform_state.last_block_info(),
            &platform_state,
        );

        let fee_result = fee_results.remove(0);

        let credits_verified = platform
            .platform
            .drive
            .calculate_total_credits_balance(None, &platform_version_with_higher_fees.drive)
            .expect("expected to check sum trees");

        let balanced = credits_verified
            .ok()
            .expect("expected that credits will balance when we remove in same block");

        assert!(balanced, "platform should be balanced {}", credits_verified);

        let refund_amount = fee_result
            .fee_refunds
            .calculate_refunds_amount_for_identity(identity.id())
            .expect("expected refunds for identity");

        println!("{}", insertion_fee_result.storage_fee);
        println!("{}", refund_amount);

        // we should be refunding around 21% after 25 years.
        let refunded_percentage = refund_amount * 100 / insertion_fee_result.storage_fee;
        assert_eq!(refunded_percentage, 98);

        assert_eq!(fee_result.storage_fee, 0);

        let expected_user_balance_after_deletion =
            current_user_balance - fee_result.total_base_fee() + refund_amount;

        fetch_expected_identity_balance(
            &platform,
            identity.id(),
            &platform_version_with_higher_fees,
            expected_user_balance_after_deletion,
        );
    }

    #[test]
    fn test_document_storage_flags() {
        let mut epoch_index: EpochIndex = 1;
        let activation_core_height: CoreBlockHeight = 100;
        let epoch_core_start_height: CoreBlockHeight = 100;
        let current_core_height: CoreBlockHeight = 100;

        let (mut platform, _state, platform_version) =
            crate::query::tests::setup_platform(Some((1, activation_core_height)), Network::Regtest);

        let contract = platform_value!({
            "$format_version": "0",
            "id": "BZUodcFoFL6KvnonehrnMVggTvCe8W5MiRnZuqLb6M54",
            "schema": "https://schema.dash.org/dpp-0-4-0/meta/data-contract",
            "version": 1,
            "ownerId": "GZVdTnLFAN2yE9rLeCHBDBCr7YQgmXJuoExkY347j7Z5",
            "documentSchemas": {
                "indexedDocument": {
                    "type": "object",
                    "indices": [
                        {"name":"index1", "properties": [{"$ownerId":"asc"}, {"firstName":"desc"}], "unique":true},
                        {"name":"index2", "properties": [{"$ownerId":"asc"}, {"lastName":"desc"}], "unique":true},
                        {"name":"index3", "properties": [{"lastName":"asc"}]},
                        {"name":"index4", "properties": [{"$createdAt":"asc"}, {"$updatedAt":"asc"}]},
                        {"name":"index5", "properties": [{"$updatedAt":"asc"}]},
                        {"name":"index6", "properties": [{"$createdAt":"asc"}]}
                    ],
                    "properties":{
                        "firstName": {
                            "type": "string",
                            "maxLength": 63,
                            "position": 0,
                        },
                        "lastName": {
                            "type": "string",
                            "maxLength": 63,
                            "position": 1,
                        }
                    },
                    "required": ["firstName", "$createdAt", "$updatedAt", "lastName"],
                    "additionalProperties": false,
                },
            },
        });

        // first we need to deserialize the contract
        let contract = DataContract::from_value(contract, false, platform_version)
            .expect("expected data contract");

        platform.drive
            .apply_contract(
                &contract,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                None,
                platform_version,
            )
            .expect("should create a contract");

        fast_forward_to_block(
            &platform,
            5000 * epoch_index as u64,
            activation_core_height as u64,
            epoch_core_start_height,
            epoch_index,
            true,
        );

        let document_values = platform_value!({
           "$id": Identifier::new(bs58::decode("DLRWw2eRbLAW5zDU2c7wwsSFQypTSZPhFYzpY48tnaXN").into_vec()
                        .unwrap().try_into().unwrap()),
           "$type": "indexedDocument",
           "$dataContractId": Identifier::new(bs58::decode("BZUodcFoFL6KvnonehrnMVggTvCe8W5MiRnZuqLb6M54").into_vec()
                        .unwrap().try_into().unwrap()),
           "$ownerId": Identifier::new(bs58::decode("GZVdTnLFAN2yE9rLeCHBDBCr7YQgmXJuoExkY347j7Z5").into_vec()
                        .unwrap().try_into().unwrap()),
           "$revision": 1,
           "firstName": "name",
           "lastName": "lastName",
           "$createdAt": 1647535750329_u64,
           "$updatedAt": 1647535750329_u64,
        });

        let mut document = Document::from_platform_value(document_values, platform_version)
            .expect("expected to make document");

        let document_type = contract
            .document_type_for_name("indexedDocument")
            .expect("expected to get a document type");

        let current_epoch_index = platform.state.load().last_block_info().epoch.index;
        let storage_flags = Some(StorageFlags::new_single_epoch(
            current_epoch_index,
            None,
        ));
        let storage_flags_cow: Option<Cow<'_, StorageFlags>> = storage_flags.clone().map(Cow::Owned);
        println!("add_document with storage_flags:{:?}\n", storage_flags.clone());

        platform.drive
            .add_document_for_contract(
                DocumentAndContractInfo {
                    owned_document_info: OwnedDocumentInfo {
                        document_info: DocumentOwnedInfo((
                            document.clone(),
                            storage_flags_cow,
                        )),
                        owner_id: None,
                    },
                    contract: &contract,
                    document_type,
                },
                false,
                *platform.state.load().last_block_info(),
                true,
                None,
                platform_version,
                None,
            )
            .expect("should add document");

        println!("document created @ last_block_info: {:?}\n", platform.state.load().last_block_info());

        epoch_index += 1;
        fast_forward_to_block(
            &platform,
            5000 * epoch_index as u64,
            (activation_core_height + 100) as u64,
            (epoch_core_start_height + 100),
            epoch_index,
            true,
        );

        let document_properties = document.properties_mut();
        let new_value : platform_value::Value = dpp::platform_value::Value::Text("nameody".to_string()); // +3 bytes on property firstName
        println!("renaming {} -> {}\n", document_properties.get(&"firstName".to_string()).unwrap().to_string(), new_value.to_string());
        document_properties.insert("firstName".to_string(), new_value);
        println!("document_properties: {:?}\n", document_properties);

        let current_epoch_index = platform.state.load().last_block_info().epoch.index;
        let storage_flags = Some(StorageFlags::new_single_epoch(
            current_epoch_index,
            None,
        ));
        let storage_flags_cow: Option<Cow<'_, StorageFlags>> = storage_flags.clone().map(Cow::Owned);
        println!("update_document with storage_flags:{:?}\n", storage_flags.clone());

        platform.drive
            .update_document_for_contract(
                &document,
                &contract,
                document_type,
                None,
                *platform.state.load().last_block_info(),
                true,
                storage_flags_cow,
                None,
                platform_version,
                Some(&EPOCH_CHANGE_FEE_VERSION_TEST),
            )
            .expect("should update document");

        println!("document updated @ last_block_info: {:?}\n", platform.state.load().last_block_info());
        return;

        epoch_index += 1;
        fast_forward_to_block(
            &platform,
            5000 * epoch_index as u64,
            (activation_core_height + 100) as u64,
            (epoch_core_start_height + 100),
            epoch_index,
            true,
        );

        let document_properties = document.properties_mut();
        let new_value : platform_value::Value = dpp::platform_value::Value::Text("nameodysseas".to_string()); // +5 bytes on property firstName
        println!("renaming {} -> {}\n", document_properties.get(&"firstName".to_string()).unwrap().to_string(), new_value.to_string());
        document_properties.insert("firstName".to_string(), new_value);
        println!("document_properties: {:?}\n", document_properties);

        let current_epoch_index = platform.state.load().last_block_info().epoch.index;
        let storage_flags = Some(StorageFlags::new_single_epoch(
            current_epoch_index,
            None,
        ));
        let storage_flags_cow: Option<Cow<'_, StorageFlags>> = storage_flags.clone().map(Cow::Owned);
        println!("update_document with storage_flags:{:?}\n", storage_flags.clone());

        platform.drive
            .update_document_for_contract(
                &document,
                &contract,
                document_type,
                None,
                *platform.state.load().last_block_info(),
                true,
                storage_flags_cow,
                None,
                platform_version,
                Some(&EPOCH_CHANGE_FEE_VERSION_TEST),
            )
            .expect("should update document");

        println!("document updated @ last_block_info: {:?}\n", platform.state.load().last_block_info());

        println!("final last_block_info: {:?}\n", platform.state.load().last_block_info());

        return;
    }
}
