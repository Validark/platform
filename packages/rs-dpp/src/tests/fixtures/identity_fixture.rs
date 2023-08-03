use crate::identity::IdentityV0;
use platform_value::platform_value;
use platform_value::string_encoding::Encoding;
use platform_value::BinaryData;
use serde_json::json;

#[cfg(feature = "identity-value-conversion")]
use crate::identity::conversion::platform_value::IdentityPlatformValueConversionMethodsV0;
use crate::prelude::{Identifier, Identity};
use crate::version::PlatformVersion;
use crate::ProtocolError;

//3bufpwQjL5qsvuP4fmCKgXJrKG852DDMYfi9J6XKqPAT
//[198, 23, 40, 120, 58, 93, 0, 165, 27, 49, 4, 117, 107, 204,  67, 46, 164, 216, 230, 135, 201, 92, 31, 155, 62, 131, 211, 177, 139, 175, 163, 237]

pub fn identity_fixture_raw_object() -> platform_value::Value {
    platform_value!({
        "protocolVersion": 1u32,
        "id": Identifier::from([198, 23, 40, 120, 58, 93, 0, 165, 27, 49, 4, 117, 107, 204,  67, 46, 164, 216, 230, 135, 201, 92, 31, 155, 62, 131, 211, 177, 139, 175, 163, 237]),
        "publicKeys": [
            {
                "id": 0u32,
                "type": 0u8,
                "purpose": 0u8,
                "securityLevel": 0u8,
                "data": BinaryData::from_string("AuryIuMtRrl/VviQuyLD1l4nmxi9ogPzC9LT7tdpo0di", Encoding::Base64).unwrap(),
                "readOnly": false
            },
            {
                "id": 1u32,
                "type": 0u8,
                "purpose": 1u8,
                "securityLevel": 3u8,
                "data": BinaryData::from_string("A8AK95PYMVX5VQKzOhcVQRCUbc9pyg3RiL7jttEMDU+L", Encoding::Base64).unwrap(),
                "readOnly": false
            }
        ],
        "balance": 10u64,
        "revision": 0u64
    })
}

pub fn identity_fixture_json() -> serde_json::Value {
    json!({
        "protocolVersion": 1,
        "id": "3bufpwQjL5qsvuP4fmCKgXJrKG852DDMYfi9J6XKqPAT",
        "publicKeys": [
            {
                "id": 0,
                "type": 0,
                "purpose": 0,
                "securityLevel": 0,
                "data": "AuryIuMtRrl/VviQuyLD1l4nmxi9ogPzC9LT7tdpo0di",
                "readOnly": false
            },
            {
                "id": 1,
                "type": 0,
                "purpose": 1,
                "securityLevel": 3,
                "data": "A8AK95PYMVX5VQKzOhcVQRCUbc9pyg3RiL7jttEMDU+L",
                "readOnly": false
            }
        ],
        "balance": 10,
        "revision": 0
    })
}

#[cfg(feature = "identity-value-conversion")]
pub fn identity_fixture(protocol_version: u32) -> Result<Identity, ProtocolError> {
    let raw_object = identity_fixture_raw_object();
    let platform_version = PlatformVersion::get(protocol_version)?;
    match platform_version
        .dpp
        .identity_versions
        .identity_structure_version
    {
        0 => Ok(IdentityV0::from_object(raw_object, platform_version)?.into()),
        version => Err(ProtocolError::UnknownVersionMismatch {
            method: "identity_fixture".to_string(),
            known_versions: vec![0],
            received: version,
        }),
    }
}
