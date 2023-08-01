use crate::data_contract::conversion::cbor_conversion::DataContractCborConversionMethodsV0;
use crate::data_contract::conversion::platform_value_conversion::v0::DataContractValueConversionMethodsV0;
use crate::data_contract::data_contract::DataContractV0;
use crate::data_contract::property_names;
use crate::util::cbor_value::CborCanonicalMap;
use crate::util::deserializer;
use crate::util::deserializer::SplitProtocolVersionOutcome;
use crate::version::PlatformVersion;
use crate::ProtocolError;
use ciborium::Value as CborValue;
use platform_value::{Identifier, Value};
use std::convert::TryFrom;

impl DataContractCborConversionMethodsV0 for DataContractV0 {
    // TODO: Do we need to use this?
    fn from_cbor_with_id(
        cbor_bytes: impl AsRef<[u8]>,
        contract_id: Option<Identifier>,
        platform_version: &PlatformVersion,
    ) -> Result<Self, ProtocolError> {
        let mut data_contract = Self::from_cbor(cbor_bytes, platform_version)?;
        if let Some(id) = contract_id {
            data_contract.id = id;
        }
        Ok(data_contract)
    }

    fn from_cbor(
        cbor_bytes: impl AsRef<[u8]>,
        platform_version: &PlatformVersion,
    ) -> Result<Self, ProtocolError> {
        let SplitProtocolVersionOutcome {
            protocol_version,
            protocol_version_size,
            main_message_bytes: contract_cbor_bytes,
        } = deserializer::split_cbor_protocol_version(cbor_bytes.as_ref())?;

        let data_contract_cbor_value: CborValue = ciborium::de::from_reader(contract_cbor_bytes)
            .map_err(|_| {
                ProtocolError::DecodingError(format!(
                    "unable to decode contract with protocol version {} offset {}",
                    protocol_version, protocol_version_size
                ))
            })?;

        let data_contract_value: Value =
            Value::try_from(data_contract_cbor_value).map_err(ProtocolError::ValueError)?;

        Self::from_object(data_contract_value, platform_version)
    }

    fn to_cbor(&self, platform_version: &PlatformVersion) -> Result<Vec<u8>, ProtocolError> {
        let value = self.to_object(platform_version)?;

        let mut buf: Vec<u8> = Vec::new();

        ciborium::ser::into_writer(value.into(), &mut buf)
            .map_err(|e| ProtocolError::EncodingError(e.to_string()))?;

        Ok(buf)
    }

    // /// Returns Data Contract as a Buffer
    // fn to_cbor_buffer(&self) -> Result<Vec<u8>, ProtocolError> {
    //     let mut object = self.to_object()?;
    //     if self.defs.is_none() {
    //         object.remove(property_names::DEFINITIONS)?;
    //     }
    //     object
    //         .to_map_mut()
    //         .unwrap()
    //         .sort_by_lexicographical_byte_ordering_keys_and_inner_maps();
    //
    //     // we are on version 0 here
    //     cbor_serializer::serializable_value_to_cbor(&object, Some(0))
    // }

    // TODO: Revisit
    fn to_cbor_canonical_map(
        &self,
        platform_version: &PlatformVersion,
    ) -> Result<CborCanonicalMap, ProtocolError> {
        let mut contract_cbor_map = CborCanonicalMap::new();

        contract_cbor_map.insert(property_names::ID, self.id.to_buffer().to_vec());
        contract_cbor_map.insert(property_names::SCHEMA, self.schema.as_str());
        contract_cbor_map.insert(property_names::VERSION, self.version);
        contract_cbor_map.insert(property_names::OWNER_ID, self.owner_id.to_buffer().to_vec());

        let docs = CborValue::serialized(&self.documents)
            .map_err(|e| ProtocolError::EncodingError(e.to_string()))?;

        contract_cbor_map.insert(property_names::DOCUMENTS, docs);

        if let Some(_defs) = &self.defs {
            contract_cbor_map.insert(
                property_names::DEFINITIONS,
                CborValue::serialized(&self.defs)
                    .map_err(|e| ProtocolError::EncodingError(e.to_string()))?,
            );
        };

        Ok(contract_cbor_map)
    }
}
