use std::collections::{BTreeMap, HashMap};

use crate::{types::*, ContextProvider, Error};
use dapi_grpc::platform::v0::get_data_contract_request::GetDataContractRequestV0;
use dapi_grpc::platform::v0::get_data_contracts_request::GetDataContractsRequestV0;
use dapi_grpc::platform::v0::get_identity_balance_and_revision_request::GetIdentityBalanceAndRevisionRequestV0;
use dapi_grpc::platform::v0::get_identity_balance_request::GetIdentityBalanceRequestV0;
use dapi_grpc::platform::v0::get_identity_by_public_key_hash_request::GetIdentityByPublicKeyHashRequestV0;
use dapi_grpc::platform::v0::get_identity_keys_request::GetIdentityKeysRequestV0;
use dapi_grpc::platform::v0::security_level_map::KeyKindRequestType as GrpcKeyKind;
use dapi_grpc::platform::v0::{
    get_data_contract_history_request, get_data_contract_request, get_data_contracts_request,
    get_identity_balance_and_revision_request, get_identity_balance_request,
    get_identity_by_public_key_hash_request, get_identity_keys_request, get_identity_request,
    wait_for_state_transition_result_request,
};
use dapi_grpc::platform::{
    v0::{self as platform, key_request_type, KeyRequestType as GrpcKeyType},
    VersionedGrpcResponse,
};
use dpp::document::{Document, DocumentV0Getters};
use dpp::prelude::{DataContract, Identifier, Identity};
use dpp::state_transition::proof_result::StateTransitionProofResult;
use dpp::state_transition::StateTransition;
use dpp::version::{PlatformVersion, TryIntoPlatformVersioned};
use drive::drive::identity::key::fetch::{
    IdentityKeysRequest, KeyKindRequestType, KeyRequestType, PurposeU8, SecurityLevelU8,
};
use drive::drive::verify::state_transition::verify_state_transition_was_executed_with_proof::VerifyKnownContractProviderFn;
pub use drive::drive::verify::RootHash;
use drive::drive::Drive;
use drive::query::DriveQuery;

use crate::verify::verify_tenderdash_proof;

lazy_static::lazy_static! {
    pub static ref PLATFORM_VERSION: PlatformVersion = PlatformVersion::latest().to_owned();
}

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

/// Parse and verify the received proof and retrieve the requested object, if any.
///
/// Use [`FromProofWithKnownContracts::maybe_from_proof_with_known_contracts()`] or
/// [`FromProofWithKnownContracts::from_proof_with_known_contracts()`] to parse and verify proofs received
/// from the Dash Platform. This includes verification of GroveDB-generated proofs and cryptographic proofs
/// generated by Tenderdash.
///
/// gRPC responses, received from the Dash Platform in response to requests containing `prove: true`, contain
/// GroveDB proof structure (including encapsulated objects) and metadata required to verify cryptographic proof
/// generated by Tenderdash. This trait provides methods that parse and verify the proof, taking into account
/// known contracts, and retrieve the requested object (or information that the object does not exist) in one step.
///
/// This trait is implemented for a state transition response defined in the [Dash Platform Protocol](dpp), as [StateTransitionProofResult].
pub trait FromProofStateTransitionResultWithKnownContracts<Req> {
    /// Request type for which this trait is implemented.
    type Request;
    /// Response type for which this trait is implemented.
    type Response;

    /// Retrieve the requested object from the proof, considering known contracts.
    ///
    /// Runs full verification of the proof and retrieves enclosed objects. If the requested object is not found,
    /// an error is thrown. This method uses [`FromProofWithKnownContracts::maybe_from_proof_with_known_contracts()`]
    /// internally.
    ///
    /// # Arguments
    ///
    /// * `request`: The request sent to the server.
    /// * `response`: The response received from the server.
    /// * `known_contract_provider`: A function that provides known contracts by their `Identifier`.
    /// * `platform_version`: The version of the platform used to determine the verification process.
    /// * `provider`: A callback implementing [ContextProvider] that provides quorum details required to verify the proof.
    ///
    /// # Returns
    ///
    /// * `Ok(object)` when the requested object was found in the proof.
    /// * `Err(Error::NotFound)` when the requested object was not found in the proof.
    /// * `Err(Error)` when either the provided data is invalid or proof validation failed.
    fn from_proof_with_known_contracts<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        state_transition: &StateTransition,
        known_contracts_provider_fn: VerifyKnownContractProviderFn,
        platform_version: &PlatformVersion,
        provider: &'a dyn ContextProvider,
    ) -> Result<Self, Error>
    where
        Self: Sized + 'a;
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
            &PLATFORM_VERSION,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(&proof, &mtd, &root_hash, provider)?;

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
            &PLATFORM_VERSION,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(&proof, &mtd, &root_hash, provider)?;

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
            &PLATFORM_VERSION,
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

        verify_tenderdash_proof(&proof, &mtd, &root_hash, provider)?;

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
                                let kt = match GrpcKeyKind::from_i32(*kind) {
                                    Some(GrpcKeyKind::CurrentKeyOfKindRequest) => {
                                        Ok(KeyKindRequestType::CurrentKeyOfKindRequest)
                                    }
                                    Some(GrpcKeyKind::AllKeysOfKindRequest) => {
                                        Ok(KeyKindRequestType::AllKeysOfKindRequest)
                                    }
                                    None => Err(Error::RequestDecodeError {
                                        error: format!("missing requested key type: {}", *kind),
                                    }),
                                };

                                match kt  {
                                    Err(e) => Err(e),
                                    Ok(d) => Ok((*level as u8, d))
                                }
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
            &PLATFORM_VERSION,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(&proof, &mtd, &root_hash, provider)?;

        Ok(maybe_identity)
    }
}

impl FromProof<platform::GetIdentityBalanceAndRevisionRequest> for IdentityBalanceAndRevision {
    type Request = platform::GetIdentityBalanceAndRevisionRequest;
    type Response = platform::GetIdentityBalanceAndRevisionResponse;

    fn maybe_from_proof<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        platform_version: &PlatformVersion,

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

        verify_tenderdash_proof(&proof, &mtd, &root_hash, provider)?;

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
            &PLATFORM_VERSION,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(&proof, &mtd, &root_hash, provider)?;

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
            &PLATFORM_VERSION,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(&proof, &mtd, &root_hash, provider)?;

        let maybe_contracts = if contracts.count_some() > 0 {
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
            &PLATFORM_VERSION,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(&proof, &mtd, &root_hash, provider)?;

        Ok(maybe_history)
    }
}

impl FromProofStateTransitionResultWithKnownContracts<platform::WaitForStateTransitionResultRequest>
    for StateTransitionProofResult
{
    type Request = platform::WaitForStateTransitionResultRequest;
    type Response = platform::WaitForStateTransitionResultResponse;

    fn from_proof_with_known_contracts<'a, I: Into<Self::Request>, O: Into<Self::Response>>(
        request: I,
        response: O,
        state_transition: &StateTransition,
        known_contracts_provider_fn: VerifyKnownContractProviderFn,
        platform_version: &PlatformVersion,

        provider: &'a dyn ContextProvider,
    ) -> Result<Self, Error>
    where
        Self: Sized + 'a,
    {
        let request: Self::Request = request.into();
        let response: Self::Response = response.into();

        // Parse response to read proof and metadata
        let proof = response.proof().or(Err(Error::NoProofInResult))?;

        let mtd = response.metadata().or(Err(Error::EmptyResponseMetadata))?;

        let (root_hash, result) = Drive::verify_state_transition_was_executed_with_proof(
            state_transition,
            &proof.grovedb_proof,
            known_contracts_provider_fn,
            platform_version,
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        Ok(result)
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
            .verify_proof(&proof.grovedb_proof, &PLATFORM_VERSION)
            .map_err(|e| Error::DriveError {
                error: e.to_string(),
            })?;
        let documents = documents
            .into_iter()
            .map(|d| (d.id(), Some(d)))
            .collect::<Documents>();

        verify_tenderdash_proof(&proof, &mtd, &root_hash, provider)?;

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
