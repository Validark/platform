use crate::{types::*, ContextProvider, Error};
use dapi_grpc::platform::v0::get_protocol_version_upgrade_vote_status_request::{
    self, GetProtocolVersionUpgradeVoteStatusRequestV0,
};
use dapi_grpc::platform::v0::security_level_map::KeyKindRequestType as GrpcKeyKind;
use dapi_grpc::platform::v0::{
    get_data_contract_history_request, get_data_contract_request, get_data_contracts_request,
    get_epochs_info_request, get_identity_balance_and_revision_request,
    get_identity_balance_request, get_identity_by_public_key_hash_request,
    get_identity_keys_request, get_identity_request, GetProtocolVersionUpgradeStateRequest,
    GetProtocolVersionUpgradeStateResponse, GetProtocolVersionUpgradeVoteStatusRequest,
    GetProtocolVersionUpgradeVoteStatusResponse,
};
use dapi_grpc::platform::{
    v0::{self as platform, key_request_type, KeyRequestType as GrpcKeyType},
    VersionedGrpcResponse,
};
use dpp::block::epoch::EpochIndex;
use dpp::block::extended_epoch_info::ExtendedEpochInfo;
use dpp::dashcore::hashes::Hash;
use dpp::dashcore::ProTxHash;
use dpp::document::{Document, DocumentV0Getters};
use dpp::prelude::{DataContract, Identifier, Identity};
use dpp::serialization::PlatformDeserializable;
use dpp::state_transition::proof_result::StateTransitionProofResult;
use dpp::state_transition::StateTransition;
use dpp::version::PlatformVersion;
use drive::drive::identity::key::fetch::{
    IdentityKeysRequest, KeyKindRequestType, KeyRequestType, PurposeU8, SecurityLevelU8,
};
pub use drive::drive::verify::RootHash;
use drive::drive::Drive;
use drive::error::proof::ProofError;
use drive::query::DriveQuery;
use std::array::TryFromSliceError;
use std::collections::BTreeMap;
use std::num::TryFromIntError;
use std::sync::Arc;

use crate::verify::verify_tenderdash_proof;

/// Parse and verify the received proof and retrieve the requested object, if any.
///
/// Use [`FromProof::maybe_from_proof()`] or [`FromProof::from_proof()`] to parse and verify proofs received
/// from the Dash Platform (including verification of grovedb-generated proofs and cryptographic proofs geneerated
/// by Tenderdash).
///
/// gRPC responses, received from the Dash Platform in response to requests containing `prove: true`, contain
/// GroveDB proof structure (including encapsulated objects) and metadata required to verify cryptographic proof
/// generated by the Tenderdash. This trait provides methods that parse and verify the proof and retrieve the requested
/// object (or information that the object does not exist) in one step.
///
/// This trait is implemented by several objects defined in [Dash Platform Protocol](dpp), like [Identity],
/// [DataContract], [Documents], etc. It is also implemented by several helper objects from [crate::types] module.
pub trait FromProof<Req> {
    /// Request type for which this trait is implemented.
    type Request;
    /// Response type for which this trait is implemented.
    type Response;
    /// Parse and verify the received proof and retrieve the requested object, if any.
    ///
    /// # Arguments
    ///
    /// * `request`: The request sent to the server.
    /// * `response`: The response received from the server.
    /// * `provider`: A callback implementing [ContextProvider] that provides quorum details required to verify the proof.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(object))` when the requested object was found in the proof.
    /// * `Ok(None)` when the requested object was not found in the proof; this can be interpreted as proof of non-existence.
    /// For collections, returns Ok(None) if none of the requested objects were found.
    /// * `Err(Error)` when either the provided data is invalid or proof validation failed.
    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,
        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized + 'a;

    /// Retrieve the requested object from the proof.
    ///
    /// Runs full verification of the proof and retrieves enclosed objects.
    ///
    /// This method uses [`FromProof::maybe_from_proof()`] internally and throws an error
    /// if the requested object does not exist in the proof.
    ///
    /// # Arguments
    ///
    /// * `request`: The request sent to the server.
    /// * `response`: The response received from the server.
    /// * `provider`: A callback implementing [ContextProvider] that provides quorum details required to verify the proof.
    ///
    /// # Returns
    ///
    /// * `Ok(object)` when the requested object was found in the proof.
    /// * `Err(Error::DocumentMissingInProof)` when the requested object was not found in the proof.
    /// * `Err(Error)` when either the provided data is invalid or proof validation failed.
    fn from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,
        provider: &'a dyn ContextProvider,
    ) -> Result<Self, Error>
    where
        Self: Sized + 'a,
    {
        Self::maybe_from_proof(request, response, platform_version, provider)?
            .ok_or(Error::NotFound)
    }
}

impl FromProof<platform::GetIdentityRequest> for Identity {
    type Request = platform::GetIdentityRequest;
    type Response = platform::GetIdentityResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,
        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        Identity: Sized + 'a,
    {
        let request: platform::GetIdentityRequest = request.into();
        let response: Self::Response = response.into();

        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let id = match request.version.ok_or(Error::EmptyVersion)? {
            get_identity_request::Version::V0(v0) => {
                Identifier::from_bytes(&v0.id).map_err(|e| Error::ProtocolError {
                    error: e.to_string(),
                })?
            }
        };

        // Extract content from proof and verify Drive/GroveDB proofs
        let (root_hash, maybe_identity) = Drive::verify_full_identity_by_identity_id(
            &proof.grovedb_proof,
            false,
            id.into_buffer(),
            platform_version,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        Ok(maybe_identity)
    }
}

// TODO: figure out how to deal with mock::automock
impl FromProof<platform::GetIdentityByPublicKeyHashRequest> for Identity {
    type Request = platform::GetIdentityByPublicKeyHashRequest;
    type Response = platform::GetIdentityByPublicKeyHashResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,

        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        Identity: 'a,
    {
        let request = request.into();
        let response = response.into();
        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let public_key_hash = match request.version.ok_or(Error::EmptyVersion)? {
            get_identity_by_public_key_hash_request::Version::V0(v0) => {
                let public_key_hash: [u8; 20] =
                    v0.public_key_hash
                        .try_into()
                        .map_err(|_| Error::DriveError {
                            error: "Ivalid public key hash length".to_string(),
                        })?;
                public_key_hash
            }
        };

        // Extract content from proof and verify Drive/GroveDB proofs
        let (root_hash, maybe_identity) = Drive::verify_full_identity_by_public_key_hash(
            &proof.grovedb_proof,
            public_key_hash,
            platform_version,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        Ok(maybe_identity)
    }
}

impl FromProof<platform::GetIdentityKeysRequest> for IdentityPublicKeys {
    type Request = platform::GetIdentityKeysRequest;
    type Response = platform::GetIdentityKeysResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,

        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        IdentityPublicKeys: 'a,
    {
        let request: Self::Request = request.into();
        let response: Self::Response = response.into();

        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let (request_type, identity_id, limit, offset) =
            match request.version.ok_or(Error::EmptyVersion)? {
                get_identity_keys_request::Version::V0(v0) => {
                    let request_type = v0.request_type;
                    let identity_id = Identifier::from_bytes(&v0.identity_id)
                        .map_err(|e| Error::ProtocolError {
                            error: e.to_string(),
                        })?
                        .into_buffer();
                    let limit = v0.limit.map(|i| i as u16);
                    let offset = v0.offset.map(|i| i as u16);
                    (request_type, identity_id, limit, offset)
                }
            };

        let key_request = match parse_key_request_type(&request_type)? {
            KeyRequestType::SpecificKeys(specific_keys) => {
                IdentityKeysRequest::new_specific_keys_query(&identity_id, specific_keys)
            }
            KeyRequestType::AllKeys => IdentityKeysRequest::new_all_keys_query(&identity_id, None),
            KeyRequestType::SearchKey(criteria) => IdentityKeysRequest {
                identity_id,
                request_type: KeyRequestType::SearchKey(criteria),
                limit,
                offset,
            },
            KeyRequestType::ContractBoundKey(id, purpose, kind) => IdentityKeysRequest {
                identity_id,
                request_type: KeyRequestType::ContractBoundKey(id, purpose, kind),
                limit,
                offset,
            },
            KeyRequestType::ContractDocumentTypeBoundKey(id, s, purpose, kind) => {
                IdentityKeysRequest {
                    identity_id,
                    request_type: KeyRequestType::ContractDocumentTypeBoundKey(
                        id, s, purpose, kind,
                    ),
                    limit,
                    offset,
                }
            }
        };

        tracing::debug!(?identity_id, "checking proof of identity keys");

        // Extract content from proof and verify Drive/GroveDB proofs
        let (root_hash, maybe_identity) = Drive::verify_identity_keys_by_identity_id(
            &proof.grovedb_proof,
            key_request,
            false,
            platform_version,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        let maybe_keys: Option<IdentityPublicKeys> = if let Some(identity) = maybe_identity {
            if identity.loaded_public_keys.is_empty() {
                None
            } else {
                let mut keys = identity
                    .loaded_public_keys
                    .into_iter()
                    .map(|(k, v)| (k, Some(v.clone())))
                    .collect::<IdentityPublicKeys>();

                let mut not_found = identity
                    .not_found_public_keys
                    .into_iter()
                    .map(|k| (k, None))
                    .collect::<IdentityPublicKeys>();

                keys.append(&mut not_found);

                Some(keys)
            }
        } else {
            None
        };

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        Ok(maybe_keys)
    }
}

fn parse_key_request_type(request: &Option<GrpcKeyType>) -> Result<KeyRequestType, Error> {
    let key_request_type = request
        .to_owned()
        .ok_or(Error::RequestDecodeError {
            error: "missing key request type".to_string(),
        })?
        .request
        .ok_or(Error::RequestDecodeError {
            error: "empty request field in key request type".to_string(),
        })?;

    let request_type = match key_request_type {
        key_request_type::Request::AllKeys(_) => KeyRequestType::AllKeys,
        key_request_type::Request::SpecificKeys(specific_keys) => {
            KeyRequestType::SpecificKeys(specific_keys.key_ids)
        }
        key_request_type::Request::SearchKey(search_key) => {
            let purpose = search_key
                .purpose_map
                .iter()
                .map(|(k, v)| {
                    let v=  v.security_level_map
                            .iter()
                            .map(|(level, kind)| {
                                let kt = match GrpcKeyKind::try_from(*kind).map_err(|e| Error::RequestDecodeError {
                                    error: format!("invalid key kind: {}", e),
                                })? {
                                    GrpcKeyKind::CurrentKeyOfKindRequest => {
                                        KeyKindRequestType::CurrentKeyOfKindRequest
                                    }
                                    GrpcKeyKind::AllKeysOfKindRequest => {
                                        KeyKindRequestType::AllKeysOfKindRequest
                                    }
                                };
                                
                                Ok((*level as u8, kt))
                            })
                            .collect::<Result<BTreeMap<SecurityLevelU8,KeyKindRequestType>,Error>>();

                            match v {
                                Err(e) =>Err(e),
                                Ok(d) => Ok(((*k as u8),d)),
                            }
                })
                .collect::<Result<BTreeMap<PurposeU8, BTreeMap<SecurityLevelU8, KeyKindRequestType>>,Error>>()?;

            KeyRequestType::SearchKey(purpose)
        }
    };

    Ok(request_type)
}

impl FromProof<platform::GetIdentityBalanceRequest> for IdentityBalance {
    type Request = platform::GetIdentityBalanceRequest;
    type Response = platform::GetIdentityBalanceResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,

        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        IdentityBalance: 'a,
    {
        let request: Self::Request = request.into();
        let response: Self::Response = response.into();

        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let id = match request.version.ok_or(Error::EmptyVersion)? {
            get_identity_balance_request::Version::V0(v0) => Identifier::from_bytes(&v0.id)
                .map_err(|e| Error::ProtocolError {
                    error: e.to_string(),
                }),
        }?;

        // Extract content from proof and verify Drive/GroveDB proofs
        let (root_hash, maybe_identity) = Drive::verify_identity_balance_for_identity_id(
            &proof.grovedb_proof,
            id.into_buffer(),
            false,
            platform_version,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        Ok(maybe_identity)
    }
}

impl FromProof<platform::GetIdentityBalanceAndRevisionRequest> for IdentityBalanceAndRevision {
    type Request = platform::GetIdentityBalanceAndRevisionRequest;
    type Response = platform::GetIdentityBalanceAndRevisionResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        _platform_version: &PlatformVersion,

        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        IdentityBalanceAndRevision: 'a,
    {
        let request: Self::Request = request.into();
        let response: Self::Response = response.into();

        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let id = match request.version.ok_or(Error::EmptyVersion)? {
            get_identity_balance_and_revision_request::Version::V0(v0) => {
                Identifier::from_bytes(&v0.id).map_err(|e| Error::ProtocolError {
                    error: e.to_string(),
                })
            }
        }?;

        // Extract content from proof and verify Drive/GroveDB proofs
        let (root_hash, maybe_identity) =
            Drive::verify_identity_balance_and_revision_for_identity_id(
                &proof.grovedb_proof,
                id.into_buffer(),
                false,
            )
            .map_err(|e| Error::DriveError {
                error: e.to_string(),
            })?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        Ok(maybe_identity)
    }
}

impl FromProof<platform::GetDataContractRequest> for DataContract {
    type Request = platform::GetDataContractRequest;
    type Response = platform::GetDataContractResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,

        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        DataContract: 'a,
    {
        let request: Self::Request = request.into();
        let response: Self::Response = response.into();

        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let id = match request.version.ok_or(Error::EmptyVersion)? {
            get_data_contract_request::Version::V0(v0) => {
                Identifier::from_bytes(&v0.id).map_err(|e| Error::ProtocolError {
                    error: e.to_string(),
                })
            }
        }?;

        // Extract content from proof and verify Drive/GroveDB proofs
        let (root_hash, maybe_contract) = Drive::verify_contract(
            &proof.grovedb_proof,
            None,
            false,
            false,
            id.into_buffer(),
            platform_version,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        Ok(maybe_contract)
    }
}

impl FromProof<platform::GetDataContractsRequest> for DataContracts {
    type Request = platform::GetDataContractsRequest;
    type Response = platform::GetDataContractsResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,

        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        DataContracts: 'a,
    {
        let request: Self::Request = request.into();
        let response: Self::Response = response.into();

        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let ids = match request.version.ok_or(Error::EmptyVersion)? {
            get_data_contracts_request::Version::V0(v0) => v0.ids,
        };

        let ids = ids
            .iter()
            .map(|id| {
                id.clone()
                    .try_into()
                    .map_err(|_e| Error::RequestDecodeError {
                        error: format!("wrong id size: expected: {}, got: {}", 32, id.len()),
                    })
            })
            .collect::<Result<Vec<[u8; 32]>, Error>>()?;

        // Extract content from proof and verify Drive/GroveDB proofs
        let (root_hash, contracts) = Drive::verify_contracts(
            &proof.grovedb_proof,
            false,
            ids.as_slice(),
            platform_version,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        let maybe_contracts: Option<BTreeMap<Identifier, Option<DataContract>>> =
            if !contracts.is_empty() {
                let contracts: DataContracts = contracts
                    .into_iter()
                    .try_fold(DataContracts::new(), |mut acc, (k, v)| {
                        Identifier::from_bytes(&k).map(|id| {
                            acc.insert(id, v);
                            acc
                        })
                    })
                    .map_err(|e| Error::ResultEncodingError {
                        error: e.to_string(),
                    })?;

                Some(contracts)
            } else {
                None
            };

        Ok(maybe_contracts)
    }
}

impl FromProof<platform::GetDataContractHistoryRequest> for DataContractHistory {
    type Request = platform::GetDataContractHistoryRequest;
    type Response = platform::GetDataContractHistoryResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,

        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized + 'a,
    {
        let request: Self::Request = request.into();
        let response: Self::Response = response.into();

        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let (id, limit, offset, start_at_ms) = match request.version.ok_or(Error::EmptyVersion)? {
            get_data_contract_history_request::Version::V0(v0) => {
                let id = Identifier::from_bytes(&v0.id).map_err(|e| Error::ProtocolError {
                    error: e.to_string(),
                })?;
                let limit = u32_to_u16_opt(v0.limit.unwrap_or_default())?;
                let offset = u32_to_u16_opt(v0.offset.unwrap_or_default())?;
                let start_at_ms = v0.start_at_ms;
                (id, limit, offset, start_at_ms)
            }
        };

        // Extract content from proof and verify Drive/GroveDB proofs
        let (root_hash, maybe_history) = Drive::verify_contract_history(
            &proof.grovedb_proof,
            id.into_buffer(),
            start_at_ms,
            limit,
            offset,
            platform_version,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        Ok(maybe_history)
    }
}

impl FromProof<platform::BroadcastStateTransitionRequest> for StateTransitionProofResult {
    type Request = platform::BroadcastStateTransitionRequest;
    type Response = platform::WaitForStateTransitionResultResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,
        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized + 'a,
    {
        let request: Self::Request = request.into();
        let response: Self::Response = response.into();

        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let state_transition = StateTransition::deserialize_from_bytes(&request.state_transition)
            .map_err(|e| Error::ProtocolError {
            error: e.to_string(),
        })?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let known_contracts_provider_fn =
            |id: &Identifier| -> Result<Option<Arc<DataContract>>, drive::error::Error> {
                provider.get_data_contract(id).map_err(|e| {
                    drive::error::Error::Proof(ProofError::ErrorRetrievingContract(e.to_string()))
                })
            };

        let (root_hash, result) = Drive::verify_state_transition_was_executed_with_proof(
            &state_transition,
            &proof.grovedb_proof,
            &known_contracts_provider_fn,
            platform_version,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        Ok(Some(result))
    }
}

impl FromProof<platform::GetEpochsInfoRequest> for ExtendedEpochInfo {
    type Request = platform::GetEpochsInfoRequest;
    type Response = platform::GetEpochsInfoResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,
        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized + 'a,
    {
        let epochs =
            ExtendedEpochInfos::maybe_from_proof(request, response, platform_version, provider)?;

        if let Some(mut e) = epochs {
            if e.len() != 1 {
                return Err(Error::RequestDecodeError {
                    error: format!("expected 1 epoch, got {}", e.len()),
                });
            }
            let epoch = e.pop_first().and_then(|v| v.1);
            Ok(epoch)
        } else {
            Ok(None)
        }
    }
}

impl FromProof<platform::GetEpochsInfoRequest> for ExtendedEpochInfos {
    type Request = platform::GetEpochsInfoRequest;
    type Response = platform::GetEpochsInfoResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,
        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized + 'a,
    {
        let request: Self::Request = request.into();
        let response: Self::Response = response.into();
        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let (start_epoch, count, ascending) = match request.version.ok_or(Error::EmptyVersion)? {
            get_epochs_info_request::Version::V0(v0) => (v0.start_epoch, v0.count, v0.ascending),
        };

        let current_epoch: EpochIndex = try_u32_to_u16(mtd.epoch)?;
        let start_epoch: Option<EpochIndex> = if let Some(epoch) = start_epoch {
            Some(try_u32_to_u16(epoch)?)
        } else {
            None
        };
        let count = try_u32_to_u16(count)?;

        let (root_hash, epoch_info) = Drive::verify_epoch_infos(
            &proof.grovedb_proof,
            current_epoch,
            start_epoch,
            count,
            ascending,
            platform_version,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        let epoch_info = epoch_info
            .into_iter()
            .map(|v| {
                #[allow(clippy::infallible_destructuring_match)]
                let info = match &v {
                    ExtendedEpochInfo::V0(i) => i,
                };

                (info.index, Some(v))
            })
            .collect::<BTreeMap<EpochIndex, Option<ExtendedEpochInfo>>>();

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        let maybe_epoch_info = if epoch_info.count_some() > 0 {
            Some(epoch_info)
        } else {
            None
        };

        Ok(maybe_epoch_info)
    }
}

fn try_u32_to_u16(i: u32) -> Result<u16, Error> {
    i.try_into()
        .map_err(|e: TryFromIntError| Error::RequestDecodeError {
            error: e.to_string(),
        })
}

impl FromProof<GetProtocolVersionUpgradeStateRequest> for ProtocolVersionUpgrades {
    type Request = GetProtocolVersionUpgradeStateRequest;
    type Response = GetProtocolVersionUpgradeStateResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        _request: I,
        response: O,
        platform_version: &PlatformVersion,
        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized + 'a,
    {
        let response: Self::Response = response.into();
        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;
        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let (root_hash, object) =
            Drive::verify_upgrade_state(&proof.grovedb_proof, platform_version)?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        if object.is_empty() {
            return Ok(None);
        }

        Ok(Some(
            object.into_iter().map(|(k, v)| (k, Some(v))).collect(),
        ))
    }
}

impl FromProof<GetProtocolVersionUpgradeVoteStatusRequest> for MasternodeProtocolVotes {
    type Request = GetProtocolVersionUpgradeVoteStatusRequest;
    type Response = GetProtocolVersionUpgradeVoteStatusResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,
        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized + 'a,
    {
        let request = request.into();
        let response: Self::Response = response.into();
        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;
        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let request_v0: GetProtocolVersionUpgradeVoteStatusRequestV0 = match request.version {
            Some(get_protocol_version_upgrade_vote_status_request::Version::V0(v0)) => v0,
            None => return Err(Error::EmptyVersion),
        };

        let start_pro_tx_hash: Option<[u8; 32]> =
            if request_v0.start_pro_tx_hash.is_empty() {
                None
            } else {
                Some(request_v0.start_pro_tx_hash[..].try_into().map_err(
                    |e: TryFromSliceError| Error::RequestDecodeError {
                        error: e.to_string(),
                    },
                )?)
            };

        let (root_hash, objects) = Drive::verify_upgrade_vote_status(
            &proof.grovedb_proof,
            start_pro_tx_hash,
            try_u32_to_u16(request_v0.count)?,
            platform_version,
        )?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        if objects.is_empty() {
            return Ok(None);
        }
        let votes: MasternodeProtocolVotes = objects
            .into_iter()
            .map(|(key, value)| {
                ProTxHash::from_slice(&key)
                    .map(|protxhash| {
                        (
                            protxhash,
                            Some(MasternodeProtocolVote {
                                pro_tx_hash: protxhash,
                                voted_version: value,
                            }),
                        )
                    })
                    .map_err(|e| Error::ResultEncodingError {
                        error: e.to_string(),
                    })
            })
            .collect::<Result<MasternodeProtocolVotes, Error>>()?;

        Ok(Some(votes))
    }
}
// #[cfg_attr(feature = "mocks", mockall::automock)]
impl<'dq, Q> FromProof<Q> for Documents
where
    Q: TryInto<DriveQuery<'dq>> + Clone + 'dq,
    Q::Error: std::fmt::Display,
{
    type Request = Q;
    type Response = platform::GetDocumentsResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,

        provider: &'a dyn ContextProvider,
    ) -> Result<Option<Self>, Error>
    where
        Self: 'a,
    {
        let request: Self::Request = request.into();
        let response: Self::Response = response.into();

        let request: DriveQuery<'dq> =
            request
                .clone()
                .try_into()
                .map_err(|e: Q::Error| Error::RequestDecodeError {
                    error: e.to_string(),
                })?;

        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let (root_hash, documents) = request
            .verify_proof(&proof.grovedb_proof, platform_version)
            .map_err(|e| Error::DriveError {
                error: e.to_string(),
            })?;
        let documents = documents
            .into_iter()
            .map(|d| (d.id(), Some(d)))
            .collect::<Documents>();

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        if documents.is_empty() {
            Ok(None)
        } else {
            Ok(Some(documents))
        }
    }
}

/// Convert u32, if 0 return None, otherwise return Some(u16).
/// Errors when value is out of range.
fn u32_to_u16_opt(i: u32) -> Result<Option<u16>, Error> {
    let i: Option<u16> = if i != 0 {
        let i: u16 =
            i.try_into()
                .map_err(|e: std::num::TryFromIntError| Error::RequestDecodeError {
                    error: format!("value {} out of range: {}", i, e),
                })?;
        Some(i)
    } else {
        None
    };

    Ok(i)
}

/// Determine number of non-None elements
pub trait Length {
    /// Return number of non-None elements in the data structure
    fn count_some(&self) -> usize;
}

impl<T: Length> Length for Option<T> {
    fn count_some(&self) -> usize {
        match self {
            None => 0,
            Some(i) => i.count_some(),
        }
    }
}

impl<T> Length for Vec<Option<T>> {
    fn count_some(&self) -> usize {
        self.iter().filter(|v| v.is_some()).count()
    }
}

impl<K, T> Length for Vec<(K, Option<T>)> {
    fn count_some(&self) -> usize {
        self.iter().filter(|(_, v)| v.is_some()).count()
    }
}

impl<K, T> Length for BTreeMap<K, Option<T>> {
    fn count_some(&self) -> usize {
        self.iter().filter(|(_, v)| v.is_some()).count()
    }
}

/// Implement Length trait for a type
///
/// # Arguments
///
/// * `$object`: The type for which to implement Length trait
/// * `$len`: A closure that returns the length of the object; if ommitted, defaults to 1
macro_rules! define_length {
    ($object:ty,$len:expr) => {
        impl Length for $object {
            fn count_some(&self) -> usize {
                #[allow(clippy::redundant_closure_call)]
                $len(self)
            }
        }
    };
    ($object:ty) => {
        define_length!($object, |_| 1);
    };
}

define_length!(DataContract);
define_length!(DataContractHistory, |d: &DataContractHistory| d.len());
// define_length!(DataContracts, |d: &DataContracts| d.count_some());
define_length!(Document);
// define_length!(Documents, |x: &Documents| x.len());
define_length!(Identity);
define_length!(IdentityBalance);
define_length!(IdentityBalanceAndRevision);
// define_length!(IdentityPublicKeys, |d: &IdentityPublicKeys| d.count_some());
