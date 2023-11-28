//! This crate provides [DAPIClient] --- transport layer for a decentralized API for Dash.

#![deny(missing_docs)]

mod address_list;
mod dapi_client;
#[cfg(feature = "dump")]
pub mod dump;
#[cfg(feature = "mocks")]
pub mod mock;
mod request_settings;
pub mod transport;

pub use dapi_client::RequestExecutor;
use futures::{future::BoxFuture, FutureExt};
pub use http::Uri;

pub use address_list::AddressList;
pub use dapi_client::{DAPIClient, DAPIClientError};
#[cfg(feature = "dump")]
pub use dump::DumpData;
pub use request_settings::RequestSettings;

/// A DAPI request could be executed with an initialized [DAPIClient].
///
/// # Examples
/// ```
/// use rs_dapi_client::{RequestSettings, AddressList, mock::MockDAPIClient, DAPIClientError, DAPIRequest};
/// use dapi_grpc::platform::v0::{self as proto};
///
/// # let _ = async {
/// let mut client = MockDAPIClient::new();
/// let request: proto::GetIdentityRequest = proto::get_identity_request::GetIdentityRequestV0 { id: b"0".to_vec(), prove: true }.into();
/// let response = request.execute(&mut client, RequestSettings::default()).await?;
/// # Ok::<(), DAPIClientError<_>>(())
/// # };
/// ```
pub trait DAPIRequest {
    /// Response from DAPI for this specific request.
    type Response;
    /// An error type for the transport this request uses.
    type TransportError;

    /// Executes the request.
    fn execute<'c, D: RequestExecutor>(
        self,
        dapi_client: &'c D,
        settings: RequestSettings,
    ) -> BoxFuture<'c, Result<Self::Response, DAPIClientError<Self::TransportError>>>
    where
        Self: 'c;
}

/// The trait is intentionally made sealed since it defines what is possible to send to DAPI.
impl<T: transport::TransportRequest + Send> DAPIRequest for T {
    type Response = T::Response;

    type TransportError = <T::Client as transport::TransportClient>::Error;

    fn execute<'c, D: RequestExecutor>(
        self,
        dapi_client: &'c D,
        settings: RequestSettings,
    ) -> BoxFuture<'c, Result<Self::Response, DAPIClientError<Self::TransportError>>>
    where
        Self: 'c,
    {
        dapi_client.execute(self, settings).boxed()
    }
}

/// Allows to flag the transport error variant how tolerant we are of it and whether we can
/// try to do a request again.
pub trait CanRetry {
    /// Get boolean flag that indicates if the error is retryable.
    fn can_retry(&self) -> bool;
}
