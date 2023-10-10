use dpp::data_contract::DataContractFactory;

pub const PLATFORM_IP: &str = "10.56.229.104";
pub const CORE_PORT: u16 = 30002;
pub const CORE_USER: &str = "PdXjj4HC";
pub const CORE_PASSWORD: &str = "POv4lqSbzO7m";
pub const PLATFORM_PORT: u16 = 2443;

/// Existing identity ID, created as part of platform test suite run
pub const IDENTITY_ID_BYTES: [u8; 32] = [
    65, 63, 57, 243, 204, 9, 106, 71, 187, 2, 94, 221, 190, 127, 141, 114, 137, 209, 243, 50, 60,
    215, 90, 101, 229, 15, 115, 5, 44, 117, 182, 217,
];

// allow unused because we just include() this file, and the functions are used in other files
#[allow(unused)]
fn setup_api() -> rs_sdk::Sdk {
    use rs_dapi_client::AddressList;
    use rs_sdk::core::CoreClient;

    let uri = http::Uri::from_maybe_shared(format!("http://{}:{}", PLATFORM_IP, PLATFORM_PORT))
        .expect("platform address");
    let addresses = AddressList::from(vec![uri]);
    let api = rs_sdk::SdkBuilder::new(addresses)
        .with_core(PLATFORM_IP, CORE_PORT, CORE_USER, CORE_PASSWORD)
        .build()
        .expect("cannot initialize api");

    api
}

// allow unused because we just include() this file, and the functions are used in other files
#[allow(unused)]
fn base64_identifier(base64str: &str) -> dpp::prelude::Identifier {
    let b64 = base64::engine::general_purpose::STANDARD;
    let bytes = base64::Engine::decode(&b64, base64str).expect("base64 decode identifier");
    dpp::prelude::Identifier::from_bytes(bytes.as_ref()).expect("invalid identifier format")
}

// allow unused because we just include() this file, and the functions are used in other files
#[allow(unused)]
fn mock_document_type() -> dpp::data_contract::document_type::DocumentType {
    use dpp::{
        data_contract::document_type::DocumentType,
        platform_value::platform_value,
        prelude::Identifier,
        version::{PlatformVersion, PlatformVersionCurrentVersion},
    };

    let platform_version = PlatformVersion::get_current().unwrap();

    let schema = platform_value!({
        "type": "object",
        "properties": {
            "a": {
                "type": "string",
                "maxLength": 10,
            }
        },
        "additionalProperties": false,
    });

    DocumentType::try_from_schema(
        Identifier::random(),
        "document_type_name",
        schema,
        None,
        false,
        false,
        true,
        platform_version,
    )
    .expect("expected to create a document type")
}

// allow unused because we just include() this file, and the functions are used in other files
#[allow(unused)]
fn mock_data_contract(
    document_type: Option<&dpp::data_contract::document_type::DocumentType>,
) -> dpp::prelude::DataContract {
    use dpp::{
        data_contract::document_type::{accessors::DocumentTypeV0Getters, DocumentType},
        platform_value::{platform_value, Value},
        prelude::{DataContract, Identifier},
        version::{PlatformVersion, PlatformVersionCurrentVersion},
    };
    use std::collections::BTreeMap;

    let platform_version = PlatformVersion::latest();
    let protocol_version = platform_version.protocol_version;

    let owner_id = Identifier::from_bytes(&IDENTITY_ID_BYTES).unwrap();

    // let factory = DataContractFactory::new(protocol_version, None)
    //     .expect("expected to create a factory for get_dpns_data_contract_fixture");
    // let data_contract =factory
    //     .create_with_value_config(owner_id, document_schemas.into(), None, Some(defs))
    //     .expect("data in fixture should be correct");
    let mut document_types: BTreeMap<String, Value> = BTreeMap::new();

    if let Some(doc) = document_type {
        let schema = doc.schema();
        document_types.insert(doc.name().to_string(), schema.clone());
    }

    let data_contract = DataContractFactory::new(protocol_version, None)
        .unwrap()
        .create(owner_id, platform_value!(document_types), None, None)
        .expect("create data contract")
        .data_contract_owned();

    data_contract
}
