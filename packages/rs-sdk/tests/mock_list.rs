use dpp::{
    data_contract::{
        accessors::v0::DataContractV0Getters,
        document_type::{
            accessors::DocumentTypeV0Getters, random_document::CreateRandomDocument, DocumentType,
        },
    },
    document::{Document, DocumentV0Getters},
    platform_value::platform_value,
};
use rs_sdk::{
    platform::{DocumentQuery, List},
    Sdk,
};
include!("common.rs");

#[tokio::test]
async fn test_mock_document_list() {
    let mut sdk = Sdk::new_mock();
    let document_type: DocumentType = mock_document_type();
    let data_contract = mock_data_contract(Some(&document_type));

    let expected = vec![document_type
        .random_document(None, sdk.version())
        .expect("document should be created")];

    let document_id = expected[0].id();
    let document_type_name = document_type.name();

    // [DocumentQuery::new_with_document_id] will fetch the data contract first, so we need to define an expectation for it.
    sdk.mock()
        .expect_fetch(data_contract.id(), Some(data_contract.clone()))
        .await;

    let query = DocumentQuery::new(data_contract, document_type_name)
        .expect("create document query")
        .with_document_id(&document_id);

    sdk.mock()
        .expect_list(query.clone(), Some(expected.clone()))
        .await;

    let retrieved = Document::list(&mut sdk, query)
        .await
        .unwrap()
        .expect("identity should exist");

    assert_eq!(retrieved, expected);
}
