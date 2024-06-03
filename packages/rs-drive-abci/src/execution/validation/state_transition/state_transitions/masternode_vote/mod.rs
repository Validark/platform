mod balance;
mod nonce;
mod state;
mod structure;

use dpp::block::block_info::BlockInfo;
use dpp::block::epoch::Epoch;
use dpp::state_transition::masternode_vote_transition::MasternodeVoteTransition;
use dpp::validation::ConsensusValidationResult;
use dpp::version::PlatformVersion;
use drive::state_transition_action::StateTransitionAction;

use drive::grovedb::TransactionArg;

use crate::error::execution::ExecutionError;
use crate::error::Error;
use crate::execution::types::state_transition_execution_context::StateTransitionExecutionContext;
use crate::platform_types::platform::PlatformRef;
use crate::rpc::core::CoreRPCLike;

use crate::execution::validation::state_transition::masternode_vote::state::v0::MasternodeVoteStateTransitionStateValidationV0;
use crate::execution::validation::state_transition::processor::v0::StateTransitionStateValidationV0;
use crate::execution::validation::state_transition::transformer::StateTransitionActionTransformerV0;
use crate::execution::validation::state_transition::ValidationMode;
use crate::platform_types::platform_state::v0::PlatformStateV0Methods;

impl StateTransitionActionTransformerV0 for MasternodeVoteTransition {
    fn transform_into_action<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        _block_info: &BlockInfo,
        _validation_mode: ValidationMode,
        _execution_context: &mut StateTransitionExecutionContext,
        tx: TransactionArg,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error> {
        let platform_version =
            PlatformVersion::get(platform.state.current_protocol_version_in_consensus())?;
        match platform_version
            .drive_abci
            .validation_and_processing
            .state_transitions
            .masternode_vote_state_transition
            .transform_into_action
        {
            0 => self.transform_into_action_v0(platform, tx, platform_version),
            version => Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                method: "masternode votes state transition: transform_into_action".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}

impl StateTransitionStateValidationV0 for MasternodeVoteTransition {
    fn validate_state<C: CoreRPCLike>(
        &self,
        _action: Option<StateTransitionAction>,
        platform: &PlatformRef<C>,
        _validation_mode: ValidationMode,
        _epoch: &Epoch,
        _execution_context: &mut StateTransitionExecutionContext,
        tx: TransactionArg,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error> {
        let platform_version =
            PlatformVersion::get(platform.state.current_protocol_version_in_consensus())?;
        match platform_version
            .drive_abci
            .validation_and_processing
            .state_transitions
            .masternode_vote_state_transition
            .state
        {
            0 => self.validate_state_v0(platform, tx, platform_version),
            version => Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                method: "masternode votes state transition: validate_state".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::execution::validation::state_transition::state_transitions::tests::setup_identity;
    use crate::test::helpers::setup::TestPlatformBuilder;
    use dpp::block::block_info::BlockInfo;
    use dpp::dash_to_credits;
    use dpp::data_contract::accessors::v0::DataContractV0Getters;
    use dpp::data_contract::document_type::accessors::DocumentTypeV0Getters;
    use dpp::data_contract::document_type::random_document::{
        CreateRandomDocument, DocumentFieldFillSize, DocumentFieldFillType,
    };
    use dpp::document::{Document, DocumentV0Getters, DocumentV0Setters};
    use dpp::identity::accessors::IdentityGettersV0;
    use dpp::platform_value::{Bytes32, Value};
    use dpp::serialization::PlatformSerializable;
    use dpp::state_transition::documents_batch_transition::methods::v0::DocumentsBatchTransitionMethodsV0;
    use dpp::state_transition::documents_batch_transition::DocumentsBatchTransition;
    use platform_version::version::PlatformVersion;
    use rand::prelude::StdRng;
    use rand::SeedableRng;
    use dapi_grpc::platform::v0::{get_contested_resource_vote_state_request, get_contested_resource_vote_state_response, get_contested_resources_request, get_contested_resources_response, get_vote_polls_by_end_date_request, get_vote_polls_by_end_date_response, GetContestedResourceVoteStateRequest, GetContestedResourceVoteStateResponse, GetVotePollsByEndDateRequest, GetContestedResourcesRequest, GetVotePollsByEndDateResponse};
    use dapi_grpc::platform::v0::get_contested_resource_vote_state_request::get_contested_resource_vote_state_request_v0::ResultType;
    use dapi_grpc::platform::v0::get_contested_resource_vote_state_request::GetContestedResourceVoteStateRequestV0;
    use dapi_grpc::platform::v0::get_contested_resource_vote_state_response::{get_contested_resource_vote_state_response_v0, GetContestedResourceVoteStateResponseV0};
    use dapi_grpc::platform::v0::get_vote_polls_by_end_date_request::GetVotePollsByEndDateRequestV0;
    use dapi_grpc::platform::v0::get_vote_polls_by_end_date_response::{get_vote_polls_by_end_date_response_v0, GetVotePollsByEndDateResponseV0};
    use dapi_grpc::platform::v0::get_vote_polls_by_end_date_response::get_vote_polls_by_end_date_response_v0::SerializedVotePollsByTimestamp;
    use dpp::state_transition::masternode_vote_transition::MasternodeVoteTransition;
    use dpp::voting::vote_choices::resource_vote_choice::ResourceVoteChoice::TowardsIdentity;
    use dpp::voting::vote_polls::contested_document_resource_vote_poll::ContestedDocumentResourceVotePoll;
    use dpp::voting::vote_polls::VotePoll;
    use dpp::voting::votes::resource_vote::ResourceVote;
    use dpp::voting::votes::resource_vote::v0::ResourceVoteV0;
    use dpp::voting::votes::Vote;
    use drive::drive::object_size_info::DataContractResolvedInfo;
    use drive::drive::votes::resolved::vote_polls::contested_document_resource_vote_poll::ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed;
    use drive::query::vote_poll_vote_state_query::ContestedDocumentVotePollDriveQueryResultType::DocumentsAndVoteTally;
    use drive::query::vote_poll_vote_state_query::ResolvedContestedDocumentVotePollDriveQuery;
    use dpp::document::serialization_traits::DocumentPlatformConversionMethodsV0;
    use dpp::identifier::Identifier;
    use dpp::identity::{IdentityPublicKey, IdentityV0};
    use dpp::prelude::{DataContract, Identity};
    use dpp::state_transition::masternode_vote_transition::methods::MasternodeVoteTransitionMethodsV0;
    use dpp::util::hash::hash_double;
    use dpp::util::strings::convert_to_homograph_safe_chars;
    use drive::query::vote_polls_by_document_type_query::ResolvedVotePollsByDocumentTypeQuery;
    use simple_signer::signer::SimpleSigner;
    use crate::platform_types::platform_state::PlatformState;
    use crate::rpc::core::MockCoreRPCLike;
    use crate::test::helpers::setup::TempPlatform;
    use dpp::serialization::PlatformDeserializable;
    use drive::query::VotePollsByEndDateDriveQuery;
    use crate::platform_types::platform_state::v0::PlatformStateV0Methods;
    use dpp::block::extended_block_info::v0::ExtendedBlockInfoV0;
    use dpp::platform_value::IdentifierBytes32;
    use dpp::platform_value::Value::Text;
    use dapi_grpc::platform::v0::{get_prefunded_specialized_balance_request, GetPrefundedSpecializedBalanceRequest};
    use dapi_grpc::platform::v0::get_prefunded_specialized_balance_request::GetPrefundedSpecializedBalanceRequestV0;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use arc_swap::Guard;
    use rand::Rng;
    use dapi_grpc::platform::v0::get_contested_resources_request::GetContestedResourcesRequestV0;
    use dapi_grpc::platform::v0::get_contested_resources_response::{get_contested_resources_response_v0, GetContestedResourcesResponseV0};
    use dapi_grpc::platform::v0::get_contested_resource_voters_for_identity_request::GetContestedResourceVotersForIdentityRequestV0;
    use dapi_grpc::platform::v0::get_contested_resource_voters_for_identity_response::{
        get_contested_resource_voters_for_identity_response_v0,
        GetContestedResourceVotersForIdentityResponseV0,
    };
    use dapi_grpc::platform::v0::{
        get_contested_resource_voters_for_identity_request,
        get_contested_resource_voters_for_identity_response,
        GetContestedResourceVotersForIdentityRequest,
    };
    use drive::query::vote_poll_contestant_votes_query::ResolvedContestedDocumentVotePollVotesDriveQuery;
    use dapi_grpc::platform::v0::get_contested_resource_voters_for_identity_request::get_contested_resource_voters_for_identity_request_v0;
    use dpp::platform_value;
    use dapi_grpc::platform::v0::get_contested_resource_identity_votes_request::{
        get_contested_resource_identity_votes_request_v0,
        GetContestedResourceIdentityVotesRequestV0,
    };
    use dapi_grpc::platform::v0::get_contested_resource_identity_votes_response::{
        get_contested_resource_identity_votes_response_v0,
        GetContestedResourceIdentityVotesResponseV0,
    };
    use dapi_grpc::platform::v0::{
        get_contested_resource_identity_votes_request,
        get_contested_resource_identity_votes_response,
        GetContestedResourceIdentityVotesRequest,
    };
    use dpp::voting::vote_choices::resource_vote_choice::ResourceVoteChoice::Lock;
    use drive::drive::votes::storage_form::contested_document_resource_storage_form::ContestedDocumentResourceVoteStorageForm;
    use crate::error::Error;
    use dashcore_rpc::dashcore_rpc_json::{DMNState, MasternodeListItem, MasternodeType};
    use dpp::dashcore::hashes::Hash;
    use dpp::dashcore::{ProTxHash, Txid};
    use dpp::prelude::IdentityNonce;
    use dpp::voting::vote_choices::resource_vote_choice::ResourceVoteChoice;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::ops::Deref;
    use dapi_grpc::platform::v0::get_contested_resource_vote_state_request::get_contested_resource_vote_state_request_v0;
    use drive::query::vote_poll_vote_state_query::Contender;
    use dapi_grpc::platform::v0::get_prefunded_specialized_balance_response;
    use dapi_grpc::platform::v0::get_prefunded_specialized_balance_response::{
        get_prefunded_specialized_balance_response_v0,
        GetPrefundedSpecializedBalanceResponseV0,
    };
    use dpp::fee::Credits;
    use drive::drive::Drive;

    mod vote_tests {
        use assert_matches::assert_matches;
        use drive::query::vote_poll_vote_state_query::ContestedDocumentVotePollDriveQueryResultType;
        use crate::platform_types::state_transitions_processing_result::StateTransitionExecutionResult::SuccessfulExecution;
        use super::*;

        fn setup_masternode_identity(
            platform: &mut TempPlatform<MockCoreRPCLike>,
            seed: u64,
            platform_version: &PlatformVersion,
        ) -> (Identity, SimpleSigner, IdentityPublicKey) {
            let mut signer = SimpleSigner::default();

            let mut rng = StdRng::seed_from_u64(seed);

            let (voting_key, voting_private_key) =
                IdentityPublicKey::random_voting_key_with_rng(0, &mut rng, platform_version)
                    .expect("expected to get key pair");

            signer.add_key(voting_key.clone(), voting_private_key.clone());

            let identity: Identity = IdentityV0 {
                id: Identifier::random_with_rng(&mut rng),
                public_keys: BTreeMap::from([(0, voting_key.clone())]),
                balance: 0,
                revision: 0,
            }
            .into();

            // We just add this identity to the system first

            platform
                .drive
                .add_new_identity(
                    identity.clone(),
                    true,
                    &BlockInfo::default(),
                    true,
                    None,
                    platform_version,
                )
                .expect("expected to add a new identity");

            let mut platform_state = platform.state.load().clone().deref().clone();

            let pro_tx_hash = ProTxHash::from_byte_array(identity.id().to_buffer());

            let random_ip = Ipv4Addr::new(
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
            );

            platform_state.full_masternode_list_mut().insert(
                pro_tx_hash,
                MasternodeListItem {
                    node_type: MasternodeType::Regular,
                    pro_tx_hash,
                    collateral_hash: Txid::from_byte_array(rng.gen()),
                    collateral_index: 0,
                    collateral_address: rng.gen(),
                    operator_reward: 0.0,
                    state: DMNState {
                        service: SocketAddr::new(IpAddr::V4(random_ip), 19999),
                        registered_height: 0,
                        pose_revived_height: None,
                        pose_ban_height: None,
                        revocation_reason: 0,
                        owner_address: rng.gen(),
                        voting_address: rng.gen(),
                        payout_address: rng.gen(),
                        pub_key_operator: vec![],
                        operator_payout_address: None,
                        platform_node_id: None,
                        platform_p2p_port: None,
                        platform_http_port: None,
                    },
                },
            );

            platform.state.store(Arc::new(platform_state));

            (identity, signer, voting_key)
        }

        fn create_dpns_name_contest(
            platform: &mut TempPlatform<MockCoreRPCLike>,
            platform_state: &PlatformState,
            seed: u64,
            name: &str,
            platform_version: &PlatformVersion,
        ) -> (Identity, Identity, Arc<DataContract>) {
            let mut rng = StdRng::seed_from_u64(seed);

            let (identity_1, signer_1, key_1) =
                setup_identity(platform, rng.gen(), dash_to_credits!(0.5));

            let (identity_2, signer_2, key_2) =
                setup_identity(platform, rng.gen(), dash_to_credits!(0.5));

            // Flip them if needed so identity 1 id is always smaller than identity 2 id
            let (identity_1, identity_2, signer_1, signer_2, key_1, key_2) =
                if identity_1.id() < identity_2.id() {
                    (identity_1, identity_2, signer_1, signer_2, key_1, key_2)
                } else {
                    (identity_2, identity_1, signer_2, signer_1, key_2, key_1)
                };

            let dpns = platform.drive.cache.system_data_contracts.load_dpns();
            let dpns_contract = dpns.clone();

            let preorder = dpns_contract
                .document_type_for_name("preorder")
                .expect("expected a profile document type");

            assert!(!preorder.documents_mutable());
            assert!(preorder.documents_can_be_deleted());
            assert!(!preorder.documents_transferable().is_transferable());

            let domain = dpns_contract
                .document_type_for_name("domain")
                .expect("expected a profile document type");

            assert!(!domain.documents_mutable());
            assert!(!domain.documents_can_be_deleted());
            assert!(!domain.documents_transferable().is_transferable());

            let entropy = Bytes32::random_with_rng(&mut rng);

            let mut preorder_document_1 = preorder
                .random_document_with_identifier_and_entropy(
                    &mut rng,
                    identity_1.id(),
                    entropy,
                    DocumentFieldFillType::FillIfNotRequired,
                    DocumentFieldFillSize::AnyDocumentFillSize,
                    platform_version,
                )
                .expect("expected a random document");

            let mut preorder_document_2 = preorder
                .random_document_with_identifier_and_entropy(
                    &mut rng,
                    identity_2.id(),
                    entropy,
                    DocumentFieldFillType::FillIfNotRequired,
                    DocumentFieldFillSize::AnyDocumentFillSize,
                    platform_version,
                )
                .expect("expected a random document");

            let mut document_1 = domain
                .random_document_with_identifier_and_entropy(
                    &mut rng,
                    identity_1.id(),
                    entropy,
                    DocumentFieldFillType::FillIfNotRequired,
                    DocumentFieldFillSize::AnyDocumentFillSize,
                    platform_version,
                )
                .expect("expected a random document");

            let mut document_2 = domain
                .random_document_with_identifier_and_entropy(
                    &mut rng,
                    identity_2.id(),
                    entropy,
                    DocumentFieldFillType::FillIfNotRequired,
                    DocumentFieldFillSize::AnyDocumentFillSize,
                    platform_version,
                )
                .expect("expected a random document");

            document_1.set("parentDomainName", "dash".into());
            document_1.set("normalizedParentDomainName", "dash".into());
            document_1.set("label", name.into());
            document_1.set(
                "normalizedLabel",
                convert_to_homograph_safe_chars(name).into(),
            );
            document_1.set("records.dashUniqueIdentityId", document_1.owner_id().into());
            document_1.set("subdomainRules.allowSubdomains", false.into());

            document_2.set("parentDomainName", "dash".into());
            document_2.set("normalizedParentDomainName", "dash".into());
            document_2.set("label", name.into());
            document_2.set(
                "normalizedLabel",
                convert_to_homograph_safe_chars(name).into(),
            );
            document_2.set("records.dashUniqueIdentityId", document_2.owner_id().into());
            document_2.set("subdomainRules.allowSubdomains", false.into());

            let salt_1: [u8; 32] = rng.gen();
            let salt_2: [u8; 32] = rng.gen();

            let mut salted_domain_buffer_1: Vec<u8> = vec![];
            salted_domain_buffer_1.extend(salt_1);
            salted_domain_buffer_1
                .extend((convert_to_homograph_safe_chars(name) + ".dash").as_bytes());

            let salted_domain_hash_1 = hash_double(salted_domain_buffer_1);

            let mut salted_domain_buffer_2: Vec<u8> = vec![];
            salted_domain_buffer_2.extend(salt_2);
            salted_domain_buffer_2
                .extend((convert_to_homograph_safe_chars(name) + ".dash").as_bytes());

            let salted_domain_hash_2 = hash_double(salted_domain_buffer_2);

            preorder_document_1.set("saltedDomainHash", salted_domain_hash_1.into());
            preorder_document_2.set("saltedDomainHash", salted_domain_hash_2.into());

            document_1.set("preorderSalt", salt_1.into());
            document_2.set("preorderSalt", salt_2.into());

            let documents_batch_create_preorder_transition_1 =
                DocumentsBatchTransition::new_document_creation_transition_from_document(
                    preorder_document_1,
                    preorder,
                    entropy.0,
                    &key_1,
                    2,
                    0,
                    &signer_1,
                    platform_version,
                    None,
                    None,
                    None,
                )
                .expect("expect to create documents batch transition");

            let documents_batch_create_serialized_preorder_transition_1 =
                documents_batch_create_preorder_transition_1
                    .serialize_to_bytes()
                    .expect("expected documents batch serialized state transition");

            let documents_batch_create_preorder_transition_2 =
                DocumentsBatchTransition::new_document_creation_transition_from_document(
                    preorder_document_2,
                    preorder,
                    entropy.0,
                    &key_2,
                    2,
                    0,
                    &signer_2,
                    platform_version,
                    None,
                    None,
                    None,
                )
                .expect("expect to create documents batch transition");

            let documents_batch_create_serialized_preorder_transition_2 =
                documents_batch_create_preorder_transition_2
                    .serialize_to_bytes()
                    .expect("expected documents batch serialized state transition");

            let documents_batch_create_transition_1 =
                DocumentsBatchTransition::new_document_creation_transition_from_document(
                    document_1,
                    domain,
                    entropy.0,
                    &key_1,
                    3,
                    0,
                    &signer_1,
                    platform_version,
                    None,
                    None,
                    None,
                )
                .expect("expect to create documents batch transition");

            let documents_batch_create_serialized_transition_1 =
                documents_batch_create_transition_1
                    .serialize_to_bytes()
                    .expect("expected documents batch serialized state transition");

            let documents_batch_create_transition_2 =
                DocumentsBatchTransition::new_document_creation_transition_from_document(
                    document_2,
                    domain,
                    entropy.0,
                    &key_2,
                    3,
                    0,
                    &signer_2,
                    platform_version,
                    None,
                    None,
                    None,
                )
                .expect("expect to create documents batch transition");

            let documents_batch_create_serialized_transition_2 =
                documents_batch_create_transition_2
                    .serialize_to_bytes()
                    .expect("expected documents batch serialized state transition");

            let transaction = platform.drive.grove.start_transaction();

            let processing_result = platform
                .platform
                .process_raw_state_transitions(
                    &vec![
                        documents_batch_create_serialized_preorder_transition_1.clone(),
                        documents_batch_create_serialized_preorder_transition_2.clone(),
                    ],
                    platform_state,
                    &BlockInfo::default_with_time(
                        platform_state
                            .last_committed_block_time_ms()
                            .unwrap_or_default()
                            + 3000,
                    ),
                    &transaction,
                    platform_version,
                )
                .expect("expected to process state transition");

            platform
                .drive
                .grove
                .commit_transaction(transaction)
                .unwrap()
                .expect("expected to commit transaction");

            assert_eq!(processing_result.valid_count(), 2);

            let transaction = platform.drive.grove.start_transaction();

            let processing_result = platform
                .platform
                .process_raw_state_transitions(
                    &vec![
                        documents_batch_create_serialized_transition_1.clone(),
                        documents_batch_create_serialized_transition_2.clone(),
                    ],
                    platform_state,
                    &BlockInfo::default_with_time(
                        platform_state
                            .last_committed_block_time_ms()
                            .unwrap_or_default()
                            + 3000,
                    ),
                    &transaction,
                    platform_version,
                )
                .expect("expected to process state transition");

            platform
                .drive
                .grove
                .commit_transaction(transaction)
                .unwrap()
                .expect("expected to commit transaction");

            assert_eq!(processing_result.valid_count(), 2);
            (identity_1, identity_2, dpns_contract)
        }

        fn verify_dpns_name_contest(
            platform: &mut TempPlatform<MockCoreRPCLike>,
            platform_state: &Guard<Arc<PlatformState>>,
            dpns_contract: &DataContract,
            identity_1: &Identity,
            identity_2: &Identity,
            name: &str,
            platform_version: &PlatformVersion,
        ) {
            // Now let's run a query for the vote totals

            let domain = dpns_contract
                .document_type_for_name("domain")
                .expect("expected a profile document type");

            let config = bincode::config::standard()
                .with_big_endian()
                .with_no_limit();

            let dash_encoded = bincode::encode_to_vec(Value::Text("dash".to_string()), config)
                .expect("expected to encode the word dash");

            let quantum_encoded =
                bincode::encode_to_vec(Value::Text(convert_to_homograph_safe_chars(name)), config)
                    .expect("expected to encode the word quantum");

            let index_name = "parentNameAndLabel".to_string();

            let query_validation_result = platform
                .query_contested_resource_vote_state(
                    GetContestedResourceVoteStateRequest {
                        version: Some(get_contested_resource_vote_state_request::Version::V0(
                            GetContestedResourceVoteStateRequestV0 {
                                contract_id: dpns_contract.id().to_vec(),
                                document_type_name: domain.name().clone(),
                                index_name: index_name.clone(),
                                index_values: vec![dash_encoded.clone(), quantum_encoded.clone()],
                                result_type: ResultType::DocumentsAndVoteTally as i32,
                                allow_include_locked_and_abstaining_vote_tally: true,
                                start_at_identifier_info: None,
                                count: None,
                                order_ascending: true,
                                prove: false,
                            },
                        )),
                    },
                    platform_state,
                    platform_version,
                )
                .expect("expected to execute query")
                .into_data()
                .expect("expected query to be valid");

            let get_contested_resource_vote_state_response::Version::V0(
                GetContestedResourceVoteStateResponseV0 {
                    metadata: _,
                    result,
                },
            ) = query_validation_result.version.expect("expected a version");

            let Some(
                get_contested_resource_vote_state_response_v0::Result::ContestedResourceContenders(
                    get_contested_resource_vote_state_response_v0::ContestedResourceContenders {
                        contenders,
                        ..
                    },
                ),
            ) = result
            else {
                panic!("expected contenders")
            };

            assert_eq!(contenders.len(), 2);

            let first_contender = contenders.first().unwrap();

            let second_contender = contenders.last().unwrap();

            let first_contender_document = Document::from_bytes(
                first_contender
                    .document
                    .as_ref()
                    .expect("expected a document")
                    .as_slice(),
                domain,
                platform_version,
            )
            .expect("expected to get document");

            let second_contender_document = Document::from_bytes(
                second_contender
                    .document
                    .as_ref()
                    .expect("expected a document")
                    .as_slice(),
                domain,
                platform_version,
            )
            .expect("expected to get document");

            assert_ne!(first_contender_document, second_contender_document);

            assert_eq!(first_contender.identifier, identity_1.id().to_vec());

            assert_eq!(second_contender.identifier, identity_2.id().to_vec());

            assert_eq!(first_contender.vote_count, Some(0));

            assert_eq!(second_contender.vote_count, Some(0));

            let GetContestedResourceVoteStateResponse { version } = platform
                .query_contested_resource_vote_state(
                    GetContestedResourceVoteStateRequest {
                        version: Some(get_contested_resource_vote_state_request::Version::V0(
                            GetContestedResourceVoteStateRequestV0 {
                                contract_id: dpns_contract.id().to_vec(),
                                document_type_name: domain.name().clone(),
                                index_name: "parentNameAndLabel".to_string(),
                                index_values: vec![dash_encoded, quantum_encoded],
                                result_type: ResultType::DocumentsAndVoteTally as i32,
                                allow_include_locked_and_abstaining_vote_tally: true,
                                start_at_identifier_info: None,
                                count: None,
                                order_ascending: true,
                                prove: true,
                            },
                        )),
                    },
                    platform_state,
                    platform_version,
                )
                .expect("expected to execute query")
                .into_data()
                .expect("expected query to be valid");

            let get_contested_resource_vote_state_response::Version::V0(
                GetContestedResourceVoteStateResponseV0 {
                    metadata: _,
                    result,
                },
            ) = version.expect("expected a version");

            let Some(get_contested_resource_vote_state_response_v0::Result::Proof(proof)) = result
            else {
                panic!("expected contenders")
            };

            let resolved_contested_document_vote_poll_drive_query =
                ResolvedContestedDocumentVotePollDriveQuery {
                    vote_poll: ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
                        contract: DataContractResolvedInfo::BorrowedDataContract(dpns_contract),
                        document_type_name: domain.name().clone(),
                        index_name: index_name.clone(),
                        index_values: vec![
                            Value::Text("dash".to_string()),
                            Value::Text(convert_to_homograph_safe_chars(name)),
                        ],
                    },
                    result_type: DocumentsAndVoteTally,
                    offset: None,
                    limit: None,
                    start_at: None,
                    order_ascending: true,
                    allow_include_locked_and_abstaining_vote_tally: true,
                };

            let (_, result) = resolved_contested_document_vote_poll_drive_query
                .verify_vote_poll_vote_state_proof(proof.grovedb_proof.as_ref(), platform_version)
                .expect("expected to verify proof");

            let contenders = result.contenders;

            assert_eq!(contenders.len(), 2);

            let first_contender = contenders.first().unwrap();

            let second_contender = contenders.last().unwrap();

            let first_contender_document = Document::from_bytes(
                first_contender
                    .serialized_document
                    .as_ref()
                    .expect("expected a document")
                    .as_slice(),
                domain,
                platform_version,
            )
            .expect("expected to get document");

            let second_contender_document = Document::from_bytes(
                second_contender
                    .serialized_document
                    .as_ref()
                    .expect("expected a document")
                    .as_slice(),
                domain,
                platform_version,
            )
            .expect("expected to get document");

            assert_ne!(first_contender_document, second_contender_document);

            assert_eq!(first_contender.identity_id, identity_1.id());

            assert_eq!(second_contender.identity_id, identity_2.id());

            assert_eq!(first_contender.vote_tally, Some(0));

            assert_eq!(second_contender.vote_tally, Some(0));
        }

        fn perform_vote(
            platform: &mut TempPlatform<MockCoreRPCLike>,
            platform_state: &Guard<Arc<PlatformState>>,
            dpns_contract: &DataContract,
            resource_vote_choice: ResourceVoteChoice,
            name: &str,
            signer: &SimpleSigner,
            masternode_id: Identifier,
            voting_key: &IdentityPublicKey,
            nonce: IdentityNonce,
            platform_version: &PlatformVersion,
        ) {
            // Let's vote for contender 1

            let vote = Vote::ResourceVote(ResourceVote::V0(ResourceVoteV0 {
                vote_poll: VotePoll::ContestedDocumentResourceVotePoll(
                    ContestedDocumentResourceVotePoll {
                        contract_id: dpns_contract.id(),
                        document_type_name: "domain".to_string(),
                        index_name: "parentNameAndLabel".to_string(),
                        index_values: vec![
                            Value::Text("dash".to_string()),
                            Value::Text(convert_to_homograph_safe_chars(name)),
                        ],
                    },
                ),
                resource_vote_choice,
            }));

            let masternode_vote_transition = MasternodeVoteTransition::try_from_vote_with_signer(
                vote,
                signer,
                masternode_id,
                voting_key,
                nonce,
                platform_version,
                None,
            )
            .expect("expected to make transition vote");

            let masternode_vote_serialized_transition = masternode_vote_transition
                .serialize_to_bytes()
                .expect("expected documents batch serialized state transition");

            let transaction = platform.drive.grove.start_transaction();

            let processing_result = platform
                .platform
                .process_raw_state_transitions(
                    &vec![masternode_vote_serialized_transition.clone()],
                    platform_state,
                    &BlockInfo::default(),
                    &transaction,
                    platform_version,
                )
                .expect("expected to process state transition");

            platform
                .drive
                .grove
                .commit_transaction(transaction)
                .unwrap()
                .expect("expected to commit transaction");

            let execution_result = processing_result.into_execution_results().remove(0);
            assert_matches!(execution_result, SuccessfulExecution(..));
        }

        fn perform_votes(
            platform: &mut TempPlatform<MockCoreRPCLike>,
            dpns_contract: &DataContract,
            resource_vote_choice: ResourceVoteChoice,
            name: &str,
            count: u64,
            start_seed: u64,
            platform_version: &PlatformVersion,
        ) {
            for i in 0..count {
                let (masternode, signer, voting_key) =
                    setup_masternode_identity(platform, start_seed + i, platform_version);

                let platform_state = platform.state.load();

                perform_vote(
                    platform,
                    &platform_state,
                    dpns_contract,
                    resource_vote_choice,
                    name,
                    &signer,
                    masternode.id(),
                    &voting_key,
                    1,
                    platform_version,
                );
            }
        }

        fn perform_votes_multi(
            platform: &mut TempPlatform<MockCoreRPCLike>,
            dpns_contract: &DataContract,
            resource_vote_choices: Vec<(ResourceVoteChoice, u64)>,
            name: &str,
            start_seed: u64,
            platform_version: &PlatformVersion,
        ) {
            let mut count_aggregate = start_seed;
            for (resource_vote_choice, count) in resource_vote_choices.into_iter() {
                perform_votes(
                    platform,
                    dpns_contract,
                    resource_vote_choice,
                    name,
                    count,
                    count_aggregate,
                    platform_version,
                );
                count_aggregate += count;
            }
        }

        fn get_vote_states(
            platform: &TempPlatform<MockCoreRPCLike>,
            platform_state: &PlatformState,
            dpns_contract: &DataContract,
            name: &str,
            count: Option<u32>,
            order_ascending: bool,
            allow_include_locked_and_abstaining_vote_tally: bool,
            start_at_identifier_info: Option<
                get_contested_resource_vote_state_request_v0::StartAtIdentifierInfo,
            >,
            result_type: ResultType,
            platform_version: &PlatformVersion,
        ) -> (Vec<Contender>, Option<u32>, Option<u32>) {
            // Now let's run a query for the vote totals

            let domain = dpns_contract
                .document_type_for_name("domain")
                .expect("expected a profile document type");

            let config = bincode::config::standard()
                .with_big_endian()
                .with_no_limit();

            let dash_encoded = bincode::encode_to_vec(Value::Text("dash".to_string()), config)
                .expect("expected to encode the word dash");

            let quantum_encoded =
                bincode::encode_to_vec(Value::Text(convert_to_homograph_safe_chars(name)), config)
                    .expect("expected to encode the word quantum");

            let index_name = "parentNameAndLabel".to_string();

            let query_validation_result = platform
                .query_contested_resource_vote_state(
                    GetContestedResourceVoteStateRequest {
                        version: Some(get_contested_resource_vote_state_request::Version::V0(
                            GetContestedResourceVoteStateRequestV0 {
                                contract_id: dpns_contract.id().to_vec(),
                                document_type_name: domain.name().clone(),
                                index_name: index_name.clone(),
                                index_values: vec![dash_encoded.clone(), quantum_encoded.clone()],
                                result_type: result_type as i32,
                                allow_include_locked_and_abstaining_vote_tally,
                                start_at_identifier_info,
                                count,
                                order_ascending,
                                prove: false,
                            },
                        )),
                    },
                    platform_state,
                    platform_version,
                )
                .expect("expected to execute query")
                .into_data()
                .expect("expected query to be valid");

            let get_contested_resource_vote_state_response::Version::V0(
                GetContestedResourceVoteStateResponseV0 {
                    metadata: _,
                    result,
                },
            ) = query_validation_result.version.expect("expected a version");

            let Some(
                get_contested_resource_vote_state_response_v0::Result::ContestedResourceContenders(
                    get_contested_resource_vote_state_response_v0::ContestedResourceContenders {
                        contenders,
                        abstain_vote_tally,
                        lock_vote_tally,
                    },
                ),
            ) = result
            else {
                panic!("expected contenders")
            };
            (
                contenders
                    .into_iter()
                    .map(|contender| Contender {
                        identity_id: contender.identifier.try_into().expect("expected 32 bytes"),
                        document: contender.document.map(|document_bytes| {
                            Document::from_bytes(
                                document_bytes.as_slice(),
                                domain,
                                platform_version,
                            )
                            .expect("expected to deserialize document")
                        }),
                        vote_tally: contender.vote_count,
                    })
                    .collect(),
                abstain_vote_tally,
                lock_vote_tally,
            )
        }

        fn get_proved_vote_states(
            platform: &TempPlatform<MockCoreRPCLike>,
            platform_state: &PlatformState,
            dpns_contract: &DataContract,
            name: &str,
            count: Option<u32>,
            order_ascending: bool,
            allow_include_locked_and_abstaining_vote_tally: bool,
            start_at_identifier_info: Option<
                get_contested_resource_vote_state_request_v0::StartAtIdentifierInfo,
            >,
            result_type: ResultType,
            platform_version: &PlatformVersion,
        ) -> (Vec<Contender>, Option<u32>, Option<u32>) {
            // Now let's run a query for the vote totals

            let domain = dpns_contract
                .document_type_for_name("domain")
                .expect("expected a profile document type");

            let config = bincode::config::standard()
                .with_big_endian()
                .with_no_limit();

            let dash_encoded = bincode::encode_to_vec(Value::Text("dash".to_string()), config)
                .expect("expected to encode the word dash");

            let quantum_encoded =
                bincode::encode_to_vec(Value::Text(convert_to_homograph_safe_chars(name)), config)
                    .expect("expected to encode the word quantum");

            let index_name = "parentNameAndLabel".to_string();

            let query_validation_result = platform
                .query_contested_resource_vote_state(
                    GetContestedResourceVoteStateRequest {
                        version: Some(get_contested_resource_vote_state_request::Version::V0(
                            GetContestedResourceVoteStateRequestV0 {
                                contract_id: dpns_contract.id().to_vec(),
                                document_type_name: domain.name().clone(),
                                index_name: index_name.clone(),
                                index_values: vec![dash_encoded.clone(), quantum_encoded.clone()],
                                result_type: result_type as i32,
                                allow_include_locked_and_abstaining_vote_tally,
                                start_at_identifier_info,
                                count,
                                order_ascending,
                                prove: true,
                            },
                        )),
                    },
                    platform_state,
                    platform_version,
                )
                .expect("expected to execute query")
                .into_data()
                .expect("expected query to be valid");

            let get_contested_resource_vote_state_response::Version::V0(
                GetContestedResourceVoteStateResponseV0 {
                    metadata: _,
                    result,
                },
            ) = query_validation_result.version.expect("expected a version");

            let Some(get_contested_resource_vote_state_response_v0::Result::Proof(proof)) = result
            else {
                panic!("expected contenders")
            };

            let resolved_contested_document_vote_poll_drive_query =
                ResolvedContestedDocumentVotePollDriveQuery {
                    vote_poll: ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
                        contract: DataContractResolvedInfo::BorrowedDataContract(dpns_contract),
                        document_type_name: domain.name().clone(),
                        index_name: index_name.clone(),
                        index_values: vec![
                            Value::Text("dash".to_string()),
                            Value::Text("quantum".to_string()),
                        ],
                    },
                    result_type: ContestedDocumentVotePollDriveQueryResultType::try_from(
                        result_type as i32,
                    )
                    .expect("expected valid result type"),
                    offset: None,
                    limit: count.map(|a| a as u16),
                    start_at: None,
                    order_ascending,
                    allow_include_locked_and_abstaining_vote_tally,
                };

            let (_, result) = resolved_contested_document_vote_poll_drive_query
                .verify_vote_poll_vote_state_proof(proof.grovedb_proof.as_ref(), platform_version)
                .expect("expected to verify proof");

            let abstaining_vote_tally = result.abstaining_vote_tally;
            let lock_vote_tally = result.locked_vote_tally;

            let contenders = result.contenders;
            (
                contenders
                    .into_iter()
                    .map(|contender| Contender {
                        identity_id: contender.identity_id,
                        document: contender.serialized_document.map(|document_bytes| {
                            Document::from_bytes(
                                document_bytes.as_slice(),
                                domain,
                                platform_version,
                            )
                            .expect("expected to deserialize document")
                        }),
                        vote_tally: contender.vote_tally,
                    })
                    .collect(),
                abstaining_vote_tally,
                lock_vote_tally,
            )
        }

        mod contests_requests_query {
            use super::*;
            #[test]
            fn test_not_proved_contests_request() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (identity_1, identity_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                verify_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    &identity_1,
                    &identity_2,
                    "quantum",
                    platform_version,
                );

                let (identity_3, identity_4, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    8,
                    "cooldog",
                    platform_version,
                );

                verify_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    &identity_3,
                    &identity_4,
                    "cooldog",
                    platform_version,
                );

                let domain = dpns_contract
                    .document_type_for_name("domain")
                    .expect("expected a profile document type");

                let index_name = "parentNameAndLabel".to_string();

                let config = bincode::config::standard()
                    .with_big_endian()
                    .with_no_limit();

                let dash_encoded = bincode::encode_to_vec(Value::Text("dash".to_string()), config)
                    .expect("expected to encode value");

                let query_validation_result = platform
                    .query_contested_resources(
                        GetContestedResourcesRequest {
                            version: Some(get_contested_resources_request::Version::V0(
                                GetContestedResourcesRequestV0 {
                                    contract_id: dpns_contract.id().to_vec(),
                                    document_type_name: domain.name().clone(),
                                    index_name: index_name.clone(),
                                    start_index_values: vec![dash_encoded.clone()],
                                    end_index_values: vec![],
                                    start_at_value_info: None,
                                    count: None,
                                    order_ascending: true,
                                    prove: false,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_contested_resources_response::Version::V0(
                    GetContestedResourcesResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = query_validation_result.version.expect("expected a version");

                let Some(get_contested_resources_response_v0::Result::ContestedResourceValues(
                    get_contested_resources_response_v0::ContestedResourceValues {
                        contested_resource_values,
                    },
                )) = result
                else {
                    panic!("expected contested resources")
                };

                assert_eq!(contested_resource_values.len(), 2);
            }

            #[test]
            fn test_proved_contests_request() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (identity_1, identity_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                let (identity_3, identity_4, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    8,
                    "cooldog",
                    platform_version,
                );

                let domain = dpns_contract
                    .document_type_for_name("domain")
                    .expect("expected a profile document type");

                let index_name = "parentNameAndLabel".to_string();

                let config = bincode::config::standard()
                    .with_big_endian()
                    .with_no_limit();

                let dash_encoded = bincode::encode_to_vec(Value::Text("dash".to_string()), config)
                    .expect("expected to encode value");

                let query_validation_result = platform
                    .query_contested_resources(
                        GetContestedResourcesRequest {
                            version: Some(get_contested_resources_request::Version::V0(
                                GetContestedResourcesRequestV0 {
                                    contract_id: dpns_contract.id().to_vec(),
                                    document_type_name: domain.name().clone(),
                                    index_name: index_name.clone(),
                                    start_index_values: vec![dash_encoded],
                                    end_index_values: vec![],
                                    start_at_value_info: None,
                                    count: None,
                                    order_ascending: true,
                                    prove: true,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_contested_resources_response::Version::V0(
                    GetContestedResourcesResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = query_validation_result.version.expect("expected a version");

                let Some(get_contested_resources_response_v0::Result::Proof(proof)) = result else {
                    panic!("expected proof")
                };

                let resolved_contested_document_vote_poll_drive_query =
                    ResolvedVotePollsByDocumentTypeQuery {
                        contract: DataContractResolvedInfo::BorrowedDataContract(
                            dpns_contract.as_ref(),
                        ),
                        document_type_name: domain.name(),
                        index_name: &index_name,
                        start_index_values: &vec!["dash".into()],
                        end_index_values: &vec![],
                        limit: None,
                        order_ascending: true,
                        start_at_value: &None,
                    };

                let (_, contests) = resolved_contested_document_vote_poll_drive_query
                    .verify_contests_proof(proof.grovedb_proof.as_ref(), platform_version)
                    .expect("expected to verify proof");

                assert_eq!(contests.len(), 2);
            }
        }

        mod vote_state_query {
            use super::*;

            #[test]
            fn test_not_proved_vote_state_query_request_after_vote() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1, contender_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                let (masternode_1, signer_1, voting_key_1) =
                    setup_masternode_identity(&mut platform, 29, platform_version);

                let platform_state = platform.state.load();

                perform_vote(
                    &mut platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    TowardsIdentity(contender_1.id()),
                    "quantum",
                    &signer_1,
                    masternode_1.id(),
                    &voting_key_1,
                    1,
                    platform_version,
                );

                let (contenders, abstaining, locking) = get_vote_states(
                    &platform,
                    &platform_state,
                    &dpns_contract,
                    "quantum",
                    None,
                    true,
                    true,
                    None,
                    ResultType::DocumentsAndVoteTally,
                    platform_version,
                );

                assert_eq!(contenders.len(), 2);

                let first_contender = contenders.first().unwrap();

                let second_contender = contenders.last().unwrap();

                assert_ne!(first_contender.document, second_contender.document);

                assert_eq!(first_contender.identity_id, contender_1.id());

                assert_eq!(second_contender.identity_id, contender_2.id());

                assert_eq!(first_contender.vote_tally, Some(1));

                assert_eq!(second_contender.vote_tally, Some(0));

                assert_eq!(abstaining, Some(0));

                assert_eq!(locking, Some(0));
            }

            #[test]
            fn test_proved_vote_state_query_request_after_vote() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1, contender_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                let (masternode_1, signer_1, voting_key_1) =
                    setup_masternode_identity(&mut platform, 29, platform_version);

                let platform_state = platform.state.load();

                perform_vote(
                    &mut platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    TowardsIdentity(contender_1.id()),
                    "quantum",
                    &signer_1,
                    masternode_1.id(),
                    &voting_key_1,
                    1,
                    platform_version,
                );

                let (contenders, abstaining, locking) = get_proved_vote_states(
                    &platform,
                    &platform_state,
                    &dpns_contract,
                    "quantum",
                    None,
                    true,
                    true,
                    None,
                    ResultType::DocumentsAndVoteTally,
                    platform_version,
                );

                assert_eq!(contenders.len(), 2);

                let first_contender = contenders.first().unwrap();

                let second_contender = contenders.last().unwrap();

                assert_ne!(first_contender.document, second_contender.document);

                assert_eq!(first_contender.identity_id, contender_1.id());

                assert_eq!(second_contender.identity_id, contender_2.id());

                assert_eq!(first_contender.vote_tally, Some(1));

                assert_eq!(second_contender.vote_tally, Some(0));

                assert_eq!(abstaining, Some(0));

                assert_eq!(locking, Some(0));
            }

            #[test]
            fn test_not_proved_vote_state_query_request_after_many_votes() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1, contender_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                perform_votes_multi(
                    &mut platform,
                    dpns_contract.as_ref(),
                    vec![
                        (TowardsIdentity(contender_1.id()), 50),
                        (TowardsIdentity(contender_2.id()), 5),
                        (ResourceVoteChoice::Abstain, 10),
                        (ResourceVoteChoice::Lock, 3),
                    ],
                    "quantum",
                    10,
                    platform_version,
                );

                let (contenders, abstaining, locking) = get_vote_states(
                    &platform,
                    &platform_state,
                    &dpns_contract,
                    "quantum",
                    None,
                    true,
                    true,
                    None,
                    ResultType::DocumentsAndVoteTally,
                    platform_version,
                );

                assert_eq!(contenders.len(), 2);

                let first_contender = contenders.first().unwrap();

                let second_contender = contenders.last().unwrap();

                assert_ne!(first_contender.document, second_contender.document);

                assert_eq!(first_contender.identity_id, contender_1.id());

                assert_eq!(second_contender.identity_id, contender_2.id());

                assert_eq!(first_contender.vote_tally, Some(50));

                assert_eq!(second_contender.vote_tally, Some(5));

                assert_eq!(abstaining, Some(10));

                assert_eq!(locking, Some(3));

                // Now let's not include locked and abstaining

                let (contenders, abstaining, locking) = get_vote_states(
                    &platform,
                    &platform_state,
                    &dpns_contract,
                    "quantum",
                    None,
                    true,
                    false,
                    None,
                    ResultType::DocumentsAndVoteTally,
                    platform_version,
                );

                assert_eq!(contenders.len(), 2);

                let first_contender = contenders.first().unwrap();

                let second_contender = contenders.last().unwrap();

                assert_ne!(first_contender.document, second_contender.document);

                assert_eq!(first_contender.identity_id, contender_1.id());

                assert_eq!(second_contender.identity_id, contender_2.id());

                assert_eq!(first_contender.vote_tally, Some(50));

                assert_eq!(second_contender.vote_tally, Some(5));

                assert_eq!(abstaining, None);

                assert_eq!(locking, None);
            }

            #[test]
            fn test_proved_vote_state_query_request_after_many_votes() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1, contender_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                perform_votes_multi(
                    &mut platform,
                    dpns_contract.as_ref(),
                    vec![
                        (TowardsIdentity(contender_1.id()), 50),
                        (TowardsIdentity(contender_2.id()), 5),
                        (ResourceVoteChoice::Abstain, 10),
                        (ResourceVoteChoice::Lock, 3),
                    ],
                    "quantum",
                    10,
                    platform_version,
                );

                let (contenders, abstaining, locking) = get_proved_vote_states(
                    &platform,
                    &platform_state,
                    &dpns_contract,
                    "quantum",
                    None,
                    true,
                    true,
                    None,
                    ResultType::DocumentsAndVoteTally,
                    platform_version,
                );

                assert_eq!(contenders.len(), 2);

                let first_contender = contenders.first().unwrap();

                let second_contender = contenders.last().unwrap();

                assert_ne!(first_contender.document, second_contender.document);

                assert_eq!(first_contender.identity_id, contender_1.id());

                assert_eq!(second_contender.identity_id, contender_2.id());

                assert_eq!(first_contender.vote_tally, Some(50));

                assert_eq!(second_contender.vote_tally, Some(5));

                assert_eq!(abstaining, Some(10));

                assert_eq!(locking, Some(3));

                // Now let's not include locked and abstaining

                let (contenders, abstaining, locking) = get_proved_vote_states(
                    &platform,
                    &platform_state,
                    &dpns_contract,
                    "quantum",
                    None,
                    true,
                    false,
                    None,
                    ResultType::DocumentsAndVoteTally,
                    platform_version,
                );

                assert_eq!(contenders.len(), 2);

                let first_contender = contenders.first().unwrap();

                let second_contender = contenders.last().unwrap();

                assert_ne!(first_contender.document, second_contender.document);

                assert_eq!(first_contender.identity_id, contender_1.id());

                assert_eq!(second_contender.identity_id, contender_2.id());

                assert_eq!(first_contender.vote_tally, Some(50));

                assert_eq!(second_contender.vote_tally, Some(5));

                assert_eq!(abstaining, None);

                assert_eq!(locking, None);
            }
        }

        mod contestant_received_votes_query {
            use super::*;

            fn get_contestant_votes(
                platform: &TempPlatform<MockCoreRPCLike>,
                platform_state: &PlatformState,
                dpns_contract: &DataContract,
                contender_id: Identifier,
                name: &str,
                count: Option<u32>,
                order_ascending: bool,
                start_at_identifier_info: Option<
                    get_contested_resource_voters_for_identity_request_v0::StartAtIdentifierInfo,
                >,
                should_be_finished: bool,
                platform_version: &PlatformVersion,
            ) -> Vec<Identifier> {
                let domain = dpns_contract
                    .document_type_for_name("domain")
                    .expect("expected a profile document type");

                let config = bincode::config::standard()
                    .with_big_endian()
                    .with_no_limit();

                let dash_encoded = bincode::encode_to_vec(Value::Text("dash".to_string()), config)
                    .expect("expected to encode the word dash");

                let quantum_encoded = bincode::encode_to_vec(
                    Value::Text(convert_to_homograph_safe_chars(name)),
                    config,
                )
                .expect("expected to encode the word quantum");

                let index_name = "parentNameAndLabel".to_string();

                let query_validation_result = platform
                    .query_contested_resource_voters_for_identity(
                        GetContestedResourceVotersForIdentityRequest {
                            version: Some(
                                get_contested_resource_voters_for_identity_request::Version::V0(
                                    GetContestedResourceVotersForIdentityRequestV0 {
                                        contract_id: dpns_contract.id().to_vec(),
                                        document_type_name: domain.name().clone(),
                                        index_name: index_name.clone(),
                                        index_values: vec![
                                            dash_encoded.clone(),
                                            quantum_encoded.clone(),
                                        ],
                                        contestant_id: contender_id.to_vec(),
                                        start_at_identifier_info,
                                        count,
                                        order_ascending,
                                        prove: false,
                                    },
                                ),
                            ),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_contested_resource_voters_for_identity_response::Version::V0(
                    GetContestedResourceVotersForIdentityResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = query_validation_result.version.expect("expected a version");

                let Some(
                    get_contested_resource_voters_for_identity_response_v0::Result::ContestedResourceVoters(
                        get_contested_resource_voters_for_identity_response_v0::ContestedResourceVoters {
                            voters,
                            finished_results,
                        },
                    ),
                ) = result
                    else {
                        panic!("expected contenders")
                    };
                if should_be_finished {
                    assert!(finished_results);
                }

                voters
                    .into_iter()
                    .map(Identifier::from_vec)
                    .collect::<Result<Vec<Identifier>, platform_value::Error>>()
                    .expect("expected all voters to be identifiers")
            }

            fn get_proved_contestant_votes(
                platform: &TempPlatform<MockCoreRPCLike>,
                platform_state: &PlatformState,
                dpns_contract: &DataContract,
                contender_id: Identifier,
                name: &str,
                count: Option<u32>,
                order_ascending: bool,
                start_at_identifier_info: Option<
                    get_contested_resource_voters_for_identity_request_v0::StartAtIdentifierInfo,
                >,
                platform_version: &PlatformVersion,
            ) -> Vec<Identifier> {
                let domain = dpns_contract
                    .document_type_for_name("domain")
                    .expect("expected a profile document type");

                let config = bincode::config::standard()
                    .with_big_endian()
                    .with_no_limit();

                let dash_encoded = bincode::encode_to_vec(Value::Text("dash".to_string()), config)
                    .expect("expected to encode the word dash");

                let quantum_encoded = bincode::encode_to_vec(
                    Value::Text(convert_to_homograph_safe_chars(name)),
                    config,
                )
                .expect("expected to encode the word quantum");

                let index_name = "parentNameAndLabel".to_string();

                let query_validation_result = platform
                    .query_contested_resource_voters_for_identity(
                        GetContestedResourceVotersForIdentityRequest {
                            version: Some(
                                get_contested_resource_voters_for_identity_request::Version::V0(
                                    GetContestedResourceVotersForIdentityRequestV0 {
                                        contract_id: dpns_contract.id().to_vec(),
                                        document_type_name: domain.name().clone(),
                                        index_name: index_name.clone(),
                                        index_values: vec![
                                            dash_encoded.clone(),
                                            quantum_encoded.clone(),
                                        ],
                                        contestant_id: contender_id.to_vec(),
                                        start_at_identifier_info,
                                        count,
                                        order_ascending,
                                        prove: true,
                                    },
                                ),
                            ),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_contested_resource_voters_for_identity_response::Version::V0(
                    GetContestedResourceVotersForIdentityResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = query_validation_result.version.expect("expected a version");

                let Some(get_contested_resource_voters_for_identity_response_v0::Result::Proof(
                    proof,
                )) = result
                else {
                    panic!("expected contenders")
                };

                let resolved_contested_document_vote_poll_drive_query =
                    ResolvedContestedDocumentVotePollVotesDriveQuery {
                        vote_poll: ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
                            contract: DataContractResolvedInfo::BorrowedDataContract(
                                &dpns_contract,
                            ),
                            document_type_name: domain.name().clone(),
                            index_name: index_name.clone(),
                            index_values: vec![
                                Value::Text("dash".to_string()),
                                Value::Text("quantum".to_string()),
                            ],
                        },
                        contestant_id: contender_id,
                        offset: None,
                        limit: None,
                        start_at: None,
                        order_ascending: true,
                    };

                let (_, voters) = resolved_contested_document_vote_poll_drive_query
                    .verify_vote_poll_votes_proof(proof.grovedb_proof.as_ref(), platform_version)
                    .expect("expected to verify proof");

                voters
            }

            #[test]
            fn test_non_proved_contestant_votes_query_request() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1, contender_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                let (contender_3, _, _) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "quantum",
                    platform_version,
                );

                for i in 0..50 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 10 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_1.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                for i in 0..5 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 100 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_2.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                for i in 0..8 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 200 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_3.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }
                let voters = get_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_1.id(),
                    "quantum",
                    None,
                    true,
                    None,
                    true,
                    platform_version,
                );
                assert_eq!(voters.len(), 50);

                let voters_2 = get_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_2.id(),
                    "quantum",
                    None,
                    true,
                    None,
                    true,
                    platform_version,
                );

                assert_eq!(voters_2.len(), 5);

                let voters_3 = get_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_3.id(),
                    "quantum",
                    None,
                    true,
                    None,
                    true,
                    platform_version,
                );

                let mut voters_3_desc = get_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_3.id(),
                    "quantum",
                    None,
                    false,
                    None,
                    true,
                    platform_version,
                );

                voters_3_desc.reverse();

                assert_eq!(voters_3, voters_3_desc);

                assert_eq!(voters_3.len(), 8);

                // let's add another 50 votes
                for i in 0..50 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 400 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_1.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                let voters = get_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_1.id(),
                    "quantum",
                    None,
                    true,
                    None,
                    true,
                    platform_version,
                );
                assert_eq!(voters.len(), 100);

                // let's add another vote
                for i in 0..1 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 500 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_1.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                let voters = get_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_1.id(),
                    "quantum",
                    None,
                    true,
                    None,
                    false,
                    platform_version,
                );
                assert_eq!(voters.len(), 100);

                let voters_reversed_30 = get_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_1.id(),
                    "quantum",
                    Some(30),
                    false,
                    Some(get_contested_resource_voters_for_identity_request_v0::StartAtIdentifierInfo {
                        start_identifier: voters.last().expect("expected a last voter").to_vec(),
                        start_identifier_included: true,
                    }),
                    false,
                    platform_version,
                );
                assert_eq!(voters_reversed_30.len(), 30);

                let reversed_last_30_from_100_query: Vec<_> =
                    voters.split_at(70).1.iter().rev().cloned().collect();

                assert_eq!(voters_reversed_30, reversed_last_30_from_100_query);
            }

            #[test]
            fn test_proved_contestant_votes_query_request() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1, contender_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                let (contender_3, _, _) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "quantum",
                    platform_version,
                );

                for i in 0..50 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 10 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_1.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                for i in 0..5 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 100 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_2.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                for i in 0..8 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 200 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_3.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                let voters_1 = get_proved_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_1.id(),
                    "quantum",
                    None,
                    true,
                    None,
                    platform_version,
                );

                assert_eq!(voters_1.len(), 50);

                let voters_2 = get_proved_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_2.id(),
                    "quantum",
                    None,
                    true,
                    None,
                    platform_version,
                );

                assert_eq!(voters_2.len(), 5);

                let voters_3 = get_proved_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_3.id(),
                    "quantum",
                    None,
                    true,
                    None,
                    platform_version,
                );

                let mut voters_3_desc = get_proved_contestant_votes(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    contender_3.id(),
                    "quantum",
                    None,
                    false,
                    None,
                    platform_version,
                );

                voters_3_desc.reverse();

                assert_eq!(voters_3, voters_3_desc);

                assert_eq!(voters_3.len(), 8);
            }
        }

        mod identity_given_votes_query {
            use super::*;
            use drive::query::contested_resource_votes_given_by_identity_query::ContestedResourceVotesGivenByIdentityQuery;

            fn get_identity_given_votes(
                platform: &TempPlatform<MockCoreRPCLike>,
                platform_state: &PlatformState,
                contract: &DataContract,
                identity_id: Identifier,
                count: Option<u32>,
                order_ascending: bool,
                start_at_vote_poll_id_info: Option<
                    get_contested_resource_identity_votes_request_v0::StartAtVotePollIdInfo,
                >,
                should_be_finished: bool,
                platform_version: &PlatformVersion,
            ) -> Vec<ResourceVote> {
                let query_validation_result = platform
                    .query_contested_resource_identity_votes(
                        GetContestedResourceIdentityVotesRequest {
                            version: Some(
                                get_contested_resource_identity_votes_request::Version::V0(
                                    GetContestedResourceIdentityVotesRequestV0 {
                                        identity_id: identity_id.to_vec(),
                                        limit: count,
                                        offset: None,
                                        order_ascending,
                                        start_at_vote_poll_id_info,
                                        prove: false,
                                    },
                                ),
                            ),
                        },
                        platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_contested_resource_identity_votes_response::Version::V0(
                    GetContestedResourceIdentityVotesResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = query_validation_result.version.expect("expected a version");

                let Some(get_contested_resource_identity_votes_response_v0::Result::Votes(
                    get_contested_resource_identity_votes_response_v0::ContestedResourceIdentityVotes {
                        contested_resource_identity_votes, finished_results,
                    },
                )) = result
                else {
                    panic!("expected contenders")
                };
                if should_be_finished {
                    assert!(finished_results);
                }

                contested_resource_identity_votes
                    .into_iter()
                    .map(|vote| {
                        let get_contested_resource_identity_votes_response_v0::ContestedResourceIdentityVote {
                            contract_id, document_type_name, serialized_index_storage_values, vote_choice
                        } = vote;
                        let vote_choice = vote_choice.expect("expected a vote choice");
                        let storage_form = ContestedDocumentResourceVoteStorageForm {
                            contract_id: contract_id.try_into().expect("expected 32 bytes"),
                            document_type_name,
                            index_values: serialized_index_storage_values,
                            resource_vote_choice: (vote_choice.vote_choice_type, vote_choice.identity_id).try_into()?,
                        };
                        Ok(storage_form.resolve_with_contract(contract, platform_version)?)
                    })
                    .collect::<Result<Vec<ResourceVote>, Error>>()
                    .expect("expected all voters to be identifiers")
            }

            fn get_proved_identity_given_votes(
                platform: &TempPlatform<MockCoreRPCLike>,
                platform_state: &PlatformState,
                contract: &DataContract,
                identity_id: Identifier,
                count: Option<u16>,
                order_ascending: bool,
                start_at_vote_poll_id_info: Option<
                    get_contested_resource_identity_votes_request_v0::StartAtVotePollIdInfo,
                >,
                platform_version: &PlatformVersion,
            ) -> Vec<ResourceVote> {
                let query_validation_result = platform
                    .query_contested_resource_identity_votes(
                        GetContestedResourceIdentityVotesRequest {
                            version: Some(
                                get_contested_resource_identity_votes_request::Version::V0(
                                    GetContestedResourceIdentityVotesRequestV0 {
                                        identity_id: identity_id.to_vec(),
                                        limit: count.map(|limit| limit as u32),
                                        offset: None,
                                        order_ascending,
                                        start_at_vote_poll_id_info: start_at_vote_poll_id_info
                                            .clone(),
                                        prove: true,
                                    },
                                ),
                            ),
                        },
                        platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_contested_resource_identity_votes_response::Version::V0(
                    GetContestedResourceIdentityVotesResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = query_validation_result.version.expect("expected a version");

                let Some(get_contested_resource_identity_votes_response_v0::Result::Proof(proof)) =
                    result
                else {
                    panic!("expected contenders")
                };

                let query = ContestedResourceVotesGivenByIdentityQuery {
                    identity_id,
                    offset: None,
                    limit: count,
                    start_at: start_at_vote_poll_id_info.map(|start_at_vote_poll_info| {
                        let included = start_at_vote_poll_info.start_poll_identifier_included;
                        (
                            start_at_vote_poll_info
                                .start_at_poll_identifier
                                .try_into()
                                .expect("expected 32 bytes"),
                            included,
                        )
                    }),
                    order_ascending,
                };

                query
                    .verify_identity_votes_given_proof(
                        proof.grovedb_proof.as_slice(),
                        contract,
                        platform_version,
                    )
                    .expect("expected to verify proof")
                    .1
                    .into_values()
                    .collect()
            }

            #[test]
            fn test_not_proved_identity_given_votes_query_request() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1_quantum, contender_2_quantum, dpns_contract) =
                    create_dpns_name_contest(
                        &mut platform,
                        &platform_state,
                        7,
                        "quantum",
                        platform_version,
                    );

                let (contender_1_cooldog, contender_2_cooldog, dpns_contract) =
                    create_dpns_name_contest(
                        &mut platform,
                        &platform_state,
                        8,
                        "cooldog",
                        platform_version,
                    );

                let (contender_1_superman, contender_2_superman, dpns_contract) =
                    create_dpns_name_contest(
                        &mut platform,
                        &platform_state,
                        9,
                        "superman",
                        platform_version,
                    );

                let (masternode, signer, voting_key) =
                    setup_masternode_identity(&mut platform, 10, platform_version);

                // Now let's perform a few votes

                let platform_state = platform.state.load();

                perform_vote(
                    &mut platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    TowardsIdentity(contender_1_quantum.id()),
                    "quantum",
                    &signer,
                    masternode.id(),
                    &voting_key,
                    1,
                    platform_version,
                );

                let platform_state = platform.state.load();

                perform_vote(
                    &mut platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    TowardsIdentity(contender_2_cooldog.id()),
                    "cooldog",
                    &signer,
                    masternode.id(),
                    &voting_key,
                    2,
                    platform_version,
                );

                let platform_state = platform.state.load();

                perform_vote(
                    &mut platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    ResourceVoteChoice::Lock,
                    "superman",
                    &signer,
                    masternode.id(),
                    &voting_key,
                    3,
                    platform_version,
                );

                let mut votes = get_identity_given_votes(
                    &platform,
                    &platform_state,
                    &dpns_contract,
                    masternode.id(),
                    None,
                    true,
                    None,
                    true,
                    platform_version,
                );

                assert_eq!(votes.len(), 3);

                let vote_0 = votes.remove(0);
                let vote_1 = votes.remove(0);
                let vote_2 = votes.remove(0);

                assert_eq!(
                    vote_0,
                    ResourceVote::V0(ResourceVoteV0 {
                        vote_poll: VotePoll::ContestedDocumentResourceVotePoll(
                            ContestedDocumentResourceVotePoll {
                                contract_id: dpns_contract.id(),
                                document_type_name: "domain".to_string(),
                                index_name: "parentNameAndLabel".to_string(),
                                index_values: vec![
                                    Text("dash".to_string()),
                                    Text("c001d0g".to_string())
                                ]
                            }
                        ),
                        resource_vote_choice: TowardsIdentity(contender_2_cooldog.id())
                    })
                );

                assert_eq!(
                    vote_1,
                    ResourceVote::V0(ResourceVoteV0 {
                        vote_poll: VotePoll::ContestedDocumentResourceVotePoll(
                            ContestedDocumentResourceVotePoll {
                                contract_id: dpns_contract.id(),
                                document_type_name: "domain".to_string(),
                                index_name: "parentNameAndLabel".to_string(),
                                index_values: vec![
                                    Text("dash".to_string()),
                                    Text("superman".to_string())
                                ]
                            }
                        ),
                        resource_vote_choice: Lock
                    })
                );

                assert_eq!(
                    vote_2,
                    ResourceVote::V0(ResourceVoteV0 {
                        vote_poll: VotePoll::ContestedDocumentResourceVotePoll(
                            ContestedDocumentResourceVotePoll {
                                contract_id: dpns_contract.id(),
                                document_type_name: "domain".to_string(),
                                index_name: "parentNameAndLabel".to_string(),
                                index_values: vec![
                                    Text("dash".to_string()),
                                    Text("quantum".to_string())
                                ]
                            }
                        ),
                        resource_vote_choice: TowardsIdentity(contender_1_quantum.id())
                    })
                );
            }

            #[test]
            fn test_proved_identity_given_votes_query_request() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1_quantum, contender_2_quantum, dpns_contract) =
                    create_dpns_name_contest(
                        &mut platform,
                        &platform_state,
                        7,
                        "quantum",
                        platform_version,
                    );

                let (contender_1_cooldog, contender_2_cooldog, dpns_contract) =
                    create_dpns_name_contest(
                        &mut platform,
                        &platform_state,
                        8,
                        "cooldog",
                        platform_version,
                    );

                let (contender_1_superman, contender_2_superman, dpns_contract) =
                    create_dpns_name_contest(
                        &mut platform,
                        &platform_state,
                        9,
                        "superman",
                        platform_version,
                    );

                let (masternode, signer, voting_key) =
                    setup_masternode_identity(&mut platform, 10, platform_version);

                // Now let's perform a few votes

                let platform_state = platform.state.load();

                perform_vote(
                    &mut platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    TowardsIdentity(contender_1_quantum.id()),
                    "quantum",
                    &signer,
                    masternode.id(),
                    &voting_key,
                    1,
                    platform_version,
                );

                let platform_state = platform.state.load();

                perform_vote(
                    &mut platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    TowardsIdentity(contender_2_cooldog.id()),
                    "cooldog",
                    &signer,
                    masternode.id(),
                    &voting_key,
                    2,
                    platform_version,
                );

                let platform_state = platform.state.load();

                perform_vote(
                    &mut platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    ResourceVoteChoice::Lock,
                    "superman",
                    &signer,
                    masternode.id(),
                    &voting_key,
                    3,
                    platform_version,
                );

                let mut votes = get_proved_identity_given_votes(
                    &platform,
                    &platform_state,
                    &dpns_contract,
                    masternode.id(),
                    None,
                    true,
                    None,
                    platform_version,
                );

                assert_eq!(votes.len(), 3);

                let vote_0 = votes.remove(0);
                let vote_1 = votes.remove(0);
                let vote_2 = votes.remove(0);

                assert_eq!(
                    vote_0,
                    ResourceVote::V0(ResourceVoteV0 {
                        vote_poll: VotePoll::ContestedDocumentResourceVotePoll(
                            ContestedDocumentResourceVotePoll {
                                contract_id: dpns_contract.id(),
                                document_type_name: "domain".to_string(),
                                index_name: "parentNameAndLabel".to_string(),
                                index_values: vec![
                                    Text("dash".to_string()),
                                    Text("c001d0g".to_string())
                                ]
                            }
                        ),
                        resource_vote_choice: TowardsIdentity(contender_2_cooldog.id())
                    })
                );

                assert_eq!(
                    vote_1,
                    ResourceVote::V0(ResourceVoteV0 {
                        vote_poll: VotePoll::ContestedDocumentResourceVotePoll(
                            ContestedDocumentResourceVotePoll {
                                contract_id: dpns_contract.id(),
                                document_type_name: "domain".to_string(),
                                index_name: "parentNameAndLabel".to_string(),
                                index_values: vec![
                                    Text("dash".to_string()),
                                    Text("superman".to_string())
                                ]
                            }
                        ),
                        resource_vote_choice: Lock
                    })
                );

                assert_eq!(
                    vote_2,
                    ResourceVote::V0(ResourceVoteV0 {
                        vote_poll: VotePoll::ContestedDocumentResourceVotePoll(
                            ContestedDocumentResourceVotePoll {
                                contract_id: dpns_contract.id(),
                                document_type_name: "domain".to_string(),
                                index_name: "parentNameAndLabel".to_string(),
                                index_values: vec![
                                    Text("dash".to_string()),
                                    Text("quantum".to_string())
                                ]
                            }
                        ),
                        resource_vote_choice: TowardsIdentity(contender_1_quantum.id())
                    })
                );
            }
        }

        mod end_date_query {
            use super::*;
            use dapi_grpc::platform::v0::get_vote_polls_by_end_date_request::get_vote_polls_by_end_date_request_v0;

            #[test]
            fn test_not_proved_end_date_query_request() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                let GetVotePollsByEndDateResponse { version } = platform
                    .query_vote_polls_by_end_date_query(
                        GetVotePollsByEndDateRequest {
                            version: Some(get_vote_polls_by_end_date_request::Version::V0(
                                GetVotePollsByEndDateRequestV0 {
                                    start_time_info: None,
                                    end_time_info: None,
                                    limit: None,
                                    offset: None,
                                    ascending: true,
                                    prove: false,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_vote_polls_by_end_date_response::Version::V0(
                    GetVotePollsByEndDateResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = version.expect("expected a version");

                let Some(get_vote_polls_by_end_date_response_v0::Result::VotePollsByTimestamps(
                    get_vote_polls_by_end_date_response_v0::SerializedVotePollsByTimestamps {
                        vote_polls_by_timestamps,
                        finished_results,
                    },
                )) = result
                else {
                    panic!("expected contenders")
                };

                assert!(finished_results);

                let serialized_contested_vote_poll_bytes = vec![
                    0, 230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123, 91, 126,
                    10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83, 197, 49, 85, 6, 100,
                    111, 109, 97, 105, 110, 18, 112, 97, 114, 101, 110, 116, 78, 97, 109, 101, 65,
                    110, 100, 76, 97, 98, 101, 108, 2, 18, 4, 100, 97, 115, 104, 18, 7, 113, 117,
                    97, 110, 116, 117, 109,
                ];

                // The timestamp is 0 because there were no blocks
                assert_eq!(
                    vote_polls_by_timestamps,
                    vec![SerializedVotePollsByTimestamp {
                        timestamp: 1_209_603_000, // in ms, 2 weeks after Jan 1 1970
                        serialized_vote_polls: vec![serialized_contested_vote_poll_bytes.clone()]
                    }]
                );

                // Let's try deserializing

                let vote_poll = VotePoll::deserialize_from_bytes(
                    serialized_contested_vote_poll_bytes.as_slice(),
                )
                .expect("expected to deserialize");

                assert_eq!(
                    vote_poll,
                    VotePoll::ContestedDocumentResourceVotePoll(
                        ContestedDocumentResourceVotePoll {
                            contract_id: Identifier(IdentifierBytes32([
                                230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123,
                                91, 126, 10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83,
                                197, 49, 85
                            ])),
                            document_type_name: "domain".to_string(),
                            index_name: "parentNameAndLabel".to_string(),
                            index_values: vec![
                                Text("dash".to_string()),
                                Text("quantum".to_string())
                            ]
                        }
                    )
                );
            }

            #[test]
            fn test_proved_end_date_query_request() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                let GetVotePollsByEndDateResponse { version } = platform
                    .query_vote_polls_by_end_date_query(
                        GetVotePollsByEndDateRequest {
                            version: Some(get_vote_polls_by_end_date_request::Version::V0(
                                GetVotePollsByEndDateRequestV0 {
                                    start_time_info: None,
                                    end_time_info: None,
                                    limit: None,
                                    offset: None,
                                    ascending: true,
                                    prove: true,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_vote_polls_by_end_date_response::Version::V0(
                    GetVotePollsByEndDateResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = version.expect("expected a version");

                let Some(get_vote_polls_by_end_date_response_v0::Result::Proof(proof)) = result
                else {
                    panic!("expected contenders")
                };

                let vote_poll_by_end_date_query = VotePollsByEndDateDriveQuery {
                    start_time: None,
                    end_time: None,
                    offset: None,
                    limit: None,
                    order_ascending: true,
                };

                let (_, vote_polls_by_timestamps) = vote_poll_by_end_date_query
                    .verify_vote_polls_by_end_date_proof(
                        proof.grovedb_proof.as_ref(),
                        platform_version,
                    )
                    .expect("expected to verify proof");

                assert_eq!(
                    vote_polls_by_timestamps,
                    BTreeMap::from([(
                        1_209_603_000,
                        vec![VotePoll::ContestedDocumentResourceVotePoll(
                            ContestedDocumentResourceVotePoll {
                                contract_id: Identifier(IdentifierBytes32([
                                    230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222,
                                    123, 91, 126, 10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34,
                                    191, 83, 197, 49, 85
                                ])),
                                document_type_name: "domain".to_string(),
                                index_name: "parentNameAndLabel".to_string(),
                                index_values: vec![
                                    Text("dash".to_string()),
                                    Text("quantum".to_string())
                                ]
                            }
                        )]
                    )])
                );
            }

            #[test]
            fn test_not_proved_end_date_query_multiple_contests() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();
                let mut platform_state = (**platform_state).clone();

                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 500000,
                            height: 100,
                            core_height: 42,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create two new contenders, but we are on the same contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    8,
                    "quantum",
                    platform_version,
                );

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "coolio",
                    platform_version,
                );

                let GetVotePollsByEndDateResponse { version } = platform
                    .query_vote_polls_by_end_date_query(
                        GetVotePollsByEndDateRequest {
                            version: Some(get_vote_polls_by_end_date_request::Version::V0(
                                GetVotePollsByEndDateRequestV0 {
                                    start_time_info: None,
                                    end_time_info: None,
                                    limit: None,
                                    offset: None,
                                    ascending: true,
                                    prove: false,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_vote_polls_by_end_date_response::Version::V0(
                    GetVotePollsByEndDateResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = version.expect("expected a version");

                let Some(get_vote_polls_by_end_date_response_v0::Result::VotePollsByTimestamps(
                    get_vote_polls_by_end_date_response_v0::SerializedVotePollsByTimestamps {
                        vote_polls_by_timestamps,
                        finished_results,
                    },
                )) = result
                else {
                    panic!("expected contenders")
                };

                assert!(finished_results);

                let serialized_contested_vote_poll_bytes_1 = vec![
                    0, 230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123, 91, 126,
                    10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83, 197, 49, 85, 6, 100,
                    111, 109, 97, 105, 110, 18, 112, 97, 114, 101, 110, 116, 78, 97, 109, 101, 65,
                    110, 100, 76, 97, 98, 101, 108, 2, 18, 4, 100, 97, 115, 104, 18, 7, 113, 117,
                    97, 110, 116, 117, 109,
                ];

                let serialized_contested_vote_poll_bytes_2 = vec![
                    0, 230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123, 91, 126,
                    10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83, 197, 49, 85, 6, 100,
                    111, 109, 97, 105, 110, 18, 112, 97, 114, 101, 110, 116, 78, 97, 109, 101, 65,
                    110, 100, 76, 97, 98, 101, 108, 2, 18, 4, 100, 97, 115, 104, 18, 6, 99, 48, 48,
                    49, 49, 48,
                ];

                // The timestamp is 0 because there were no blocks
                assert_eq!(
                    vote_polls_by_timestamps,
                    vec![
                        SerializedVotePollsByTimestamp {
                            timestamp: 1_209_603_000, // in ms, 2 weeks after Jan 1 1970 + 3 seconds (chosen block time in test)
                            serialized_vote_polls: vec![
                                serialized_contested_vote_poll_bytes_1.clone()
                            ]
                        },
                        SerializedVotePollsByTimestamp {
                            timestamp: 1_210_103_000, // in ms, 500 s after Jan 1 1970 + 3 seconds (chosen block time in test)
                            serialized_vote_polls: vec![
                                serialized_contested_vote_poll_bytes_2.clone()
                            ]
                        },
                    ]
                );

                // Let's try deserializing

                let vote_poll_1 = VotePoll::deserialize_from_bytes(
                    serialized_contested_vote_poll_bytes_1.as_slice(),
                )
                .expect("expected to deserialize");

                assert_eq!(
                    vote_poll_1,
                    VotePoll::ContestedDocumentResourceVotePoll(
                        ContestedDocumentResourceVotePoll {
                            contract_id: Identifier(IdentifierBytes32([
                                230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123,
                                91, 126, 10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83,
                                197, 49, 85
                            ])),
                            document_type_name: "domain".to_string(),
                            index_name: "parentNameAndLabel".to_string(),
                            index_values: vec![
                                Text("dash".to_string()),
                                Text("quantum".to_string())
                            ]
                        }
                    )
                );

                // Let's try deserializing

                let vote_poll_2 = VotePoll::deserialize_from_bytes(
                    serialized_contested_vote_poll_bytes_2.as_slice(),
                )
                .expect("expected to deserialize");

                assert_eq!(
                    vote_poll_2,
                    VotePoll::ContestedDocumentResourceVotePoll(
                        ContestedDocumentResourceVotePoll {
                            contract_id: Identifier(IdentifierBytes32([
                                230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123,
                                91, 126, 10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83,
                                197, 49, 85
                            ])),
                            document_type_name: "domain".to_string(),
                            index_name: "parentNameAndLabel".to_string(),
                            index_values: vec![
                                Text("dash".to_string()),
                                Text("c00110".to_string())
                            ]
                        }
                    )
                );
            }

            #[test]
            fn test_proved_end_date_query_multiple_contests() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();
                let mut platform_state = (**platform_state).clone();

                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 500000,
                            height: 100,
                            core_height: 42,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create two new contenders, but we are on the same contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    8,
                    "quantum",
                    platform_version,
                );

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "coolio",
                    platform_version,
                );

                let GetVotePollsByEndDateResponse { version } = platform
                    .query_vote_polls_by_end_date_query(
                        GetVotePollsByEndDateRequest {
                            version: Some(get_vote_polls_by_end_date_request::Version::V0(
                                GetVotePollsByEndDateRequestV0 {
                                    start_time_info: None,
                                    end_time_info: None,
                                    limit: None,
                                    offset: None,
                                    ascending: true,
                                    prove: true,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_vote_polls_by_end_date_response::Version::V0(
                    GetVotePollsByEndDateResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = version.expect("expected a version");

                let Some(get_vote_polls_by_end_date_response_v0::Result::Proof(proof)) = result
                else {
                    panic!("expected contenders")
                };

                let vote_poll_by_end_date_query = VotePollsByEndDateDriveQuery {
                    start_time: None,
                    end_time: None,
                    offset: None,
                    limit: None,
                    order_ascending: true,
                };

                let (_, vote_polls_by_timestamps) = vote_poll_by_end_date_query
                    .verify_vote_polls_by_end_date_proof(
                        proof.grovedb_proof.as_ref(),
                        platform_version,
                    )
                    .expect("expected to verify proof");

                assert_eq!(
                    vote_polls_by_timestamps,
                    BTreeMap::from([
                        (
                            1_209_603_000,
                            vec![VotePoll::ContestedDocumentResourceVotePoll(
                                ContestedDocumentResourceVotePoll {
                                    contract_id: Identifier(IdentifierBytes32([
                                        230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109,
                                        222, 123, 91, 126, 10, 29, 113, 42, 9, 196, 13, 87, 33,
                                        246, 34, 191, 83, 197, 49, 85
                                    ])),
                                    document_type_name: "domain".to_string(),
                                    index_name: "parentNameAndLabel".to_string(),
                                    index_values: vec![
                                        Text("dash".to_string()),
                                        Text("quantum".to_string())
                                    ]
                                }
                            )]
                        ),
                        (
                            1_210_103_000,
                            vec![VotePoll::ContestedDocumentResourceVotePoll(
                                ContestedDocumentResourceVotePoll {
                                    contract_id: Identifier(IdentifierBytes32([
                                        230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109,
                                        222, 123, 91, 126, 10, 29, 113, 42, 9, 196, 13, 87, 33,
                                        246, 34, 191, 83, 197, 49, 85
                                    ])),
                                    document_type_name: "domain".to_string(),
                                    index_name: "parentNameAndLabel".to_string(),
                                    index_values: vec![
                                        Text("dash".to_string()),
                                        Text("c00110".to_string())
                                    ]
                                }
                            )]
                        )
                    ])
                );
            }

            #[test]
            fn test_not_proved_end_date_query_multiple_contests_with_start_at() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();
                let mut platform_state = (**platform_state).clone();

                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 500000,
                            height: 100,
                            core_height: 42,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create two new contenders, but we are on the same contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    8,
                    "quantum",
                    platform_version,
                );

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "coolio",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 900000,
                            height: 150,
                            core_height: 45,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    10,
                    "crazyman",
                    platform_version,
                );

                let GetVotePollsByEndDateResponse { version } = platform
                    .query_vote_polls_by_end_date_query(
                        GetVotePollsByEndDateRequest {
                            version: Some(get_vote_polls_by_end_date_request::Version::V0(
                                GetVotePollsByEndDateRequestV0 {
                                    start_time_info: Some(
                                        get_vote_polls_by_end_date_request_v0::StartAtTimeInfo {
                                            start_time_ms: 1_209_603_000,
                                            start_time_included: false,
                                        },
                                    ),
                                    end_time_info: None,
                                    limit: None,
                                    offset: None,
                                    ascending: true,
                                    prove: false,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_vote_polls_by_end_date_response::Version::V0(
                    GetVotePollsByEndDateResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = version.expect("expected a version");

                let Some(get_vote_polls_by_end_date_response_v0::Result::VotePollsByTimestamps(
                    get_vote_polls_by_end_date_response_v0::SerializedVotePollsByTimestamps {
                        vote_polls_by_timestamps,
                        finished_results,
                    },
                )) = result
                else {
                    panic!("expected contenders")
                };

                assert!(finished_results);

                let serialized_contested_vote_poll_bytes_2 = vec![
                    0, 230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123, 91, 126,
                    10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83, 197, 49, 85, 6, 100,
                    111, 109, 97, 105, 110, 18, 112, 97, 114, 101, 110, 116, 78, 97, 109, 101, 65,
                    110, 100, 76, 97, 98, 101, 108, 2, 18, 4, 100, 97, 115, 104, 18, 6, 99, 48, 48,
                    49, 49, 48,
                ];

                let serialized_contested_vote_poll_bytes_3 = vec![
                    0, 230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123, 91, 126,
                    10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83, 197, 49, 85, 6, 100,
                    111, 109, 97, 105, 110, 18, 112, 97, 114, 101, 110, 116, 78, 97, 109, 101, 65,
                    110, 100, 76, 97, 98, 101, 108, 2, 18, 4, 100, 97, 115, 104, 18, 8, 99, 114,
                    97, 122, 121, 109, 97, 110,
                ];

                assert_eq!(
                    vote_polls_by_timestamps,
                    vec![
                        SerializedVotePollsByTimestamp {
                            timestamp: 1_210_103_000, // in ms, 500 s after Jan 1 1970 + 3 seconds (chosen block time in test)
                            serialized_vote_polls: vec![
                                serialized_contested_vote_poll_bytes_2.clone()
                            ]
                        },
                        SerializedVotePollsByTimestamp {
                            timestamp: 1_210_503_000, // in ms, 900 s after Jan 1 1970 + 3 seconds (chosen block time in test)
                            serialized_vote_polls: vec![
                                serialized_contested_vote_poll_bytes_3.clone()
                            ]
                        },
                    ]
                );
            }

            #[test]
            fn test_not_proved_end_date_query_multiple_contests_with_end_at() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();
                let mut platform_state = (**platform_state).clone();

                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 500000,
                            height: 100,
                            core_height: 42,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create two new contenders, but we are on the same contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    8,
                    "quantum",
                    platform_version,
                );

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "coolio",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 900000,
                            height: 150,
                            core_height: 45,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    10,
                    "crazyman",
                    platform_version,
                );

                let GetVotePollsByEndDateResponse { version } = platform
                    .query_vote_polls_by_end_date_query(
                        GetVotePollsByEndDateRequest {
                            version: Some(get_vote_polls_by_end_date_request::Version::V0(
                                GetVotePollsByEndDateRequestV0 {
                                    start_time_info: Some(
                                        get_vote_polls_by_end_date_request_v0::StartAtTimeInfo {
                                            start_time_ms: 1_209_603_000,
                                            start_time_included: false,
                                        },
                                    ),
                                    end_time_info: Some(
                                        get_vote_polls_by_end_date_request_v0::EndAtTimeInfo {
                                            end_time_ms: 1_210_500_000,
                                            end_time_included: true,
                                        },
                                    ),
                                    limit: None,
                                    offset: None,
                                    ascending: true,
                                    prove: false,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_vote_polls_by_end_date_response::Version::V0(
                    GetVotePollsByEndDateResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = version.expect("expected a version");

                let Some(get_vote_polls_by_end_date_response_v0::Result::VotePollsByTimestamps(
                    get_vote_polls_by_end_date_response_v0::SerializedVotePollsByTimestamps {
                        vote_polls_by_timestamps,
                        finished_results,
                    },
                )) = result
                else {
                    panic!("expected contenders")
                };

                assert!(finished_results);

                let serialized_contested_vote_poll_bytes_2 = vec![
                    0, 230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123, 91, 126,
                    10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83, 197, 49, 85, 6, 100,
                    111, 109, 97, 105, 110, 18, 112, 97, 114, 101, 110, 116, 78, 97, 109, 101, 65,
                    110, 100, 76, 97, 98, 101, 108, 2, 18, 4, 100, 97, 115, 104, 18, 6, 99, 48, 48,
                    49, 49, 48,
                ];

                assert_eq!(
                    vote_polls_by_timestamps,
                    vec![SerializedVotePollsByTimestamp {
                        timestamp: 1_210_103_000, // in ms, 500 s after Jan 1 1970 + 3 seconds (chosen block time in test)
                        serialized_vote_polls: vec![serialized_contested_vote_poll_bytes_2.clone()]
                    },]
                );
            }

            #[test]
            fn test_not_proved_end_date_query_multiple_contests_with_end_at_before_start_at() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();
                let mut platform_state = (**platform_state).clone();

                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 500000,
                            height: 100,
                            core_height: 42,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create two new contenders, but we are on the same contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    8,
                    "quantum",
                    platform_version,
                );

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "coolio",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 900000,
                            height: 150,
                            core_height: 45,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    10,
                    "crazyman",
                    platform_version,
                );

                platform
                    .query_vote_polls_by_end_date_query(
                        GetVotePollsByEndDateRequest {
                            version: Some(get_vote_polls_by_end_date_request::Version::V0(
                                GetVotePollsByEndDateRequestV0 {
                                    start_time_info: Some(
                                        get_vote_polls_by_end_date_request_v0::StartAtTimeInfo {
                                            start_time_ms: 1_209_603_000,
                                            start_time_included: true,
                                        },
                                    ),
                                    end_time_info: Some(
                                        get_vote_polls_by_end_date_request_v0::EndAtTimeInfo {
                                            end_time_ms: 1_209_601_000,
                                            end_time_included: true,
                                        },
                                    ),
                                    limit: None,
                                    offset: None,
                                    ascending: true,
                                    prove: false,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect_err("expected query to be invalid");

                platform
                    .query_vote_polls_by_end_date_query(
                        GetVotePollsByEndDateRequest {
                            version: Some(get_vote_polls_by_end_date_request::Version::V0(
                                GetVotePollsByEndDateRequestV0 {
                                    start_time_info: Some(
                                        get_vote_polls_by_end_date_request_v0::StartAtTimeInfo {
                                            start_time_ms: 1_209_603_000,
                                            start_time_included: true,
                                        },
                                    ),
                                    end_time_info: Some(
                                        get_vote_polls_by_end_date_request_v0::EndAtTimeInfo {
                                            end_time_ms: 1_209_603_000,
                                            end_time_included: false,
                                        },
                                    ),
                                    limit: None,
                                    offset: None,
                                    ascending: true,
                                    prove: false,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect_err("expected query to be invalid");
            }

            #[test]
            fn test_not_proved_end_date_query_multiple_contests_with_start_at_ascending_false() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();
                let mut platform_state = (**platform_state).clone();

                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 500000,
                            height: 100,
                            core_height: 42,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create two new contenders, but we are on the same contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    8,
                    "quantum",
                    platform_version,
                );

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "coolio",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 900000,
                            height: 150,
                            core_height: 45,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    10,
                    "crazyman",
                    platform_version,
                );

                let GetVotePollsByEndDateResponse { version } = platform
                    .query_vote_polls_by_end_date_query(
                        GetVotePollsByEndDateRequest {
                            version: Some(get_vote_polls_by_end_date_request::Version::V0(
                                GetVotePollsByEndDateRequestV0 {
                                    start_time_info: Some(
                                        get_vote_polls_by_end_date_request_v0::StartAtTimeInfo {
                                            start_time_ms: 1_209_603_000,
                                            start_time_included: false,
                                        },
                                    ),
                                    end_time_info: None,
                                    limit: None,
                                    offset: None,
                                    ascending: false,
                                    prove: false,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_vote_polls_by_end_date_response::Version::V0(
                    GetVotePollsByEndDateResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = version.expect("expected a version");

                let Some(get_vote_polls_by_end_date_response_v0::Result::VotePollsByTimestamps(
                    get_vote_polls_by_end_date_response_v0::SerializedVotePollsByTimestamps {
                        vote_polls_by_timestamps,
                        finished_results,
                    },
                )) = result
                else {
                    panic!("expected contenders")
                };

                assert!(finished_results);

                let serialized_contested_vote_poll_bytes_2 = vec![
                    0, 230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123, 91, 126,
                    10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83, 197, 49, 85, 6, 100,
                    111, 109, 97, 105, 110, 18, 112, 97, 114, 101, 110, 116, 78, 97, 109, 101, 65,
                    110, 100, 76, 97, 98, 101, 108, 2, 18, 4, 100, 97, 115, 104, 18, 6, 99, 48, 48,
                    49, 49, 48,
                ];

                let serialized_contested_vote_poll_bytes_3 = vec![
                    0, 230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109, 222, 123, 91, 126,
                    10, 29, 113, 42, 9, 196, 13, 87, 33, 246, 34, 191, 83, 197, 49, 85, 6, 100,
                    111, 109, 97, 105, 110, 18, 112, 97, 114, 101, 110, 116, 78, 97, 109, 101, 65,
                    110, 100, 76, 97, 98, 101, 108, 2, 18, 4, 100, 97, 115, 104, 18, 8, 99, 114,
                    97, 122, 121, 109, 97, 110,
                ];

                assert_eq!(
                    vote_polls_by_timestamps,
                    vec![
                        SerializedVotePollsByTimestamp {
                            timestamp: 1_210_503_000, // in ms, 900 s after Jan 1 1970 + 3 seconds (chosen block time in test)
                            serialized_vote_polls: vec![
                                serialized_contested_vote_poll_bytes_3.clone()
                            ]
                        },
                        SerializedVotePollsByTimestamp {
                            timestamp: 1_210_103_000, // in ms, 500 s after Jan 1 1970 + 3 seconds (chosen block time in test)
                            serialized_vote_polls: vec![
                                serialized_contested_vote_poll_bytes_2.clone()
                            ]
                        },
                    ]
                );
            }

            #[test]
            fn test_proved_end_date_query_multiple_contests_with_start_at() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();
                let mut platform_state = (**platform_state).clone();

                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: BlockInfo {
                            time_ms: 500000,
                            height: 100,
                            core_height: 42,
                            epoch: Default::default(),
                        },
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                // we create two new contenders, but we are on the same contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    8,
                    "quantum",
                    platform_version,
                );

                // we create a new contest
                create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "coolio",
                    platform_version,
                );

                let GetVotePollsByEndDateResponse { version } = platform
                    .query_vote_polls_by_end_date_query(
                        GetVotePollsByEndDateRequest {
                            version: Some(get_vote_polls_by_end_date_request::Version::V0(
                                GetVotePollsByEndDateRequestV0 {
                                    start_time_info: None,
                                    end_time_info: None,
                                    limit: None,
                                    offset: None,
                                    ascending: true,
                                    prove: true,
                                },
                            )),
                        },
                        &platform_state,
                        platform_version,
                    )
                    .expect("expected to execute query")
                    .into_data()
                    .expect("expected query to be valid");

                let get_vote_polls_by_end_date_response::Version::V0(
                    GetVotePollsByEndDateResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = version.expect("expected a version");

                let Some(get_vote_polls_by_end_date_response_v0::Result::Proof(proof)) = result
                else {
                    panic!("expected contenders")
                };

                let vote_poll_by_end_date_query = VotePollsByEndDateDriveQuery {
                    start_time: None,
                    end_time: None,
                    offset: None,
                    limit: None,
                    order_ascending: true,
                };

                let (_, vote_polls_by_timestamps) = vote_poll_by_end_date_query
                    .verify_vote_polls_by_end_date_proof(
                        proof.grovedb_proof.as_ref(),
                        platform_version,
                    )
                    .expect("expected to verify proof");

                assert_eq!(
                    vote_polls_by_timestamps,
                    BTreeMap::from([
                        (
                            1_209_603_000,
                            vec![VotePoll::ContestedDocumentResourceVotePoll(
                                ContestedDocumentResourceVotePoll {
                                    contract_id: Identifier(IdentifierBytes32([
                                        230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109,
                                        222, 123, 91, 126, 10, 29, 113, 42, 9, 196, 13, 87, 33,
                                        246, 34, 191, 83, 197, 49, 85
                                    ])),
                                    document_type_name: "domain".to_string(),
                                    index_name: "parentNameAndLabel".to_string(),
                                    index_values: vec![
                                        Text("dash".to_string()),
                                        Text("quantum".to_string())
                                    ]
                                }
                            )]
                        ),
                        (
                            1_210_103_000,
                            vec![VotePoll::ContestedDocumentResourceVotePoll(
                                ContestedDocumentResourceVotePoll {
                                    contract_id: Identifier(IdentifierBytes32([
                                        230, 104, 198, 89, 175, 102, 174, 225, 231, 44, 24, 109,
                                        222, 123, 91, 126, 10, 29, 113, 42, 9, 196, 13, 87, 33,
                                        246, 34, 191, 83, 197, 49, 85
                                    ])),
                                    document_type_name: "domain".to_string(),
                                    index_name: "parentNameAndLabel".to_string(),
                                    index_values: vec![
                                        Text("dash".to_string()),
                                        Text("c00110".to_string())
                                    ]
                                }
                            )]
                        )
                    ])
                );
            }
        }

        mod prefunded_specialized_balance_query {
            use super::*;

            fn get_specialized_balance(
                platform: &TempPlatform<MockCoreRPCLike>,
                platform_state: &PlatformState,
                dpns_contract: &DataContract,
                name: &str,
                platform_version: &PlatformVersion,
            ) -> Credits {
                let vote_poll = ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
                    contract: DataContractResolvedInfo::BorrowedDataContract(dpns_contract),
                    document_type_name: "domain".to_string(),
                    index_name: "parentNameAndLabel".to_string(),
                    index_values: vec![
                        Value::Text("dash".to_string()),
                        Value::Text(convert_to_homograph_safe_chars(name)),
                    ],
                };

                let specialized_balance_response = platform
                    .query_prefunded_specialized_balance(
                        GetPrefundedSpecializedBalanceRequest {
                            version: Some(get_prefunded_specialized_balance_request::Version::V0(
                                GetPrefundedSpecializedBalanceRequestV0 {
                                    id: vote_poll
                                        .specialized_balance_id()
                                        .expect("expected a specialized balance id")
                                        .to_vec(),
                                    prove: false,
                                },
                            )),
                        },
                        platform_state,
                        platform_version,
                    )
                    .expect("expected to be able to query specialized balance")
                    .into_data()
                    .expect("expected that the query would execute successfully");

                let get_prefunded_specialized_balance_response::Version::V0(
                    GetPrefundedSpecializedBalanceResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = specialized_balance_response
                    .version
                    .expect("expected a version");

                let Some(get_prefunded_specialized_balance_response_v0::Result::Balance(balance)) =
                    result
                else {
                    panic!("expected balance")
                };
                balance
            }

            fn get_proved_specialized_balance(
                platform: &TempPlatform<MockCoreRPCLike>,
                platform_state: &PlatformState,
                dpns_contract: &DataContract,
                name: &str,
                platform_version: &PlatformVersion,
            ) -> Credits {
                let vote_poll = ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
                    contract: DataContractResolvedInfo::BorrowedDataContract(dpns_contract),
                    document_type_name: "domain".to_string(),
                    index_name: "parentNameAndLabel".to_string(),
                    index_values: vec![
                        Value::Text("dash".to_string()),
                        Value::Text(convert_to_homograph_safe_chars(name)),
                    ],
                };

                let balance_id = vote_poll
                    .specialized_balance_id()
                    .expect("expected a specialized balance id");

                let specialized_balance_response = platform
                    .query_prefunded_specialized_balance(
                        GetPrefundedSpecializedBalanceRequest {
                            version: Some(get_prefunded_specialized_balance_request::Version::V0(
                                GetPrefundedSpecializedBalanceRequestV0 {
                                    id: balance_id.to_vec(),
                                    prove: true,
                                },
                            )),
                        },
                        platform_state,
                        platform_version,
                    )
                    .expect("expected to be able to query specialized balance")
                    .into_data()
                    .expect("expected that the query would execute successfully");

                let get_prefunded_specialized_balance_response::Version::V0(
                    GetPrefundedSpecializedBalanceResponseV0 {
                        metadata: _,
                        result,
                    },
                ) = specialized_balance_response
                    .version
                    .expect("expected a version");

                let Some(get_prefunded_specialized_balance_response_v0::Result::Proof(proof)) =
                    result
                else {
                    panic!("expected balance")
                };

                Drive::verify_specialized_balance(
                    proof.grovedb_proof.as_slice(),
                    balance_id.to_buffer(),
                    false,
                    platform_version,
                )
                .expect("expected to verify balance")
                .1
                .expect("expected balance to exist")
            }

            #[test]
            fn test_non_proved_prefunded_specialized_balance_request_after_many_votes() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1, contender_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                let start_balance = get_specialized_balance(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    "quantum",
                    platform_version,
                );

                assert_eq!(start_balance, dash_to_credits!(0.4));

                let (contender_3, contender_4, _) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "quantum",
                    platform_version,
                );

                let start_balance_after_more_contenders = get_specialized_balance(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    "quantum",
                    platform_version,
                );

                assert_eq!(start_balance_after_more_contenders, dash_to_credits!(0.8));

                for i in 0..50 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 10 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_1.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                let balance_after_50_votes = get_specialized_balance(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    "quantum",
                    platform_version,
                );

                assert_eq!(balance_after_50_votes, dash_to_credits!(0.795));

                for i in 0..5 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 100 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_2.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                let balance_after_55_votes = get_specialized_balance(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    "quantum",
                    platform_version,
                );

                assert_eq!(balance_after_55_votes, dash_to_credits!(0.7945));
            }
            #[test]
            fn test_proved_prefunded_specialized_balance_request_after_many_votes() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1, contender_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                let start_balance = get_proved_specialized_balance(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    "quantum",
                    platform_version,
                );

                assert_eq!(start_balance, dash_to_credits!(0.4));

                let (contender_3, contender_4, _) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    9,
                    "quantum",
                    platform_version,
                );

                let start_balance_after_more_contenders = get_proved_specialized_balance(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    "quantum",
                    platform_version,
                );

                assert_eq!(start_balance_after_more_contenders, dash_to_credits!(0.8));

                for i in 0..50 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 10 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_1.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                let balance_after_50_votes = get_proved_specialized_balance(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    "quantum",
                    platform_version,
                );

                assert_eq!(balance_after_50_votes, dash_to_credits!(0.795));

                for i in 0..5 {
                    let (masternode, signer, voting_key) =
                        setup_masternode_identity(&mut platform, 100 + i, platform_version);

                    let platform_state = platform.state.load();

                    perform_vote(
                        &mut platform,
                        &platform_state,
                        dpns_contract.as_ref(),
                        TowardsIdentity(contender_2.id()),
                        "quantum",
                        &signer,
                        masternode.id(),
                        &voting_key,
                        1,
                        platform_version,
                    );
                }

                let balance_after_55_votes = get_proved_specialized_balance(
                    &platform,
                    &platform_state,
                    dpns_contract.as_ref(),
                    "quantum",
                    platform_version,
                );

                assert_eq!(balance_after_55_votes, dash_to_credits!(0.7945));
            }
        }
        mod document_distribution {
            use super::*;
            #[test]
            fn test_document_distribution() {
                let platform_version = PlatformVersion::latest();
                let mut platform = TestPlatformBuilder::new()
                    .build_with_mock_rpc()
                    .set_genesis_state();

                let platform_state = platform.state.load();

                let (contender_1, contender_2, dpns_contract) = create_dpns_name_contest(
                    &mut platform,
                    &platform_state,
                    7,
                    "quantum",
                    platform_version,
                );

                perform_votes_multi(
                    &mut platform,
                    dpns_contract.as_ref(),
                    vec![
                        (TowardsIdentity(contender_1.id()), 50),
                        (TowardsIdentity(contender_2.id()), 5),
                        (ResourceVoteChoice::Abstain, 10),
                        (ResourceVoteChoice::Lock, 3),
                    ],
                    "quantum",
                    10,
                    platform_version,
                );

                let platform_state = platform.state.load();
                let mut platform_state = (**platform_state).clone();

                let block_info = BlockInfo {
                    time_ms: 1_209_900_000, //2 weeks and 300s
                    height: 10000,
                    core_height: 42,
                    epoch: Default::default(),
                };

                platform_state.set_last_committed_block_info(Some(
                    ExtendedBlockInfoV0 {
                        basic_info: block_info,
                        app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                        quorum_hash: [0u8; 32],
                        block_id_hash: [0u8; 32],
                        signature: [0u8; 96],
                        round: 0,
                    }
                    .into(),
                ));

                platform.state.store(Arc::new(platform_state));

                let platform_state = platform.state.load();

                let transaction = platform.drive.grove.start_transaction();

                platform
                    .check_for_ended_vote_polls(&block_info, Some(&transaction), platform_version)
                    .expect("expected to check for ended vote polls");

                platform
                    .drive
                    .grove
                    .commit_transaction(transaction)
                    .unwrap()
                    .expect("expected to commit transaction");

                // At this point the document should have been awarded to contender 1.

                let (contenders, abstaining, locking) = get_vote_states(
                    &platform,
                    &platform_state,
                    &dpns_contract,
                    "quantum",
                    None,
                    true,
                    true,
                    None,
                    ResultType::DocumentsAndVoteTally,
                    platform_version,
                );

                assert_eq!(contenders.len(), 2);

                let first_contender = contenders.first().unwrap();

                let second_contender = contenders.last().unwrap();

                assert_ne!(first_contender.document, second_contender.document);

                assert_eq!(first_contender.identity_id, contender_1.id());

                assert_eq!(second_contender.identity_id, contender_2.id());

                assert_eq!(first_contender.vote_tally, Some(50));

                assert_eq!(second_contender.vote_tally, Some(5));

                assert_eq!(abstaining, Some(10));

                assert_eq!(locking, Some(3));

                // Now let's not include locked and abstaining

                let (contenders, abstaining, locking) = get_vote_states(
                    &platform,
                    &platform_state,
                    &dpns_contract,
                    "quantum",
                    None,
                    true,
                    false,
                    None,
                    ResultType::DocumentsAndVoteTally,
                    platform_version,
                );

                assert_eq!(contenders.len(), 2);

                let first_contender = contenders.first().unwrap();

                let second_contender = contenders.last().unwrap();

                assert_ne!(first_contender.document, second_contender.document);

                assert_eq!(first_contender.identity_id, contender_1.id());

                assert_eq!(second_contender.identity_id, contender_2.id());

                assert_eq!(first_contender.vote_tally, Some(50));

                assert_eq!(second_contender.vote_tally, Some(5));

                assert_eq!(abstaining, None);

                assert_eq!(locking, None);
            }
        }
    }
}
