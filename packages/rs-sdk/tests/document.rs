//! Test document CRUDL operations

use std::fmt::Debug;

use drive_proof_verifier::proof::from_proof::Length;
use rs_sdk::crud::{Readable, SdkQuery};
use rs_sdk::platform::document::SdkDocument;
use rs_sdk::platform::document_query::SdkDocumentQuery;

include!("common.rs");

async fn test_read<API: Sdk, O: Readable<API>, Q: SdkQuery<O::Identifier>>(
    api: &API,
    id: &Q,
    expected: Result<usize, rs_sdk::error::Error>,
) -> Result<O, rs_sdk::error::Error>
where
    O: Length + Debug + Clone,
{
    let result: Result<O, rs_sdk::error::Error> = O::read(api, id).await;

    match expected {
        Ok(count) => {
            if let Ok(ref o) = result {
                assert_eq!(count, o.count_some());
            } else {
                panic!("Expected Ok, got error")
            }
        }
        Err(e) => {
            if let Err(ref e2) = result {
                assert_eq!(e.to_string(), e2.to_string());
            } else {
                panic!("Expected error, got Ok")
            }
        }
    }

    result
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn document_read() {
    setup_logs();

    const DATA_CONTRACT_ID: &str = "U+2i5Ec8omVduoMbFf+e8sPx3QaW0B5pwQolLU+N3dw=";
    const DOCUMENT_TYPE_NAME: &str = "indexedDocument";
    const DOCUMENT_ID: &str = "0DDWWXXPtcooBgJJJTCBDZ4xxinWg5yMPbIf/iv98d4=";

    let api = setup_api();

    let data_contract_id = base64_identifier(DATA_CONTRACT_ID);
    let document_id = base64_identifier(DOCUMENT_ID);

    let query = SdkDocumentQuery::new_with_document_id(
        &api,
        data_contract_id,
        DOCUMENT_TYPE_NAME,
        document_id,
    )
    .await
    .expect("create SdkDocumentQuery");

    let _res: Result<SdkDocument, rs_sdk::error::Error> = test_read(&api, &query, Ok(1)).await;
}

pub fn setup_logs() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            "info,rs_sdk=trace,h2=info",
        ))
        .pretty()
        .with_ansi(true)
        .try_init()
        .ok();
}
