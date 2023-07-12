use crate::serialization_traits::{PlatformDeserializableFromVersionedStructure, PlatformLimitDeserializableFromVersionedStructure, PlatformSerializable, PlatformSerializableIntoStructureVersion, PlatformSerializableWithPlatformVersion};
use bincode::{config, Decode, Encode};
pub use data_contract::*;
use derive_more::From;

use bincode::enc::Encoder;
use bincode::error::EncodeError;
pub use generate_data_contract::*;
use platform_serialization::{
    PlatformDeserialize, PlatformDeserializeNoLimit, PlatformSerialize,
    PlatformVersionedDeserialize,
};
use platform_value::{Identifier, Value};
use serde::Serialize;
use std::collections::BTreeMap;
use std::convert::TryInto;

pub mod errors;
pub mod extra;

mod generate_data_contract;

#[cfg(feature = "state-transitions")]
pub mod created_data_contract;
pub mod document_type;

mod v0;

#[cfg(feature = "client")]
mod factory;
#[cfg(feature = "client")]
pub use factory::*;
mod data_contract_class_methods;
pub use data_contract_class_methods::*;
#[cfg(feature = "client")]
mod data_contract_facade;
mod serialized_version;
mod conversion;
mod data_contract_methods;
pub use data_contract_methods::*;
pub(crate) mod accessors;

#[cfg(feature = "state-transitions")]
pub use created_data_contract::CreatedDataContract;
pub use v0::*;

use crate::consensus::basic::BasicError;
use crate::consensus::ConsensusError;
use crate::data_contract::document_type::DocumentTypeRef;
use crate::data_contract::property_names::SYSTEM_VERSION;
use crate::data_contract::v0::data_contract::DataContractV0;
use crate::util::hash::hash_to_vec;
#[cfg(feature = "validation")]
use crate::validation::SimpleConsensusValidationResult;
use crate::version::{FeatureVersion, PlatformVersion};
use crate::ProtocolError;
use platform_versioning::PlatformSerdeVersionedDeserialize;
use serde_json::Value as JsonValue;
use crate::data_contract::conversion::platform_value_conversion::v0::DataContractValueConversionMethodsV0;
use crate::data_contract::serialized_version::{CONTRACT_DESERIALIZATION_LIMIT, DataContractSerializationFormat};
use crate::ProtocolError::{PlatformDeserializationError, PlatformSerializationError};

pub mod property_names {
    pub const SYSTEM_VERSION: &str = "systemVersion";
    pub const ID: &str = "$id";
    pub const OWNER_ID: &str = "ownerId";
    pub const VERSION: &str = "version";
    pub const SCHEMA: &str = "$schema";
    pub const DOCUMENTS: &str = "documents";
    pub const DEFINITIONS: &str = "$defs";
    pub const ENTROPY: &str = "entropy"; // not a data contract field actually but at some point it can be there for some time
}

pub type JsonSchema = JsonValue;
type DefinitionName = String;
pub type DocumentName = String;
type PropertyPath = String;

pub trait DataContractLike<'a> {
    fn id() -> Identifier;
    fn owner_id() -> Identifier;
    fn contract_version() -> u32;
    fn document_types() -> BTreeMap<DocumentName, DocumentTypeRef<'a>>;
}

/// Understanding Data Contract versioning
/// Data contract versioning is both for the code structure and for serialization.
///
/// The code structure is what is used in code to verify documents and is used in memory
/// There is generally only one code structure running at any given time, except in the case we
/// are switching protocol versions.
///
/// There can be a lot of serialization versions that are active, and serialization versions
/// should generally always be supported. This is because when we store something as version 1.
/// 10 years down the line when we unserialize this contract it will still be in version 1.
/// Deserialization of a data contract serialized in that version should be translated to the
/// current code structure version.
///
/// There are some scenarios to consider,
///
/// One such scenario is that the serialization version does not contain enough information for the
/// current code structure version.
///
/// Depending on the situation one of the following occurs:
/// - the contract structure can imply missing parts based on default behavior
/// - the contract structure can disable certain features dependant on missing information
/// - the contract might be unusable until it is updated by the owner
///

/// Here we use PlatformSerialize, because
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    PlatformSerdeVersionedDeserialize,
    From,
)]
#[serde(untagged)]
pub enum DataContract {
    #[versioned(0)]
    V0(DataContractV0),
}

impl PlatformSerializableWithPlatformVersion for DataContract {
    fn serialize_with_platform_version(&self, platform_version: &PlatformVersion) -> Result<Vec<u8>, ProtocolError> {
        let serialization_format = self.to_serialization_format_based_on_default_current_version(platform_version)?;
        let config = config::standard().with_big_endian().with_no_limit();
        bincode::encode_to_vec(serialization_format, config).map_err(|e| PlatformSerializationError(format!("unable to serialize DataContract: {}", e)))
    }

    fn serialize_consume_with_platform_version(self, platform_version: &PlatformVersion) -> Result<Vec<u8>, ProtocolError> {
        let serialization_format = self.into_serialization_format_based_on_default_current_version(platform_version)?;
        let config = config::standard().with_big_endian().with_no_limit();
        bincode::encode_to_vec(serialization_format, config).map_err(|e| PlatformSerializationError(format!("unable to serialize consume DataContract: {}", e)))
    }
}

impl PlatformDeserializableFromVersionedStructure for DataContract {
    fn versioned_deserialize(data: &[u8], platform_version: &PlatformVersion) -> Result<Self, ProtocolError> where Self: Sized {
        let config = config::standard().with_big_endian().with_no_limit();
        let data_contract_in_serialization_format : DataContractSerializationFormat = bincode::decode_from_slice(data, config).map_err(|e| PlatformDeserializationError(format!("unable to deserialize DataContract: {}", e)))?.0;
        DataContract::from_serialization_format(data_contract_in_serialization_format, platform_version)
    }
}

impl PlatformLimitDeserializableFromVersionedStructure for DataContract {
    fn versioned_limit_deserialize(data: &[u8], platform_version: &PlatformVersion) -> Result<Self, ProtocolError> where Self: Sized {
        let config = config::standard().with_big_endian().with_limit::<CONTRACT_DESERIALIZATION_LIMIT>();
        let data_contract_in_serialization_format : DataContractSerializationFormat = bincode::decode_from_slice(data, config).map_err(|e| PlatformDeserializationError(format!("unable to deserialize DataContract with limit: {}", e)))?.0;
        DataContract::from_serialization_format(data_contract_in_serialization_format, platform_version)
    }
}

impl Default for DataContract {
    fn default() -> Self {
        DataContract::V0(DataContractV0::default())
    }
}

impl DataContract {
    // Returns hash from Data Contract
    pub fn hash(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(hash_to_vec(self.serialize()?))
    }

    pub fn id(&self) -> Identifier {
        match self {
            DataContract::V0(v0) => v0.id,
        }
    }

    pub fn as_v0(&self) -> Option<&DataContractV0> {
        match self {
            DataContract::V0(v0) => Some(v0),
            _ => None,
        }
    }

    pub fn as_v0_mut(&mut self) -> Option<&mut DataContractV0> {
        match self {
            DataContract::V0(v0) => Some(v0),
            _ => None,
        }
    }

    pub fn into_v0(self) -> Option<DataContractV0> {
        match self {
            DataContract::V0(v0) => Some(v0),
        }
    }

    pub fn check_version_is_active(
        protocol_version: u32,
        data_contract_system_version: FeatureVersion,
    ) -> Result<bool, ProtocolError> {
        let platform_version = PlatformVersion::get(protocol_version)?;
        Ok(platform_version
            .contract
            .check_version(data_contract_system_version))
    }

    #[cfg(feature = "platform-value")]
    pub fn from_raw_object(mut raw_object: Value, platform_version: &PlatformVersion) -> Result<DataContract, ProtocolError> {
        let data_contract_system_version =
            match raw_object.remove_optional_integer::<FeatureVersion>(SYSTEM_VERSION) {
                Ok(Some(data_contract_system_version)) => data_contract_system_version,
                Ok(None) => {
                    return Err(ProtocolError::ConsensusError(
                        ConsensusError::BasicError(BasicError::VersionError(
                            "no system version found on data contract object".into(),
                        ))
                        .into(),
                    ));
                }
                Err(e) => {
                    return Err(ProtocolError::ConsensusError(
                        ConsensusError::BasicError(BasicError::VersionError(
                            format!("version error: {}", e.to_string()).into(),
                        ))
                        .into(),
                    ));
                }
            };
        match data_contract_system_version {
            0 => Ok(DataContractV0::from_raw_object(raw_object, platform_version)?.into()),
            _ => Err(ProtocolError::ConsensusError(
                ConsensusError::BasicError(BasicError::VersionError(
                    "system version found on data contract object".into(),
                ))
                .into(),
            )),
        }
    }

    #[cfg(feature = "validation")]
    pub fn validate(
        protocol_version: u32,
        raw_data_contract: &Value,
        allow_non_current_data_contract_versions: bool,
    ) -> Result<SimpleConsensusValidationResult, ProtocolError> {
        let data_contract_system_version =
            match raw_data_contract.get_optional_integer::<FeatureVersion>(SYSTEM_VERSION) {
                Ok(Some(data_contract_system_version)) => data_contract_system_version,
                Ok(None) => {
                    return Ok(SimpleConsensusValidationResult::new_with_error(
                        ConsensusError::BasicError(BasicError::VersionError(
                            "no system version found on data contract object".into(),
                        )),
                    ));
                }
                Err(e) => {
                    return Ok(SimpleConsensusValidationResult::new_with_error(
                        ConsensusError::BasicError(BasicError::VersionError(
                            format!("version error: {}", e.to_string()).into(),
                        )),
                    ));
                }
            };
        if !allow_non_current_data_contract_versions {
            Self::check_version_is_active(protocol_version, data_contract_system_version)?;
        }
        match data_contract_system_version {
            0 => DataContractV0::validate(raw_data_contract),
            _ => Ok(SimpleConsensusValidationResult::new_with_error(
                ConsensusError::BasicError(BasicError::VersionError(
                    "system version found on data contract object".into(),
                )),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data_contract::v0::DataContractV0;
    use crate::data_contract::DataContract;
    use crate::system_data_contracts::load_system_data_contract;
    use crate::version::PlatformVersion;
    use data_contracts::SystemDataContract::Dashpay;
    use serde::Serialize;
    use crate::serialization_traits::PlatformSerializableWithPlatformVersion;

    #[test]
    fn test_contract_serialization() {
        let data_contract = load_system_data_contract(Dashpay).expect("expected dashpay contract");
        let platform_version = PlatformVersion::latest();
        let serialized = data_contract
            .serialize_with_platform_version(platform_version)
            .expect("expected to serialize data contract");
        assert_eq!(
            serialized[0],
            platform_version.contract.default_current_version
        );

        let unserialized = DataContract::deserialize_with_platform_version(platform_version);
        assert_eq!(data_contract, unserialized);
    }
}
