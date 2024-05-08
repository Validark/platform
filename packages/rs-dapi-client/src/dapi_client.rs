//! [DapiClient] definition.

use backon::{ExponentialBuilder, Retryable};
use dapi_grpc::mock::Mockable;
use dapi_grpc::tonic::{async_trait, Status};
use http::Uri;
use std::error::Error;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::Instrument;

use crate::address_list::AddressListError;
use crate::connection_pool::ConnectionPool;
use crate::request_settings::AppliedRequestSettings;
use crate::{
    transport::{TransportClient, TransportRequest},
    Address, AddressList, CanRetry, RequestSettings,
};

/// General DAPI request error type.
#[derive(Debug, thiserror::Error)]
pub enum DapiClientError<TE> {
    /// The error happened on transport layer
    #[error("transport error with {1:?}: {0}")]
    Transport(TE, Uri),
    /// There are no valid DAPI addresses to use.
    #[error("no available addresses to use")]
    NoAvailableAddresses,
    /// [AddressListError] errors
    #[error("address list error: {0}")]
    AddressList(AddressListError),

    #[cfg(feature = "mocks")]
    #[error("mock error: {0}")]
    /// Error happened in mock client
    Mock(#[from] crate::mock::MockError),
}

impl<TE: CanRetry> CanRetry for DapiClientError<TE> {
    fn is_node_failure(&self) -> bool {
        use DapiClientError::*;
        match self {
            NoAvailableAddresses => false,
            Transport(transport_error, _) => transport_error.is_node_failure(),
            AddressList(_) => false,
            #[cfg(feature = "mocks")]
            Mock(_) => false,
        }
    }
}

pub trait ToUri {
    fn to_uri(&self) -> Uri;
}

impl ToUri for Status {
    fn to_uri(&self) -> Uri {
        println!(
            "status error: {:?} {:?} {:?}",
            self.metadata(),
            self.source(),
            String::from_utf8(self.details().to_vec()).unwrap_or("INVALID UTF-8".to_string())
        );
        Uri::from_static("http://127.0.0.11")
    }
}

#[async_trait]
/// DAPI client executor trait.
pub trait DapiRequestExecutor {
    /// Execute request using this DAPI client.
    async fn execute<R>(
        &self,
        request: R,
        settings: RequestSettings,
    ) -> Result<R::Response, DapiClientError<<R::Client as TransportClient>::Error>>
    where
        R: TransportRequest + Mockable,
        R::Response: Mockable;
}

/// Access point to DAPI.
#[derive(Debug, Clone)]
pub struct DapiClient {
    address_list: Arc<RwLock<AddressList>>,
    settings: RequestSettings,
    pool: ConnectionPool,
    #[cfg(feature = "dump")]
    pub(crate) dump_dir: Option<std::path::PathBuf>,
}

impl DapiClient {
    /// Initialize new [DapiClient] and optionally override default settings.
    pub fn new(address_list: AddressList, settings: RequestSettings) -> Self {
        let address_list = Arc::new(RwLock::new(address_list));

        Self {
            address_list,
            settings,
            pool: ConnectionPool::new(),
            #[cfg(feature = "dump")]
            dump_dir: None,
        }
    }

    async fn transport<R>(&self, settings: Option<&AppliedRequestSettings>) -> R::Client
    where
        R: TransportRequest + Mockable,
        R::Response: Mockable,
    {
        let mut address_list = self.address_list.write().await;

        R::Client::with_address_list(address_list.deref_mut(), settings, &self.pool)
    }

    async fn ban_address<TE>(&self, address: &Address) -> Result<Address, DapiClientError<TE>> {
        let mut guard = self.address_list.write().await;
        let banned = guard
            .ban_address(address)
            .map_err(DapiClientError::<TE>::AddressList)?;

        // schedule unban
        tokio::spawn(schedule_unban(banned.clone(), self.address_list.clone()));
        Ok(banned)
    }
}

#[async_trait]
impl DapiRequestExecutor for DapiClient {
    /// Execute the [DapiRequest](crate::DapiRequest).
    async fn execute<R>(
        &self,
        request: R,
        settings: RequestSettings,
    ) -> Result<R::Response, DapiClientError<<R::Client as TransportClient>::Error>>
    where
        R: TransportRequest + Mockable,
        R::Response: Mockable,
    {
        // Join settings of different sources to get final version of the settings for this execution:
        let applied_settings = self
            .settings
            .override_by(R::SETTINGS_OVERRIDES)
            .override_by(settings)
            .finalize();

        // Setup retry policy:
        let retry_settings = ExponentialBuilder::default()
            .with_max_times(applied_settings.retries)
            // backon doesn't accept 1.0
            .with_factor(1.001)
            .with_min_delay(Duration::from_secs(0))
            .with_max_delay(Duration::from_secs(0));

        // Save dump dir for later use, as self is moved into routine
        #[cfg(feature = "dump")]
        let dump_dir = self.dump_dir.clone();
        #[cfg(feature = "dump")]
        let dump_request = request.clone();

        // Setup DAPI request execution routine future. It's a closure that will be called
        // more once to build new future on each retry.
        let routine = move || {
            // Try to get an address to initialize transport on:

            let _span = tracing::trace_span!(
                "execute request",
                settings = ?applied_settings,
                method = request.method_name(),
            )
            .entered();

            tracing::trace!(
                ?request,
                "calling {} with {} request",
                request.method_name(),
                request.request_name(),
            );

            let transport_request = request.clone();
            let response_name = request.response_name();
            let address_list = self.address_list.clone();

            // Create a future using `async` block that will be returned from the closure on
            // each retry. Could be just a request future, but need to unpack client first.
            async move {
                // It stays wrapped in `Result` since we want to return
                // `impl Future<Output = Result<...>`, not a `Result` itself.
                let mut transport_client = self.transport::<R>(Some(&applied_settings)).await;

                let response = transport_request
                    .execute_transport(&mut transport_client, &applied_settings)
                    .await
                    .map_err(|e| {
                        let uri = e.to_uri();
                        DapiClientError::<<R::Client as TransportClient>::Error>::Transport(e, uri)
                    });

                match &response {
                    Ok(_) => {
                        tracing::trace!(?response, "received {} response", response_name);
                    }
                    Err(error) => {
                        if error.is_node_failure() {
                            if applied_settings.ban_failed_address {
                                if let DapiClientError::Transport(te, e) = error {
                                    let uri = te.to_uri();
                                    let guard = address_list.read().await;
                                    let address = guard.get(&uri).cloned();
                                    drop(guard);

                                    if let Some(address) = address {
                                        self.ban_address(&address).await?;
                                    };
                                }
                            }
                        } else {
                            tracing::trace!(?error, "received error");
                        }
                    }
                };

                response
            }
        };

        // Start the routine with retry policy applied:
        // We allow let_and_return because `result` is used later if dump feature is enabled
        let result = routine
            .retry(&retry_settings)
            .notify(|error, duration| {
                tracing::warn!(
                    ?error,
                    "retrying error with sleeping {} secs",
                    duration.as_secs_f32()
                )
            })
            .when(|e| e.is_node_failure())
            .instrument(tracing::info_span!("request routine"))
            .await;

        if let Err(error) = &result {
            if error.is_node_failure() {
                tracing::error!(?error, "request failed");
            }
        }

        // Dump request and response to disk if dump_dir is set:
        #[cfg(feature = "dump")]
        if let Ok(result) = &result {
            Self::dump_request_response(&dump_request, result, dump_dir);
        }

        result
    }
}

/// Schedule unbanning of the address after the ban period.
/// This function is called in a separate task.
async fn schedule_unban(address: Address, address_list: Arc<RwLock<AddressList>>) {
    if let Some(deadline) = address.banned_until() {
        tokio::time::sleep_until(deadline.into()).await;

        // Unban the address if it was banned and node responded successfully this time
        // We must retrieve it again, as it might have already been modified
        let addresses = address_list.read().await;
        let address = addresses.get(&address.uri()).cloned();
        drop(addresses);

        if let Some(address) = address {
            if address.is_banned() {
                if let Some(banned_until) = address.banned_until() {
                    if banned_until <= Instant::now() {
                        let mut guard = address_list.write().await;
                        if let Err(e) = guard.unban_address(&address) {
                            tracing::error!("error unbanning address {}: {:?}", address, e);
                        }
                    };
                }
            }
        } else {
            tracing::warn!(
                "unban worker: address {:?} not in the address list anymore",
                address
            );
        }
    }
}
