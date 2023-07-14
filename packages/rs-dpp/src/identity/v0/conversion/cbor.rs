use crate::identity::conversion::cbor::IdentityCborConversionMethodsV0;
use crate::identity::v0::IdentityV0;
use crate::identity::{IdentityPublicKey, KeyID};
use crate::util::cbor_value::{CborBTreeMapHelper, CborCanonicalMap};
use crate::util::deserializer;
use crate::util::deserializer::SplitProtocolVersionOutcome;
use crate::ProtocolError;
use ciborium::Value as CborValue;
use integer_encoding::VarInt;
use std::collections::BTreeMap;

impl IdentityCborConversionMethodsV0 for IdentityV0 {
    /// Converts the identity to a cbor buffer
    fn to_cbor(&self) -> Result<Vec<u8>, ProtocolError> {
        // Prepend protocol version to the result
        let mut buf = self.get_feature_version().encode_var_vec();

        let mut identity_map = CborCanonicalMap::new();

        identity_map.insert("id", self.get_id().to_buffer().to_vec());
        identity_map.insert("balance", self.get_balance());
        identity_map.insert("revision", self.get_revision());
        identity_map.insert(
            "publicKeys",
            self.get_public_keys()
                .iter()
                .map(|(_, pk)| pk.into())
                .collect::<Vec<CborValue>>(),
        );

        let mut identity_cbor = identity_map
            .to_bytes()
            .map_err(|e| ProtocolError::EncodingError(e.to_string()))?;
        buf.append(&mut identity_cbor);

        Ok(buf)
    }

    fn from_cbor(identity_cbor: &[u8]) -> Result<Self, ProtocolError> {
        let SplitProtocolVersionOutcome {
            protocol_version,
            main_message_bytes: identity_cbor_bytes,
            ..
        } = deserializer::split_cbor_protocol_version(identity_cbor)?;

        // Deserialize the contract
        let identity_map: BTreeMap<String, CborValue> =
            ciborium::de::from_reader(identity_cbor_bytes).map_err(|e| {
                ProtocolError::DecodingError(format!("Unable to decode identity CBOR: {}", e))
            })?;

        let identity_id = identity_map.get_identifier("id")?;
        let revision = identity_map.get_integer("revision")?;
        let balance: u64 = identity_map.get_integer("balance")?;

        let keys_cbor_value = identity_map.get("publicKeys").ok_or_else(|| {
            ProtocolError::DecodingError(String::from(
                "Unable to decode identity CBOR: missing property publicKeys",
            ))
        })?;
        let keys_cbor = keys_cbor_value.as_array().ok_or_else(|| {
            ProtocolError::DecodingError(String::from(
                "Unable to decode identity CBOR: invalid public keys",
            ))
        })?;

        let public_keys = keys_cbor
            .iter()
            .map(|k| IdentityPublicKey::from_cbor_value(k).map(|d| (d.id, d)))
            .collect::<Result<BTreeMap<KeyID, IdentityPublicKey>, ProtocolError>>()?;

        Ok(Self {
            feature_version: protocol_version as u16,
            id: identity_id.into(),
            public_keys,
            balance,
            revision,
            asset_lock_proof: None,
            metadata: None,
        })
    }
}
