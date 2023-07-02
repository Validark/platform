use std::collections::BTreeMap;
use platform_value::btreemap_extensions::BTreeValueRemoveFromMapHelper;
use platform_value::{IntegerReplacementType, ReplacementType, Value};
use crate::data_contract::DataContract;
use crate::ProtocolError;
use crate::state_transition::abstract_state_transition::StateTransitionValueConvert;
use crate::state_transition::data_contract_update_transition::{BINARY_FIELDS, DataContractUpdateTransitionV0, IDENTIFIER_FIELDS, U32_FIELDS};
use crate::state_transition::data_contract_update_transition::property_names::{DATA_CONTRACT, SIGNATURE, SIGNATURE_PUBLIC_KEY_ID};
use crate::state_transition::StateTransitionConvert;

impl StateTransitionValueConvert for DataContractUpdateTransitionV0 {
    fn to_object(&self, skip_signature: bool) -> Result<Value, ProtocolError> {
        let mut object: Value = platform_value::to_value(self)?;
        if skip_signature {
            Self::signature_property_paths()
                .into_iter()
                .try_for_each(|path| {
                    object
                        .remove_values_matching_path(path)
                        .map_err(ProtocolError::ValueError)
                        .map(|_| ())
                })?;
        }
        object.insert(String::from(DATA_CONTRACT), self.data_contract.to_object()?)?;
        Ok(object)
    }

    fn to_cleaned_object(&self, skip_signature: bool) -> Result<Value, ProtocolError> {
        let mut object: Value = platform_value::to_value(self)?;
        if skip_signature {
            Self::signature_property_paths()
                .into_iter()
                .try_for_each(|path| {
                    object
                        .remove_values_matching_path(path)
                        .map_err(ProtocolError::ValueError)
                        .map(|_| ())
                })?;
        }
        object.insert(
            String::from(DATA_CONTRACT),
            self.data_contract.to_cleaned_object()?,
        )?;
        Ok(object)
    }

    fn from_raw_object(
        mut raw_object: Value,
    ) -> Result<Self, ProtocolError> {
        Ok(DataContractUpdateTransitionV0 {
            signature: raw_object
                .remove_optional_binary_data(SIGNATURE)
                .map_err(ProtocolError::ValueError)?
                .unwrap_or_default(),
            signature_public_key_id: raw_object
                .get_optional_integer(SIGNATURE_PUBLIC_KEY_ID)
                .map_err(ProtocolError::ValueError)?
                .unwrap_or_default(),
            data_contract: DataContract::from_raw_object(
                raw_object.remove(DATA_CONTRACT).map_err(|_| {
                    ProtocolError::DecodingError(
                        "data contract missing on state transition".to_string(),
                    )
                })?,
            )?,
            ..Default::default()
        })
    }

    fn from_value_map(
        mut raw_data_contract_update_transition: BTreeMap<String, Value>,
    ) -> Result<Self, ProtocolError> {
        Ok(DataContractUpdateTransitionV0 {
            signature: raw_data_contract_update_transition
                .remove_optional_binary_data(SIGNATURE)
                .map_err(ProtocolError::ValueError)?
                .unwrap_or_default(),
            signature_public_key_id: raw_data_contract_update_transition
                .remove_optional_integer(SIGNATURE_PUBLIC_KEY_ID)
                .map_err(ProtocolError::ValueError)?
                .unwrap_or_default(),
            data_contract: DataContract::from_raw_object(
                raw_data_contract_update_transition
                    .remove(DATA_CONTRACT)
                    .ok_or(ProtocolError::DecodingError(
                        "data contract missing on state transition".to_string(),
                    ))?,
            )?,
            ..Default::default()
        })
    }

    fn clean_value(value: &mut Value) -> Result<(), ProtocolError> {
        value.replace_at_paths(IDENTIFIER_FIELDS, ReplacementType::Identifier)?;
        value.replace_at_paths(BINARY_FIELDS, ReplacementType::BinaryBytes)?;
        value.replace_integer_type_at_paths(U32_FIELDS, IntegerReplacementType::U32)?;
        Ok(())
    }
}