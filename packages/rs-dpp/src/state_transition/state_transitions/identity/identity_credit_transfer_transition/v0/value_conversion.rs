use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

use platform_value::btreemap_extensions::{
    BTreeValueMapHelper, BTreeValueRemoveFromMapHelper, BTreeValueRemoveInnerValueFromMapHelper,
};
use platform_value::{BinaryData, Bytes32, IntegerReplacementType, ReplacementType, Value};
use serde::{Deserialize, Serialize};

use crate::{
    data_contract::DataContract,
    identity::KeyID,
    prelude::Identifier,
    state_transition::{StateTransitionFieldTypes, StateTransitionLike, StateTransitionType},
    Convertible, NonConsensusError, ProtocolError,
};

use crate::serialization::{PlatformDeserializable, Signable};
use crate::state_transition::data_contract_create_transition::DataContractCreateTransitionV0;
use crate::state_transition::identity_credit_transfer_transition::fields::*;
use crate::state_transition::identity_credit_transfer_transition::v0::IdentityCreditTransferTransitionV0;
use crate::state_transition::StateTransitionValueConvert;
use bincode::{config, Decode, Encode};
use platform_version::version::PlatformVersion;

impl<'a> StateTransitionValueConvert<'a> for IdentityCreditTransferTransitionV0 {
    fn from_object(
        mut raw_object: Value,
        platform_version: &PlatformVersion,
    ) -> Result<Self, ProtocolError> {
        platform_value::from_value(raw_object).map_err(ProtocolError::ValueError)
    }

    fn clean_value(value: &mut Value) -> Result<(), ProtocolError> {
        value.replace_at_paths(IDENTIFIER_FIELDS, ReplacementType::Identifier)?;
        value.replace_at_paths(BINARY_FIELDS, ReplacementType::BinaryBytes)?;
        value.replace_integer_type_at_paths(U32_FIELDS, IntegerReplacementType::U32)?;
        Ok(())
    }

    fn from_value_map(
        raw_value_map: BTreeMap<String, Value>,
        platform_version: &PlatformVersion,
    ) -> Result<Self, ProtocolError> {
        let value: Value = raw_value_map.into();
        Self::from_object(value, platform_version)
    }

    fn to_object(&self, skip_signature: bool) -> Result<Value, ProtocolError> {
        let mut value = platform_value::to_value(self)?;
        if skip_signature {
            value
                .remove_values_matching_paths(Self::signature_property_paths())
                .map_err(ProtocolError::ValueError)?;
        }
        Ok(value)
    }

    fn to_cleaned_object(&self, skip_signature: bool) -> Result<Value, ProtocolError> {
        let mut value = platform_value::to_value(self)?;
        if skip_signature {
            value
                .remove_values_matching_paths(Self::signature_property_paths())
                .map_err(ProtocolError::ValueError)?;
        }
        Ok(value)
    }

    // Override to_canonical_cleaned_object to manage add_public_keys individually
    fn to_canonical_cleaned_object(&self, skip_signature: bool) -> Result<Value, ProtocolError> {
        self.to_cleaned_object(skip_signature)
    }
}