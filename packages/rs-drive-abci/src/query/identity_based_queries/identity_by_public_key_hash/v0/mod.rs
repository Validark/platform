use crate::error::query::QueryError;
use crate::error::Error;
use crate::platform_types::platform::Platform;
use crate::platform_types::platform_state::v0::PlatformStateV0Methods;
use crate::platform_types::platform_state::PlatformState;
use crate::query::QueryValidationResult;
use dapi_grpc::platform::v0::get_identity_by_public_key_hash_request::GetIdentityByPublicKeyHashRequestV0;
use dapi_grpc::platform::v0::get_identity_by_public_key_hash_response::GetIdentityByPublicKeyHashResponseV0;
use dapi_grpc::platform::v0::{
    get_identity_by_public_key_hash_response, GetIdentityByPublicKeyHashResponse, Proof,
};
use dapi_grpc::Message;
use dpp::check_validation_result_with_data;
use dpp::platform_value::Bytes20;
use dpp::serialization::PlatformSerializable;
use dpp::validation::ValidationResult;
use dpp::version::PlatformVersion;

impl<C> Platform<C> {
    pub(super) fn query_identity_by_public_key_hash_v0(
        &self,
        GetIdentityByPublicKeyHashRequestV0 {
            public_key_hash,
            prove,
        }: GetIdentityByPublicKeyHashRequestV0,
        platform_version: &PlatformVersion,
    ) -> Result<QueryValidationResult<GetIdentityByPublicKeyHashResponse>, Error> {
        let public_key_hash =
            check_validation_result_with_data!(Bytes20::from_vec(public_key_hash)
                .map(|bytes| bytes.0)
                .map_err(|_| QueryError::InvalidArgument(
                    "public key hash must be 20 bytes long".to_string()
                )));

        let response = if prove {
            let proof = self.drive.prove_full_identity_by_unique_public_key_hash(
                public_key_hash,
                None,
                platform_version,
            )?;

            let (metadata, proof) = self.response_metadata_and_proof_v0(proof);

            GetIdentityByPublicKeyHashResponse {
                version: Some(get_identity_by_public_key_hash_response::Version::V0(GetIdentityByPublicKeyHashResponseV0 {
                    result: Some(get_identity_by_public_key_hash_response::get_identity_by_public_key_hash_response_v0::Result::Proof(proof)),
                    metadata: Some(metadata),
                })),
            }
        } else {
            let maybe_identity = self.drive.fetch_full_identity_by_unique_public_key_hash(
                public_key_hash,
                None,
                platform_version,
            )?;

            let identity = check_validation_result_with_data!(maybe_identity.ok_or_else(|| {
                QueryError::NotFound(format!(
                    "identity for public key hash {} not found",
                    hex::encode(public_key_hash)
                ))
            }));

            let serialized_identity = identity
                .serialize_consume_to_bytes()
                .map_err(Error::Protocol)?;

            GetIdentityByPublicKeyHashResponse {
                version: Some(get_identity_by_public_key_hash_response::Version::V0(GetIdentityByPublicKeyHashResponseV0 {
                    metadata: Some(self.response_metadata_v0()),
                    result: Some(get_identity_by_public_key_hash_response::get_identity_by_public_key_hash_response_v0::Result::Identity(serialized_identity)),
                })),
            }
        };

        Ok(QueryValidationResult::new_with_data(response))
    }
}
