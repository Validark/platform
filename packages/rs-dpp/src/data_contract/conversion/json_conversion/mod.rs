mod v0;
pub use v0::*;

use serde_json::Value as JsonValue;
use crate::data_contract::DataContract;
use crate::data_contract::v0::DataContractV0;
use crate::ProtocolError;
use crate::version::{FeatureVersion, PlatformVersion};


impl DataContractJsonConversionMethodsV0 for DataContract {
    fn from_json_object(json_value: JsonValue, platform_version: &PlatformVersion) -> Result<Self, ProtocolError> where Self: Sized {
        match platform_version.dpp.contract_versions.contract_structure_version { 0 => {
            Ok(DataContractV0::from_json_object(json_value, platform_version)?.into())
        }
            version => Err(ProtocolError::UnknownVersionMismatch {
                method: "DataContract::from_json_object".to_string(),
                known_versions: vec![0],
                received: version,
            }),
        }
    }

    fn to_json(&self) -> Result<JsonValue, ProtocolError> {
        match self { DataContract::V0(v0) => {
            v0.to_json()
        } }
    }

    fn to_json_object(&self) -> Result<JsonValue, ProtocolError> {
        match self { DataContract::V0(v0) => {
            v0.to_json_object()
        } }
    }
}