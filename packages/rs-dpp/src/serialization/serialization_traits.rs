use crate::identity::KeyType;
use bincode::config::Config;
use bincode::enc;
use bincode::enc::write::SizeWriter;
use bincode::enc::Encoder;
use bincode::error::EncodeError;
use serde::{Deserialize, Serialize};

use crate::validation::SimpleConsensusValidationResult;
use crate::version::{FeatureVersion, PlatformVersion};
use crate::{BlsModule, ProtocolError};
use platform_value::{platform_value, Value};

pub trait Signable {
    fn signable_bytes(&self) -> Result<Vec<u8>, ProtocolError>;
}

pub trait PlatformSerializable {
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError>;

    /// If the trait is not used just do a simple serialize
    fn serialize_consume(self) -> Result<Vec<u8>, ProtocolError>
    where
        Self: Sized,
    {
        self.serialize()
    }
}

pub trait PlatformSerializableWithPlatformVersion {
    /// Version based serialization is done based on the desired structure version.
    /// For example we have DataContractV0 and DataContractV1 for code based Contracts
    /// This means objects that will execute code
    /// And we would have DataContractSerializationFormatV0 and DataContractSerializationFormatV1
    /// which are the different ways to serialize the concept of a data contract.
    /// The data contract would call versioned_serialize. There should be a converted for each
    /// Data contract Version towards each DataContractSerializationFormat
    fn serialize_with_platform_version(
        &self,
        platform_version: &PlatformVersion,
    ) -> Result<Vec<u8>, ProtocolError>;

    /// If the trait is not used just do a simple serialize
    fn serialize_consume_with_platform_version(
        self,
        platform_version: &PlatformVersion,
    ) -> Result<Vec<u8>, ProtocolError>
    where
        Self: Sized,
    {
        self.serialize_with_platform_version(platform_version)
    }
}

pub trait PlatformSerializableWithPrefixVersion {
    fn serialize_with_prefix_version(
        &self,
        feature_version: FeatureVersion,
    ) -> Result<Vec<u8>, ProtocolError>;

    /// If the trait is not used just do a simple serialize
    fn serialize_consume_with_prefix_version(
        self,
        feature_version: FeatureVersion,
    ) -> Result<Vec<u8>, ProtocolError>
    where
        Self: Sized,
    {
        self.serialize_with_prefix_version(feature_version)
    }
}

pub trait PlatformDeserializable {
    fn deserialize(data: &[u8]) -> Result<Self, ProtocolError>
    where
        Self: Sized,
    {
        Self::deserialize_no_limit(data)
    }

    fn deserialize_no_limit(data: &[u8]) -> Result<Self, ProtocolError>
    where
        Self: Sized;
}

pub trait PlatformDeserializableFromVersionedStructure {
    /// We will deserialize a versioned structure into a code structure
    /// For example we have DataContractV0 and DataContractV1
    /// The system version will tell which version to deserialize into
    /// This happens by first deserializing the data into a potentially versioned structure
    /// For example we could have DataContractSerializationFormatV0 and DataContractSerializationFormatV1
    /// Both of the structures will be valid in perpetuity as they are saved into the state.
    /// So from the bytes we could get DataContractSerializationFormatV0.
    /// Then the system_version given will tell to transform DataContractSerializationFormatV0 into
    /// DataContractV1 (if system version is 1)
    fn versioned_deserialize(
        data: &[u8],
        platform_version: &PlatformVersion,
    ) -> Result<Self, ProtocolError>
    where
        Self: Sized;
}

pub trait PlatformLimitDeserializableFromVersionedStructure {
    fn versioned_limit_deserialize(
        data: &[u8],
        platform_version: &PlatformVersion,
    ) -> Result<Self, ProtocolError>
    where
        Self: Sized;
}

pub trait ValueConvertible<'a>: Serialize + Deserialize<'a> {
    fn to_object(&self) -> Result<Value, ProtocolError>
    where
        Self: Sized + Clone,
    {
        platform_value::to_value(self.clone()).map_err(ProtocolError::ValueError)
    }

    fn into_object(self) -> Result<Value, ProtocolError>
    where
        Self: Sized,
    {
        platform_value::to_value(self).map_err(ProtocolError::ValueError)
    }

    fn from_object(value: Value) -> Result<Self, ProtocolError>
    where
        Self: Sized,
    {
        platform_value::from_value(value).map_err(ProtocolError::ValueError)
    }

    fn from_object_ref(value: &Value) -> Result<Self, ProtocolError>
    where
        Self: Sized,
    {
        platform_value::from_value(value.clone()).map_err(ProtocolError::ValueError)
    }
}

pub trait PlatformMessageSignable {
    fn verify_signature(
        &self,
        public_key_type: KeyType,
        public_key_data: &[u8],
        signature: &[u8],
    ) -> Result<SimpleConsensusValidationResult, ProtocolError>;
    fn sign_by_private_key(
        &self,
        private_key: &[u8],
        key_type: KeyType,
        bls: &impl BlsModule,
    ) -> Result<Vec<u8>, ProtocolError>;
}
