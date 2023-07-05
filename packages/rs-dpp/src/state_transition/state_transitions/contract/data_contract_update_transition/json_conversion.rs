use crate::state_transition::data_contract_update_transition::DataContractUpdateTransition;
use serde_json::Value as JsonValue;
use crate::state_transition::state_transitions::data_contract_update_transition::fields::*;
use crate::state_transition::{
    JsonSerializationOptions, StateTransitionJsonConvert, StateTransitionValueConvert,
};
use crate::ProtocolError;
use serde_json::Number;

impl StateTransitionJsonConvert for DataContractUpdateTransition {
    fn to_json(&self, options: JsonSerializationOptions) -> Result<JsonValue, ProtocolError> {
        match self {
            DataContractUpdateTransition::V0(transition) => {
                let mut value = transition.to_json(options)?;
                let map_value = value.as_object_mut().expect("expected an object");
                map_value.insert(
                    STATE_TRANSITION_PROTOCOL_VERSION.to_string(),
                    JsonValue::Number(Number::from(0)),
                )?;
                Ok(value)
            }
        }
    }
}
