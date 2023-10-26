//! Mocking mechanisms for Dash Platform SDK.
//!
//! See [MockDashPlatformSdk] for more details.
use dapi_grpc::platform::v0::{self as proto};
use dpp::version::PlatformVersion;
use drive_proof_verifier::{FromProof, MockQuorumInfoProvider};
use rs_dapi_client::{
    mock::{Key, MockDapiClient},
    transport::TransportRequest,
    DapiClient, DumpData,
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    platform::{identity::IdentityRequest, DocumentQuery, Fetch, FetchMany, Query},
    Error,
};

use super::{MockRequest, MockResponse};

/// Mechanisms to mock Dash Platform SDK.
///
/// This object is returned by [Sdk::mock()](crate::Sdk::mock()) and is used to define mock expectations.
///
/// Use [MockDashPlatformSdk::expect_fetch_many()] to define expectations for [FetchMany] requests
/// and [MockDashPlatformSdk::expect_fetch()] for [Fetch] requests.
///
/// ## Panics
///
/// Can panic on errors.
pub struct MockDashPlatformSdk {
    from_proof_expectations: BTreeMap<Key, Vec<u8>>,
    platform_version: &'static PlatformVersion,
    dapi: Arc<Mutex<MockDapiClient>>,
    prove: bool,
    quorum_provider: Option<MockQuorumInfoProvider>,
}

impl MockDashPlatformSdk {
    pub(crate) fn new(
        version: &'static PlatformVersion,
        dapi: Arc<Mutex<MockDapiClient>>,
        prove: bool,
    ) -> Self {
        Self {
            from_proof_expectations: Default::default(),
            platform_version: version,
            dapi,
            prove,
            quorum_provider: None,
        }
    }

    pub(crate) fn version<'v>(&self) -> &'v PlatformVersion {
        self.platform_version
    }
    /// Define a directory where files containing quorum information, like quorum public keys, are stored.
    ///
    /// This directory will be used to load quorum information from files.
    /// You can use [SdkBuilder::with_dump_dir()](crate::SdkBuilder::with_dump_dir()) to generate these files.
    pub fn quorum_info_dir<P: AsRef<std::path::Path>>(&mut self, dir: P) -> &mut Self {
        let mut provider = MockQuorumInfoProvider::new();
        provider.quorum_keys_dir(Some(dir.as_ref().to_path_buf()));
        self.quorum_provider = Some(provider);

        self
    }

    /// Load all expectations from files in a directory.
    ///
    /// Expectation files must be prefixed with [DapiClient::DUMP_FILE_PREFIX] and
    /// have `.json` extension.
    pub async fn load_expectations<P: AsRef<std::path::Path>>(
        &mut self,
        dir: P,
    ) -> Result<&mut Self, Error> {
        let prefix = DapiClient::DUMP_FILE_PREFIX;

        let entries = dir.as_ref().read_dir().map_err(|e| {
            Error::Config(format!(
                "cannot load mock expectations from {}: {}",
                dir.as_ref().display(),
                e
            ))
        })?;

        let files: Vec<PathBuf> = entries
            .into_iter()
            .filter_map(|x| x.ok())
            .filter(|f| {
                f.file_type().is_ok_and(|t| t.is_file())
                    && f.file_name().to_string_lossy().starts_with(prefix)
                    && f.file_name().to_string_lossy().ends_with(".json")
            })
            .map(|f| f.path())
            .collect();

        for filename in &files {
            let basename = filename.file_name().unwrap().to_str().unwrap();
            let request_type = basename.split('_').nth(2).unwrap_or_default();

            match request_type {
                "GetDataContractRequest" => {
                    self.load_expectation::<proto::GetDataContractRequest>(filename)
                        .await?
                }
                "IdentityRequest" => self.load_expectation::<IdentityRequest>(filename).await?,
                "GetIdentityBalanceRequest" => {
                    self.load_expectation::<proto::GetIdentityBalanceRequest>(filename)
                        .await?
                }
                "DocumentQuery" => self.load_expectation::<DocumentQuery>(filename).await?,
                _ => {
                    return Err(Error::Config(format!(
                        "unknown request type {} in {}",
                        request_type,
                        filename.display()
                    )))
                }
            };
        }

        Ok(self)
    }

    async fn load_expectation<T: TransportRequest + for<'de> Deserialize<'de> + MockRequest>(
        &mut self,
        path: &PathBuf,
    ) -> Result<(), Error> {
        let data = DumpData::<T>::load(path).map_err(|e| {
            Error::Config(format!(
                "cannot load mock expectations from {}: {}",
                path.display(),
                e
            ))
        })?;

        self.dapi.lock().await.expect(&data.request, &data.response);
        Ok(())
    }

    /// Expect a [Fetch] request and return provided object.
    ///
    /// This method is used to define mock expectations for [Fetch] requests.
    ///
    /// ## Generic Parameters
    ///
    /// - `O`: Type of the object that will be returned in response to the query. Must implement [Fetch] and [MockResponse].
    /// - `Q`: Type of the query that will be sent to the platform. Must implement [Query] and [MockRequest].
    ///
    /// ## Arguments
    ///
    /// - `query`: Query that will be sent to the platform.
    /// - `object`: Object that will be returned in response to `query`, or None if the object is expected to not exist.
    ///
    /// ## Returns
    ///
    /// * Some(O): If the object is expected to exist.
    /// * None: If the object is expected to not exist.
    ///
    /// ## Panics
    ///
    /// Can panic on errors.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # let r = tokio::runtime::Runtime::new().unwrap();
    /// #
    /// # r.block_on(async {
    ///     use rs_sdk::{Sdk, platform::{Identity, Fetch, dpp::identity::accessors::IdentityGettersV0}};
    ///
    ///     let mut api = Sdk::new_mock();
    ///     // Define expected response
    ///     let expected: Identity = Identity::random_identity(1, None, api.version())
    ///         .expect("create expected identity");
    ///     // Define query that will be sent
    ///     let query = expected.id();
    ///     // Expect that in response to `query`, `expected` will be returned
    ///     api.mock().expect_fetch(query, Some(expected.clone())).await;
    ///
    ///     // Fetch the identity
    ///     let retrieved = dpp::prelude::Identity::fetch(&mut api, query)
    ///         .await
    ///         .unwrap()
    ///         .expect("object should exist");
    ///
    ///     // Check that the identity is the same as expected
    ///     assert_eq!(retrieved, expected);
    /// # });
    /// ```
    pub async fn expect_fetch<
        O: Fetch + MockResponse,
        Q: Query<<O as Fetch>::Request> + MockRequest,
    >(
        &mut self,
        query: Q,
        object: Option<O>,
    ) -> &mut Self
    where
        <O as Fetch>::Request: MockRequest,
        <<O as Fetch>::Request as TransportRequest>::Response: Default,
    {
        let grpc_request = query.query(self.prove).expect("query must be correct");
        self.expect(grpc_request, object).await;

        self
    }

    /// Expect a [FetchMany] request and return provided object.
    ///
    /// This method is used to define mock expectations for [FetchMany] requests.
    ///
    /// ## Generic Parameters
    ///
    /// - `O`: Type of the object that will be returned in response to the query.
    /// Must implement [FetchMany]. `Vec<O>` must implement [MockResponse].
    /// - `Q`: Type of the query that will be sent to the platform. Must implement [Query] and [MockRequest].
    ///
    /// ## Arguments
    ///
    /// - `query`: Query that will be sent to the platform.
    /// - `objects`: Vector of objects that will be returned in response to `query`, or None if no objects are expected.
    ///
    /// ## Returns
    ///
    /// * `Some(Vec<O>)`: If the objects are expected to exist.
    /// * `None`: If the objects are expected to not exist.
    ///
    /// ## Panics
    ///
    /// Can panic on errors.
    ///
    /// ## Example
    ///
    /// Usage example is similar to
    /// [MockDashPlatformSdk::expect_fetch()], but the expected
    /// object must be a vector of objects.
    pub async fn expect_fetch_many<
        O: FetchMany,
        Q: Query<<O as FetchMany>::Request> + MockRequest,
    >(
        &mut self,
        query: Q,
        objects: Option<Vec<O>>,
    ) -> &mut Self
    where
        Vec<O>: MockResponse,
        <O as FetchMany>::Request: MockRequest,
        <<O as FetchMany>::Request as TransportRequest>::Response: Default,
        Vec<O>: FromProof<
                <O as FetchMany>::Request,
                Request = <O as FetchMany>::Request,
                Response = <<O as FetchMany>::Request as TransportRequest>::Response,
            > + Sync,
    {
        let grpc_request = query.query(self.prove).expect("query must be correct");
        self.expect(grpc_request, objects).await;

        self
    }

    /// Save expectations for a request.
    async fn expect<I: TransportRequest + MockRequest, O: MockResponse>(
        &mut self,
        grpc_request: I,
        returned_object: Option<O>,
    ) where
        I::Response: Default,
    {
        let key = grpc_request.mock_key();

        // This expectation will work for from_proof
        self.from_proof_expectations
            .insert(key, returned_object.mock_serialize(self));

        // This expectation will work for execute
        let mut dapi_guard = self.dapi.lock().await;
        // We don't really care about the response, as it will be mocked by from_proof
        dapi_guard.expect(&grpc_request, &Default::default());
    }

    /// Wrapper around [FromProof] that uses mock expectations instead of executing [FromProof] trait.
    pub(crate) fn parse_proof<I, O: FromProof<I>>(
        &self,
        request: O::Request,
        response: O::Response,
    ) -> Result<Option<O>, drive_proof_verifier::Error>
    where
        O::Request: MockRequest,
        Option<O>: MockResponse,
        // O: FromProof<<O as FromProof<I>>::Request>,
    {
        let data = match self.from_proof_expectations.get(&request.mock_key()) {
            Some(d) => Option::<O>::mock_deserialize(self, d),
            None => {
                let provider = self.quorum_provider.as_ref()
                    .ok_or(drive_proof_verifier::Error::InvalidQuorum{
                        error:"expectation not found and quorum info provider not initialized with sdk.mock().quorum_info_dir()".to_string()
                    })?;
                O::maybe_from_proof(request, response, provider)?
            }
        };

        Ok(data)
    }
}
