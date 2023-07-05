use serde_json::Value as JsonValue;
use crate::ProtocolError;
use serde::Serialize;

#[derive(Debug, Copy, Clone, Default)]
pub struct JsonSerializationOptions {
    pub skip_signature: bool,
    pub into_validating_json: bool,
}

/// The trait contains methods related to conversion of StateTransition into different formats
pub trait StateTransitionJsonConvert: Serialize {
    /// Returns the [`serde_json::Value`] instance that encodes:
    ///  - Identifiers  - with base58
    ///  - Binary data  - with base64
    fn to_json(&self, options: JsonSerializationOptions) -> Result<JsonValue, ProtocolError> {
        if options.into_validating_json {
            self.to_object(options.skip_signature)?
                .try_into_validating_json()
                .map_err(ProtocolError::ValueError)
        } else {
            self.to_object(options.skip_signature)?
                .try_into()
                .map_err(ProtocolError::ValueError)
        }
    }
}
