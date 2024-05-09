use crate::version::contracts::SystemDataContractVersions;
use crate::version::dpp_versions::{
    AssetLockVersions, ContractVersions, CostVersions, DPPValidationVersions, DPPVersion,
    DataContractMethodVersions, DataContractValidationVersions, DocumentFeatureVersionBounds,
    DocumentMethodVersions, DocumentTransitionVersions, DocumentTypeClassMethodVersions,
    DocumentTypeIndexVersions, DocumentTypeMethodVersions, DocumentTypeSchemaVersions,
    DocumentTypeValidationVersions, DocumentTypeVersions, DocumentVersions,
    DocumentsBatchTransitionValidationVersions, DocumentsBatchTransitionVersions,
    IdentityKeyTypeMethodVersions, IdentityTransitionAssetLockVersions, IdentityTransitionVersions,
    IdentityVersions, JsonSchemaValidatorVersions, PublicKeyInCreationMethodVersions,
    RecursiveSchemaValidatorVersions, StateTransitionConversionVersions,
    StateTransitionMethodVersions, StateTransitionSerializationVersions, StateTransitionVersions,
};
use crate::version::drive_abci_versions::{
    DriveAbciAssetLockValidationVersions, DriveAbciBlockEndMethodVersions,
    DriveAbciBlockFeeProcessingMethodVersions, DriveAbciBlockStartMethodVersions,
    DriveAbciCoreBasedUpdatesMethodVersions, DriveAbciCoreChainLockMethodVersionsAndConstants,
    DriveAbciCoreSubsidyMethodVersions, DriveAbciDocumentsStateTransitionValidationVersions,
    DriveAbciEngineMethodVersions, DriveAbciEpochMethodVersions,
    DriveAbciFeePoolInwardsDistributionMethodVersions,
    DriveAbciFeePoolOutwardsDistributionMethodVersions,
    DriveAbciIdentityCreditWithdrawalMethodVersions, DriveAbciInitializationMethodVersions,
    DriveAbciMasternodeIdentitiesUpdatesMethodVersions, DriveAbciMethodVersions,
    DriveAbciPlatformStateStorageMethodVersions, DriveAbciProtocolUpgradeMethodVersions,
    DriveAbciQueryDataContractVersions, DriveAbciQueryIdentityVersions,
    DriveAbciQuerySystemVersions, DriveAbciQueryVersions,
    DriveAbciStateTransitionCommonValidationVersions,
    DriveAbciStateTransitionProcessingMethodVersions, DriveAbciStateTransitionValidationVersion,
    DriveAbciStateTransitionValidationVersions, DriveAbciStructureVersions,
    DriveAbciValidationDataTriggerAndBindingVersions, DriveAbciValidationDataTriggerVersions,
    DriveAbciValidationVersions, DriveAbciVersion, PenaltyAmounts,
};
use crate::version::drive_versions::{DriveAssetLockMethodVersions, DriveBalancesMethodVersions, DriveBatchOperationsMethodVersion, DriveContractApplyMethodVersions, DriveContractCostsMethodVersions, DriveContractGetMethodVersions, DriveContractInsertMethodVersions, DriveContractMethodVersions, DriveContractProveMethodVersions, DriveContractUpdateMethodVersions, DriveCreditPoolEpochsMethodVersions, DriveCreditPoolMethodVersions, DriveCreditPoolPendingEpochRefundsMethodVersions, DriveCreditPoolStorageFeeDistributionPoolMethodVersions, DriveCreditPoolUnpaidEpochMethodVersions, DriveDataContractOperationMethodVersions, DriveDocumentDeleteMethodVersions, DriveDocumentEstimationCostsMethodVersions, DriveDocumentIndexUniquenessMethodVersions, DriveDocumentInsertContestedMethodVersions, DriveDocumentInsertMethodVersions, DriveDocumentMethodVersions, DriveDocumentQueryMethodVersions, DriveDocumentUpdateMethodVersions, DriveEstimatedCostsMethodVersions, DriveFeesMethodVersions, DriveFetchMethodVersions, DriveGroveApplyMethodVersions, DriveGroveBasicMethodVersions, DriveGroveBatchMethodVersions, DriveGroveCostMethodVersions, DriveGroveMethodVersions, DriveIdentityContractInfoMethodVersions, DriveIdentityCostEstimationMethodVersions, DriveIdentityFetchAttributesMethodVersions, DriveIdentityFetchFullIdentityMethodVersions, DriveIdentityFetchMethodVersions, DriveIdentityFetchPartialIdentityMethodVersions, DriveIdentityFetchPublicKeyHashesMethodVersions, DriveIdentityInsertMethodVersions, DriveIdentityKeyHashesToIdentityInsertMethodVersions, DriveIdentityKeysFetchMethodVersions, DriveIdentityKeysInsertMethodVersions, DriveIdentityKeysMethodVersions, DriveIdentityKeysProveMethodVersions, DriveIdentityMethodVersions, DriveIdentityProveMethodVersions, DriveIdentityUpdateMethodVersions, DriveIdentityWithdrawalDocumentMethodVersions, DriveIdentityWithdrawalMethodVersions, DriveIdentityWithdrawalTransactionIndexMethodVersions, DriveIdentityWithdrawalTransactionMethodVersions, DriveIdentityWithdrawalTransactionQueueMethodVersions, DriveInitializationMethodVersions, DriveMethodVersions, DriveOperationsMethodVersion, DrivePlatformStateMethodVersions, DrivePlatformSystemMethodVersions, DrivePrefundedSpecializedMethodVersions, DriveProtocolUpgradeVersions, DriveProveMethodVersions, DriveStateTransitionMethodVersions, DriveStateTransitionOperationMethodVersions, DriveStructureVersion, DriveSystemEstimationCostsMethodVersions, DriveVerifyContractMethodVersions, DriveVerifyDocumentMethodVersions, DriveVerifyIdentityMethodVersions, DriveVerifyMethodVersions, DriveVerifySingleDocumentMethodVersions, DriveVerifyStateTransitionMethodVersions, DriveVerifySystemMethodVersions, DriveVerifyVoteMethodVersions, DriveVersion, DriveVoteCleanupMethodVersions, DriveVoteContestedResourceInsertMethodVersions, DriveVoteInsertMethodVersions, DriveVoteMethodVersions, DriveVoteSetupMethodVersions};
use crate::version::fee::v1::FEE_VERSION1;
use crate::version::protocol_version::{FeatureVersionBounds, PlatformVersion};
use crate::version::{AbciStructureVersion, PlatformArchitectureVersion};

pub const PROTOCOL_VERSION_1: u32 = 1;

pub const PLATFORM_V1: PlatformVersion = PlatformVersion {
    protocol_version: 1,
    identity: FeatureVersionBounds {
        min_version: 0,
        max_version: 0,
        default_current_version: 0,
    },
    proofs: FeatureVersionBounds {
        min_version: 0,
        max_version: 0,
        default_current_version: 0,
    },
    drive: DriveVersion {
        structure: DriveStructureVersion {
            document_indexes: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            identity_indexes: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            pools: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
        },
        methods: DriveMethodVersions {
            initialization: DriveInitializationMethodVersions {
                create_initial_state_structure: 0,
            },
            credit_pools: DriveCreditPoolMethodVersions {
                epochs: DriveCreditPoolEpochsMethodVersions {
                    get_epochs_infos: 0,
                    prove_epochs_infos: 0,
                    get_epoch_fee_multiplier: 0,
                    get_epoch_processing_credits_for_distribution: 0,
                    get_epoch_storage_credits_for_distribution: 0,
                    get_epoch_total_credits_for_distribution: 0,
                    get_storage_credits_for_distribution_for_epochs_in_range: 0,
                    get_epoch_start_time: 0,
                    get_epoch_start_block_core_height: 0,
                    get_epoch_start_block_height: 0,
                    get_first_epoch_start_block_info_between_epochs: 0,
                    get_epoch_proposers: 0,
                    get_epochs_proposer_block_count: 0,
                    add_update_pending_epoch_refunds_operations: 0,
                    is_epochs_proposers_tree_empty: 0,
                },
                pending_epoch_refunds: DriveCreditPoolPendingEpochRefundsMethodVersions {
                    add_delete_pending_epoch_refunds_except_specified: 0,
                    fetch_and_add_pending_epoch_refunds_to_collection: 0,
                    fetch_pending_epoch_refunds: 0,
                    add_update_pending_epoch_refunds_operations: 0,
                },
                storage_fee_distribution_pool:
                    DriveCreditPoolStorageFeeDistributionPoolMethodVersions {
                        get_storage_fees_from_distribution_pool: 0,
                    },
                unpaid_epoch: DriveCreditPoolUnpaidEpochMethodVersions {
                    get_unpaid_epoch_index: 0,
                },
            },
            protocol_upgrade: DriveProtocolUpgradeVersions {
                clear_version_information: 0,
                fetch_versions_with_counter: 0,
                fetch_proved_versions_with_counter: 0,
                fetch_validator_version_votes: 0,
                fetch_proved_validator_version_votes: 0,
                remove_validators_proposed_app_versions: 0,
                update_validator_proposed_app_version: 0,
            },
            prove: DriveProveMethodVersions {
                prove_elements: 0,
                prove_multiple_state_transition_results: 0,
            },
            balances: DriveBalancesMethodVersions {
                add_to_system_credits: 0,
                add_to_system_credits_operations: 0,
                remove_from_system_credits: 0,
                remove_from_system_credits_operations: 0,
                calculate_total_credits_balance: 0,
            },
            document: DriveDocumentMethodVersions {
                query: DriveDocumentQueryMethodVersions { query_documents: 0 },
                delete: DriveDocumentDeleteMethodVersions {
                    add_estimation_costs_for_remove_document_to_primary_storage: 0,
                    delete_document_for_contract: 0,
                    delete_document_for_contract_id: 0,
                    delete_document_for_contract_apply_and_add_to_operations: 0,
                    remove_document_from_primary_storage: 0,
                    remove_reference_for_index_level_for_contract_operations: 0,
                    remove_indices_for_index_level_for_contract_operations: 0,
                    remove_indices_for_top_index_level_for_contract_operations: 0,
                    delete_document_for_contract_id_with_named_type_operations: 0,
                    delete_document_for_contract_with_named_type_operations: 0,
                    delete_document_for_contract_operations: 0,
                },
                insert: DriveDocumentInsertMethodVersions {
                    add_document: 0,
                    add_document_for_contract: 0,
                    add_document_for_contract_apply_and_add_to_operations: 0,
                    add_document_for_contract_operations: 0,
                    add_document_to_primary_storage: 0,
                    add_indices_for_index_level_for_contract_operations: 0,
                    add_indices_for_top_index_level_for_contract_operations: 0,
                    add_reference_for_index_level_for_contract_operations: 0,
                },
                insert_contested: DriveDocumentInsertContestedMethodVersions {
                    add_contested_document: 0,
                    add_contested_document_for_contract: 0,
                    add_contested_document_for_contract_apply_and_add_to_operations: 0,
                    add_contested_document_for_contract_operations: 0,
                    add_contested_document_to_primary_storage: 0,
                    add_contested_indices_for_index_level_for_contract_operations: 0,
                    add_contested_indices_for_top_index_level_for_contract_operations: 0,
                    add_contested_reference_for_index_level_for_contract_operations: 0,
                },
                update: DriveDocumentUpdateMethodVersions {
                    add_update_multiple_documents_operations: 0,
                    update_document_for_contract: 0,
                    update_document_for_contract_apply_and_add_to_operations: 0,
                    update_document_for_contract_id: 0,
                    update_document_for_contract_operations: 0,
                    update_document_with_serialization_for_contract: 0,
                    update_serialized_document_for_contract: 0,
                },
                estimation_costs: DriveDocumentEstimationCostsMethodVersions {
                    add_estimation_costs_for_add_document_to_primary_storage: 0,
                    stateless_delete_of_non_tree_for_costs: 0,
                },
                index_uniqueness: DriveDocumentIndexUniquenessMethodVersions {
                    validate_document_uniqueness: 0,
                    validate_document_create_transition_action_uniqueness: 0,
                    validate_document_replace_transition_action_uniqueness: 0,
                    validate_document_transfer_transition_action_uniqueness: 0,
                    validate_document_purchase_transition_action_uniqueness: 0,
                    validate_document_update_price_transition_action_uniqueness: 0,
                    validate_uniqueness_of_data: 0,
                },
            },
            vote: DriveVoteMethodVersions {
                insert: DriveVoteInsertMethodVersions {
                    register_identity_vote: 0,
                },
                contested_resource_insert: DriveVoteContestedResourceInsertMethodVersions {
                    register_contested_resource_identity_vote: 0,
                    register_identity_vote: 0,
                    add_vote_poll_end_date_query: 0,
                },
                cleanup: DriveVoteCleanupMethodVersions {
                    remove_votes_for_identity: 0,
                },
                setup: DriveVoteSetupMethodVersions {
                    add_initial_vote_tree_main_structure_operations: 0,
                },
            },
            contract: DriveContractMethodVersions {
                prove: DriveContractProveMethodVersions {
                    prove_contract: 0,
                    prove_contract_history: 0,
                    prove_contracts: 0,
                },
                apply: DriveContractApplyMethodVersions {
                    apply_contract: 0,
                    apply_contract_with_serialization: 0,
                },
                insert: DriveContractInsertMethodVersions {
                    add_contract_to_storage: 0,
                    insert_contract: 0,
                },
                update: DriveContractUpdateMethodVersions { update_contract: 0 },
                costs: DriveContractCostsMethodVersions {
                    add_estimation_costs_for_contract_insertion: 0,
                },
                get: DriveContractGetMethodVersions {
                    fetch_contract: 0,
                    fetch_contract_with_history: 0,
                    get_cached_contract_with_fetch_info: 0,
                    get_contract_with_fetch_info: 0,
                    get_contracts_with_fetch_info: 0,
                },
            },
            fees: DriveFeesMethodVersions { calculate_fee: 0 },
            estimated_costs: DriveEstimatedCostsMethodVersions {
                add_estimation_costs_for_levels_up_to_contract: 0,
                add_estimation_costs_for_levels_up_to_contract_document_type_excluded: 0,
            },
            asset_lock: DriveAssetLockMethodVersions {
                add_asset_lock_outpoint: 0,
                add_estimation_costs_for_adding_asset_lock: 0,
                fetch_asset_lock_outpoint_info: 0,
            },
            verify: DriveVerifyMethodVersions {
                contract: DriveVerifyContractMethodVersions {
                    verify_contract: 0,
                    verify_contract_history: 0,
                },
                document: DriveVerifyDocumentMethodVersions {
                    verify_proof: 0,
                    verify_proof_keep_serialized: 0,
                    verify_start_at_document_in_proof: 0,
                },
                identity: DriveVerifyIdentityMethodVersions {
                    verify_full_identities_by_public_key_hashes: 0,
                    verify_full_identity_by_identity_id: 0,
                    verify_full_identity_by_public_key_hash: 0,
                    verify_identity_balance_for_identity_id: 0,
                    verify_identity_balances_for_identity_ids: 0,
                    verify_identity_id_by_public_key_hash: 0,
                    verify_identity_ids_by_public_key_hashes: 0,
                    verify_identity_keys_by_identity_id: 0,
                    verify_identity_nonce: 0,
                    verify_identity_contract_nonce: 0,
                    verify_identities_contract_keys: 0,
                },
                single_document: DriveVerifySingleDocumentMethodVersions {
                    verify_proof: 0,
                    verify_proof_keep_serialized: 0,
                },
                system: DriveVerifySystemMethodVersions {
                    verify_epoch_infos: 0,
                    verify_elements: 0,
                    verify_upgrade_state: 0,
                    verify_upgrade_vote_status: 0,
                },
                voting: DriveVerifyVoteMethodVersions {
                    verify_masternode_vote: 0,
                },
                state_transition: DriveVerifyStateTransitionMethodVersions {
                    verify_state_transition_was_executed_with_proof: 0,
                },
            },
            identity: DriveIdentityMethodVersions {
                fetch: DriveIdentityFetchMethodVersions {
                    public_key_hashes: DriveIdentityFetchPublicKeyHashesMethodVersions {
                        fetch_full_identities_by_unique_public_key_hashes: 0,
                        fetch_full_identity_by_unique_public_key_hash: 0,
                        fetch_identity_id_by_unique_public_key_hash: 0,
                        fetch_identity_ids_by_non_unique_public_key_hash: 0,
                        fetch_identity_ids_by_unique_public_key_hashes: 0,
                        fetch_serialized_full_identity_by_unique_public_key_hash: 0,
                        has_any_of_unique_public_key_hashes: 0,
                        has_non_unique_public_key_hash: 0,
                        has_non_unique_public_key_hash_already_for_identity: 0,
                        has_unique_public_key_hash: 0,
                    },
                    attributes: DriveIdentityFetchAttributesMethodVersions {
                        revision: 0,
                        nonce: 0,
                        identity_contract_nonce: 0,
                        balance: 0,
                        balance_include_debt: 0,
                        negative_balance: 0,
                    },
                    partial_identity: DriveIdentityFetchPartialIdentityMethodVersions {
                        fetch_identity_revision_with_keys: 0,
                        fetch_identity_balance_with_keys: 0,
                        fetch_identity_balance_with_keys_and_revision: 0,
                        fetch_identity_with_balance: 0,
                        fetch_identity_keys: 0,
                    },
                    full_identity: DriveIdentityFetchFullIdentityMethodVersions {
                        fetch_full_identity: Some(0),
                        fetch_full_identities: Some(0),
                    },
                },
                prove: DriveIdentityProveMethodVersions {
                    full_identity: 0,
                    full_identities: 0,
                    identity_nonce: 0,
                    identity_contract_nonce: 0,
                    identities_contract_keys: 0,
                    prove_full_identities_by_unique_public_key_hashes: 0,
                    prove_full_identity_by_unique_public_key_hash: 0,
                    prove_identity_id_by_unique_public_key_hash: 0,
                    prove_identity_ids_by_unique_public_key_hashes: 0,
                },
                keys: DriveIdentityKeysMethodVersions {
                    fetch: DriveIdentityKeysFetchMethodVersions {
                        fetch_all_current_identity_keys: 0,
                        fetch_all_identity_keys: 0,
                        fetch_identities_all_keys: 0,
                        fetch_identity_keys: 0,
                        fetch_identities_contract_keys: 0,
                    },
                    prove: DriveIdentityKeysProveMethodVersions {
                        prove_identities_all_keys: 0,
                        prove_identity_keys: 0,
                    },
                    insert: DriveIdentityKeysInsertMethodVersions {
                        create_key_tree_with_keys: 0,
                        create_new_identity_key_query_trees: 0,
                        insert_key_searchable_references: 0,
                        insert_key_to_storage: 0,
                        insert_new_non_unique_key: 0,
                        insert_new_unique_key: 0,
                        replace_key_in_storage: 0,
                    },
                    insert_key_hash_identity_reference:
                        DriveIdentityKeyHashesToIdentityInsertMethodVersions {
                            add_estimation_costs_for_insert_non_unique_public_key_hash_reference: 0,
                            add_estimation_costs_for_insert_unique_public_key_hash_reference: 0,
                            insert_non_unique_public_key_hash_reference_to_identity: 0,
                            insert_reference_to_non_unique_key: 0,
                            insert_reference_to_unique_key: 0,
                            insert_unique_public_key_hash_reference_to_identity: 0,
                        },
                },
                update: DriveIdentityUpdateMethodVersions {
                    update_identity_revision: 0,
                    merge_identity_nonce: 0,
                    update_identity_negative_credit_operation: 0,
                    initialize_identity_revision: 0,
                    disable_identity_keys: 0,
                    re_enable_identity_keys: 0,
                    add_new_non_unique_keys_to_identity: 0,
                    add_new_unique_keys_to_identity: 0,
                    add_new_keys_to_identity: 0,
                    insert_identity_balance: 0,
                    initialize_negative_identity_balance: 0,
                    add_to_identity_balance: 0,
                    add_to_previous_balance: 0,
                    apply_balance_change_from_fee_to_identity: 0,
                    remove_from_identity_balance: 0,
                },
                insert: DriveIdentityInsertMethodVersions {
                    add_new_identity: 0,
                },
                contract_info: DriveIdentityContractInfoMethodVersions {
                    add_potential_contract_info_for_contract_bounded_key: 0,
                    merge_identity_contract_nonce: 0,
                },
                cost_estimation: DriveIdentityCostEstimationMethodVersions {
                    for_authentication_keys_security_level_in_key_reference_tree: 0,
                    for_balances: 0,
                    for_contract_info: 0,
                    for_contract_info_group: 0,
                    for_contract_info_group_keys: 0,
                    for_contract_info_group_key_purpose: 0,
                    for_keys_for_identity_id: 0,
                    for_negative_credit: 0,
                    for_purpose_in_key_reference_tree: 0,
                    for_root_key_reference_tree: 0,
                    for_update_revision: 0,
                },
                withdrawals: DriveIdentityWithdrawalMethodVersions {
                    document: DriveIdentityWithdrawalDocumentMethodVersions {
                        fetch_oldest_withdrawal_documents_by_status: 0,
                        find_up_to_100_withdrawal_documents_by_status_and_transaction_indices: 0,
                    },
                    transaction: DriveIdentityWithdrawalTransactionMethodVersions {
                        index: DriveIdentityWithdrawalTransactionIndexMethodVersions {
                            fetch_next_withdrawal_transaction_index: 0,
                            add_update_next_withdrawal_transaction_index_operation: 0,
                        },
                        queue: DriveIdentityWithdrawalTransactionQueueMethodVersions {
                            add_enqueue_untied_withdrawal_transaction_operations: 0,
                            dequeue_untied_withdrawal_transactions: 0,
                        },
                    },
                },
            },
            platform_system: DrivePlatformSystemMethodVersions {
                estimation_costs: DriveSystemEstimationCostsMethodVersions {
                    for_total_system_credits_update: 0,
                },
            },
            operations: DriveOperationsMethodVersion {
                rollback_transaction: 0,
                drop_cache: 0,
                commit_transaction: 0,
                apply_partial_batch_low_level_drive_operations: 0,
                apply_partial_batch_grovedb_operations: 0,
                apply_batch_low_level_drive_operations: 0,
                apply_batch_grovedb_operations: 0,
            },
            state_transitions: DriveStateTransitionMethodVersions {
                operations: DriveStateTransitionOperationMethodVersions {
                    finalization_tasks: 0,
                    contracts: DriveDataContractOperationMethodVersions {
                        finalization_tasks: 0,
                    },
                },
            },
            batch_operations: DriveBatchOperationsMethodVersion {
                convert_drive_operations_to_grove_operations: 0,
                apply_drive_operations: 0,
            },
            platform_state: DrivePlatformStateMethodVersions {
                fetch_platform_state_bytes: 0,
                store_platform_state_bytes: 0,
            },
            fetch: DriveFetchMethodVersions { fetch_elements: 0 },
            prefunded_specialized_balances: DrivePrefundedSpecializedMethodVersions {
                add_prefunded_specialized_balance: 0,
                add_prefunded_specialized_balance_operations: 0,
                deduct_from_prefunded_specialized_balance: 0,
                deduct_from_prefunded_specialized_balance_operations: 0,
                estimated_cost_for_prefunded_specialized_balance_update: 0,
            },
        },
        grove_methods: DriveGroveMethodVersions {
            basic: DriveGroveBasicMethodVersions {
                grove_insert: 0,
                grove_insert_empty_tree: 0,
                grove_insert_empty_sum_tree: 0,
                grove_insert_if_not_exists: 0,
                grove_clear: 0,
                grove_delete: 0,
                grove_get_raw: 0,
                grove_get_raw_optional: 0,
                grove_get_raw_value_u64_from_encoded_var_vec: 0,
                grove_get: 0,
                grove_get_path_query_serialized_results: 0,
                grove_get_path_query: 0,
                grove_get_path_query_with_optional: 0,
                grove_get_raw_path_query_with_optional: 0,
                grove_get_raw_path_query: 0,
                grove_get_proved_path_query: 0,
                grove_get_proved_path_query_with_conditional: 0,
                grove_get_sum_tree_total_value: 0,
                grove_has_raw: 0,
            },
            batch: DriveGroveBatchMethodVersions {
                batch_insert_empty_tree: 0,
                batch_insert_empty_tree_if_not_exists: 0,
                batch_insert_empty_tree_if_not_exists_check_existing_operations: 0,
                batch_insert: 0,
                batch_insert_if_not_exists: 0,
                batch_insert_if_changed_value: 0,
                batch_delete: 0,
                batch_remove_raw: 0,
                batch_delete_up_tree_while_empty: 0,
                batch_refresh_reference: 0,
            },
            apply: DriveGroveApplyMethodVersions {
                grove_apply_operation: 0,
                grove_apply_batch: 0,
                grove_apply_partial_batch: 0,
            },
            costs: DriveGroveCostMethodVersions {
                grove_batch_operations_costs: 0,
            },
        },
    },
    abci_structure: AbciStructureVersion {
        extended_block_info: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
    },
    platform_architecture: PlatformArchitectureVersion {
        data_contract_factory_structure_version: 0,
        document_factory_structure_version: 0,
    },
    drive_abci: DriveAbciVersion {
        structs: DriveAbciStructureVersions {
            platform_state_structure: 0,
            platform_state_for_saving_structure: 0,
            state_transition_execution_context: 0,
            commit: 0,
            masternode: 0,
        },
        methods: DriveAbciMethodVersions {
            engine: DriveAbciEngineMethodVersions {
                init_chain: 0,
                check_tx: 0,
                run_block_proposal: 0,
                finalize_block_proposal: 0,
            },
            initialization: DriveAbciInitializationMethodVersions {
                initial_core_height: 0,
                create_genesis_state: 0,
            },
            core_based_updates: DriveAbciCoreBasedUpdatesMethodVersions {
                update_core_info: 0,
                update_masternode_list: 0,
                update_quorum_info: 0,
                masternode_updates: DriveAbciMasternodeIdentitiesUpdatesMethodVersions {
                    get_voter_identity_key: 0,
                    get_operator_identity_keys: 0,
                    get_owner_identity_key: 0,
                    get_voter_identifier: 0,
                    get_operator_identifier: 0,
                    create_operator_identity: 0,
                    create_owner_identity: 0,
                    create_voter_identity: 0,
                    hash_protxhash_with_key_data: 0,
                    disable_identity_keys: 0,
                    update_masternode_identities: 0,
                    update_operator_identity: 0,
                    update_owner_withdrawal_address: 0,
                    update_voter_identity: 0,
                },
            },
            protocol_upgrade: DriveAbciProtocolUpgradeMethodVersions {
                check_for_desired_protocol_upgrade: 0,
                upgrade_protocol_version_on_epoch_change: 0,
                protocol_version_upgrade_percentage_needed: 75,
            },
            block_fee_processing: DriveAbciBlockFeeProcessingMethodVersions {
                add_process_epoch_change_operations: 0,
                process_block_fees: 0,
            },
            core_subsidy: DriveAbciCoreSubsidyMethodVersions {
                epoch_core_reward_credits_for_distribution: 0,
            },
            core_chain_lock: DriveAbciCoreChainLockMethodVersionsAndConstants {
                choose_quorum: 0,
                verify_chain_lock: 0,
                verify_chain_lock_locally: 0,
                verify_chain_lock_through_core: 0,
                make_sure_core_is_synced_to_chain_lock: 0,
                recent_block_count_amount: 2,
            },
            fee_pool_inwards_distribution: DriveAbciFeePoolInwardsDistributionMethodVersions {
                add_distribute_block_fees_into_pools_operations: 0,
                add_distribute_storage_fee_to_epochs_operations: 0,
            },
            fee_pool_outwards_distribution: DriveAbciFeePoolOutwardsDistributionMethodVersions {
                add_distribute_fees_from_oldest_unpaid_epoch_pool_to_proposers_operations: 0,
                add_epoch_pool_to_proposers_payout_operations: 0,
                find_oldest_epoch_needing_payment: 0,
                fetch_reward_shares_list_for_masternode: 0,
            },
            withdrawals: DriveAbciIdentityCreditWithdrawalMethodVersions {
                build_untied_withdrawal_transactions_from_documents: 0,
                dequeue_and_build_unsigned_withdrawal_transactions: 0,
                fetch_transactions_block_inclusion_status: 0,
                pool_withdrawals_into_transactions_queue: 0,
                update_broadcasted_withdrawal_statuses: 0,
                append_signatures_and_broadcast_withdrawal_transactions: 0,
            },
            state_transition_processing: DriveAbciStateTransitionProcessingMethodVersions {
                execute_event: 0,
                process_raw_state_transitions: 0,
                decode_raw_state_transitions: 0,
                validate_fees_of_event: 0,
            },
            epoch: DriveAbciEpochMethodVersions {
                gather_epoch_info: 0,
                get_genesis_time: 0,
            },
            block_start: DriveAbciBlockStartMethodVersions {
                clear_drive_block_cache: 0,
            },
            block_end: DriveAbciBlockEndMethodVersions {
                update_state_cache: 0,
                update_drive_cache: 0,
                validator_set_update: 0,
            },
            platform_state_storage: DriveAbciPlatformStateStorageMethodVersions {
                fetch_platform_state: 0,
                store_platform_state: 0,
            },
        },
        validation_and_processing: DriveAbciValidationVersions {
            state_transitions: DriveAbciStateTransitionValidationVersions {
                common_validation_methods: DriveAbciStateTransitionCommonValidationVersions {
                    asset_locks: DriveAbciAssetLockValidationVersions {
                        fetch_asset_lock_transaction_output_sync: 0,
                        verify_asset_lock_is_not_spent_and_has_enough_balance: 0,
                    },
                    validate_identity_public_key_contract_bounds: 0,
                    validate_identity_public_key_ids_dont_exist_in_state: 0,
                    validate_identity_public_key_ids_exist_in_state: 0,
                    validate_state_transition_identity_signed: 0,
                    validate_unique_identity_public_key_hashes_in_state: 0,
                    validate_master_key_uniqueness: 0,
                    validate_simple_pre_check_balance: 0,
                },
                max_asset_lock_usage_attempts: 16,
                identity_create_state_transition: DriveAbciStateTransitionValidationVersion {
                    basic_structure: Some(0),
                    advanced_structure: Some(0),
                    identity_signatures: Some(0),
                    advanced_minimum_balance_pre_check: None,
                    nonce: None,
                    state: 0,
                    transform_into_action: 0,
                },
                identity_update_state_transition: DriveAbciStateTransitionValidationVersion {
                    basic_structure: Some(0),
                    advanced_structure: Some(0),
                    identity_signatures: Some(0),
                    advanced_minimum_balance_pre_check: None,
                    nonce: Some(0),
                    state: 0,
                    transform_into_action: 0,
                },
                identity_top_up_state_transition: DriveAbciStateTransitionValidationVersion {
                    basic_structure: Some(0),
                    advanced_structure: None,
                    identity_signatures: None,
                    advanced_minimum_balance_pre_check: None,
                    nonce: None,
                    state: 0,
                    transform_into_action: 0,
                },
                identity_credit_withdrawal_state_transition:
                    DriveAbciStateTransitionValidationVersion {
                        basic_structure: Some(0),
                        advanced_structure: None,
                        identity_signatures: None,
                        advanced_minimum_balance_pre_check: Some(0),
                        nonce: Some(0),
                        state: 0,
                        transform_into_action: 0,
                    },
                identity_credit_transfer_state_transition:
                    DriveAbciStateTransitionValidationVersion {
                        basic_structure: Some(0),
                        advanced_structure: None,
                        identity_signatures: None,
                        advanced_minimum_balance_pre_check: Some(0),
                        nonce: Some(0),
                        state: 0,
                        transform_into_action: 0,
                    },
                masternode_vote_state_transition: DriveAbciStateTransitionValidationVersion {
                    basic_structure: Some(0),
                    advanced_structure: None,
                    identity_signatures: None,
                    advanced_minimum_balance_pre_check: None,
                    nonce: Some(0),
                    state: 0,
                    transform_into_action: 0,
                },
                contract_create_state_transition: DriveAbciStateTransitionValidationVersion {
                    basic_structure: Some(0),
                    advanced_structure: None,
                    identity_signatures: None,
                    advanced_minimum_balance_pre_check: None,
                    nonce: Some(0),
                    state: 0,
                    transform_into_action: 0,
                },
                contract_update_state_transition: DriveAbciStateTransitionValidationVersion {
                    basic_structure: None,
                    advanced_structure: None,
                    identity_signatures: None,
                    advanced_minimum_balance_pre_check: None,
                    nonce: Some(0),
                    state: 0,
                    transform_into_action: 0,
                },
                documents_batch_state_transition:
                    DriveAbciDocumentsStateTransitionValidationVersions {
                        balance_pre_check: 0,
                        basic_structure: 0,
                        advanced_structure: 0,
                        state: 0,
                        revision: 0,
                        transform_into_action: 0,
                        data_triggers: DriveAbciValidationDataTriggerAndBindingVersions {
                            bindings: 0,
                            triggers: DriveAbciValidationDataTriggerVersions {
                                create_contact_request_data_trigger: 0,
                                create_domain_data_trigger: 0,
                                create_identity_data_trigger: 0,
                                create_feature_flag_data_trigger: 0,
                                create_masternode_reward_shares_data_trigger: 0,
                                delete_withdrawal_data_trigger: 0,
                                reject_data_trigger: 0,
                            },
                        },
                        document_create_transition_structure_validation: 0,
                        document_delete_transition_structure_validation: 0,
                        document_replace_transition_structure_validation: 0,
                        document_transfer_transition_structure_validation: 0,
                        document_purchase_transition_structure_validation: 0,
                        document_update_price_transition_structure_validation: 0,
                        document_create_transition_state_validation: 0,
                        document_delete_transition_state_validation: 0,
                        document_replace_transition_state_validation: 0,
                        document_transfer_transition_state_validation: 0,
                        document_purchase_transition_state_validation: 0,
                        document_update_price_transition_state_validation: 0,
                    },
            },
            process_state_transition: 0,
            state_transition_to_execution_event_for_check_tx: 0,
            penalties: PenaltyAmounts {
                identity_id_not_correct: 50000000,
                unique_key_already_present: 10000000,
                validation_of_added_keys_structure_failure: 10000000,
                validation_of_added_keys_proof_of_possession_failure: 50000000,
            },
        },
        query: DriveAbciQueryVersions {
            max_returned_elements: 100,
            response_metadata: 0,
            proofs_query: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            document_query: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            identity_based_queries: DriveAbciQueryIdentityVersions {
                identity: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                keys: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                identities_contract_keys: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                identity_nonce: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                identity_contract_nonce: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                balance: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                balance_and_revision: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                identity_by_public_key_hash: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
            data_contract_based_queries: DriveAbciQueryDataContractVersions {
                data_contract: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                data_contract_history: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                data_contracts: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
            system: DriveAbciQuerySystemVersions {
                version_upgrade_state: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                version_upgrade_vote_status: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                epoch_infos: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
                path_elements: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
        },
    },
    dpp: DPPVersion {
        costs: CostVersions {
            signature_verify: 0,
        },
        validation: DPPValidationVersions {
            json_schema_validator: JsonSchemaValidatorVersions {
                new: 0,
                validate: 0,
                compile: 0,
                compile_and_validate: 0,
            },
            data_contract: DataContractValidationVersions {
                validate: 0,
                validate_config_update: 0,
                validate_index_definitions: 0,
                validate_index_naming_duplicates: 0,
                validate_not_defined_properties: 0,
                validate_property_definition: 0,
            },
            document_type: DocumentTypeValidationVersions { validate_update: 0 },
        },
        state_transition_serialization_versions: StateTransitionSerializationVersions {
            identity_public_key_in_creation: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            identity_create_state_transition: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            identity_update_state_transition: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            identity_top_up_state_transition: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            identity_credit_withdrawal_state_transition: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            identity_credit_transfer_state_transition: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            masternode_vote_state_transition: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            contract_create_state_transition: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            contract_update_state_transition: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            documents_batch_state_transition: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            document_base_state_transition: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            document_create_state_transition: DocumentFeatureVersionBounds {
                bounds: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
            document_replace_state_transition: DocumentFeatureVersionBounds {
                bounds: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
            document_delete_state_transition: DocumentFeatureVersionBounds {
                bounds: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
            document_transfer_state_transition: DocumentFeatureVersionBounds {
                bounds: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
            document_update_price_state_transition: DocumentFeatureVersionBounds {
                bounds: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
            document_purchase_state_transition: DocumentFeatureVersionBounds {
                bounds: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
        },
        state_transition_conversion_versions: StateTransitionConversionVersions {
            identity_to_identity_create_transition: 0,
            identity_to_identity_top_up_transition: 0,
            identity_to_identity_withdrawal_transition: 0,
            identity_to_identity_create_transition_with_signer: 0,
        },
        state_transition_method_versions: StateTransitionMethodVersions {
            public_key_in_creation_methods: PublicKeyInCreationMethodVersions {
                from_public_key_signed_with_private_key: 0,
                from_public_key_signed_external: 0,
                hash: 0,
                duplicated_key_ids_witness: 0,
                duplicated_keys_witness: 0,
                validate_identity_public_keys_structure: 0,
            },
        },
        state_transitions: StateTransitionVersions {
            max_state_transition_size: 20000,
            max_transitions_in_documents_batch: 1,
            documents: DocumentTransitionVersions {
                documents_batch_transition: DocumentsBatchTransitionVersions {
                    validation: DocumentsBatchTransitionValidationVersions {
                        find_duplicates_by_id: 0,
                        validate_base_structure: 0,
                    },
                },
            },
            identities: IdentityTransitionVersions {
                max_public_keys_in_creation: 6,
                asset_locks: IdentityTransitionAssetLockVersions {
                    required_asset_lock_duff_balance_for_processing_start_for_identity_create:
                        200000,
                    required_asset_lock_duff_balance_for_processing_start_for_identity_top_up:
                        50000,
                    validate_asset_lock_transaction_structure: 0,
                    validate_instant_asset_lock_proof_structure: 0,
                },
            },
        },
        contract_versions: ContractVersions {
            max_serialized_size: 65000,
            contract_serialization_version: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            contract_structure_version: 0,
            created_data_contract_structure: 0,
            config: 0,
            methods: DataContractMethodVersions {
                validate_document: 0,
                schema: 0,
            },
            document_type_versions: DocumentTypeVersions {
                index_versions: DocumentTypeIndexVersions {
                    index_levels_from_indices: 0,
                },
                class_method_versions: DocumentTypeClassMethodVersions {
                    try_from_schema: 0,
                    create_document_types_from_document_schemas: 0,
                },
                structure_version: 0,
                schema: DocumentTypeSchemaVersions {
                    enrich_with_base_schema: 0,
                    find_identifier_and_binary_paths: 0,
                    validate_max_depth: 0,
                    max_depth: 256,
                    recursive_schema_validator_versions: RecursiveSchemaValidatorVersions {
                        traversal_validator: 0,
                        byte_array_has_no_items_as_parent_validator: 0,
                        pattern_is_valid_regex_validator: 0,
                    },
                    validate_schema_compatibility: 0,
                },
                methods: DocumentTypeMethodVersions {
                    create_document_from_data: 0,
                    create_document_with_prevalidated_properties: 0,
                    estimated_size: 0,
                    index_for_types: 0,
                    max_size: 0,
                    serialize_value_for_key: 0,
                },
            },
        },
        document_versions: DocumentVersions {
            document_structure_version: 0,
            document_serialization_version: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            document_cbor_serialization_version: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            extended_document_structure_version: 0,
            extended_document_serialization_version: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            document_method_versions: DocumentMethodVersions {
                is_equal_ignoring_timestamps: 0,
                hash: 0,
                get_raw_for_contract: 0,
                get_raw_for_document_type: 0,
            },
        },
        identity_versions: IdentityVersions {
            identity_structure_version: 0,
            identity_key_structure_version: 0,
            identity_key_type_method_versions: IdentityKeyTypeMethodVersions {
                random_public_key_data: 0,
                random_public_and_private_key_data: 0,
            },
        },
        asset_lock_versions: AssetLockVersions {
            reduced_asset_lock_value: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
        },
    },
    system_data_contracts: SystemDataContractVersions {
        withdrawals: 1,
        dpns: 1,
        dashpay: 1,
        masternode_reward_shares: 1,
        feature_flags: 1,
    },
    fee_version: FEE_VERSION1,
};
