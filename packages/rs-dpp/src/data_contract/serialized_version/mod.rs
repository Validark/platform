use crate::data_contract::serialized_version::v0::DataContractSerializationFormatV0;
use bincode::{Decode, Encode};
use crate::data_contract::data_contract::DataContractV0;
use crate::data_contract::DataContract;
use crate::version::{FeatureVersion, PlatformVersion};
use crate::ProtocolError;

mod v0;

pub const CONTRACT_DESERIALIZATION_LIMIT: usize = 15000;

#[derive(Encode, Decode)]
pub enum DataContractSerializationFormat {
    V0(DataContractSerializationFormatV0),
}

impl DataContract {
    pub fn to_serialization_format_based_on_default_current_version(&self, platform_version: &PlatformVersion) -> Result<DataContractSerializationFormat, ProtocolError> {
        match self {
            DataContract::V0(v0) => {
                match platform_version.dpp.contract_serialization_version.data_contract_serialization_version.default_current_version {
                    0 => {
                        let v0_format :DataContractSerializationFormatV0 = v0.clone().into();
                        v0_format.into()
                    }
                    version => Err(ProtocolError::UnknownVersionMismatch {
                        method: "DataContract::serialize_to_default_current_version".to_string(),
                        known_versions: vec![0],
                        received: version,
                    }),
                }
            }
        }
    }

    pub fn into_serialization_format_based_on_default_current_version(self, platform_version: &PlatformVersion) -> Result<DataContractSerializationFormat, ProtocolError> {
        match self {
            DataContract::V0(v0) => {
                match platform_version.dpp.contract_serialization_version.data_contract_serialization_version.default_current_version {
                    0 => {
                        let v0_format :DataContractSerializationFormatV0 = v0.into();
                        v0_format.into()
                    }
                    version => Err(ProtocolError::UnknownVersionMismatch {
                        method: "DataContract::serialize_consume_to_default_current_version".to_string(),
                        known_versions: vec![0],
                        received: version,
                    }),
                }
            }
        }
    }

    pub fn from_serialization_format(serialization_format: DataContractSerializationFormat, platform_version: &PlatformVersion) -> Result<Self, ProtocolError> {
        match serialization_format {
            DataContractSerializationFormat::V0(serialization_format_v0) => {
                match platform_version.dpp.contract_structure_version {
                    0 => {
                        let data_contract = DataContractV0::try_from(serialization_format_v0, platform_version)?;
                        data_contract.into()
                    }
                    version => Err(ProtocolError::UnknownVersionMismatch {
                        method: "DataContract::from_serialization_format".to_string(),
                        known_versions: vec![0],
                        received: version,
                    }),
                }
            }
        }
    }
}