//! Configuration helpers for mocking of dash-platform-sdk.
//!
//! This module contains [Config] struct that can be used to configure dash-platform-sdk.
//! It's mainly used for testing.

use dash_sdk::RequestSettings;
use dpp::platform_value::string_encoding::Encoding;
use dpp::{
    dashcore::{hashes::Hash, ProTxHash},
    prelude::Identifier,
};
use rs_dapi_client::AddressList;
use serde::Deserialize;
use std::{path::PathBuf, str::FromStr};

/// Existing document ID
///
// TODO: this is copy-paste from drive-abci `packages/rs-sdk/tests/fetch/main.rs` where it's private,
// consider defining it in `data-contracts` crate
const DPNS_DASH_TLD_DOCUMENT_ID: [u8; 32] = [
    215, 242, 197, 63, 70, 169, 23, 171, 110, 91, 57, 162, 215, 188, 38, 11, 100, 146, 137, 69, 55,
    68, 209, 224, 212, 242, 106, 141, 142, 255, 55, 207,
];

#[derive(Debug, Deserialize)]
/// Configuration for dash-platform-sdk.
///
/// Content of this configuration is loaded from environment variables or `${CARGO_MANIFEST_DIR}/.env` file
/// when the [Config::new()] is called.
/// Variable names in the enviroment and `.env` file must be prefixed with [DASH_SDK_](Config::CONFIG_PREFIX)
/// and written as SCREAMING_SNAKE_CASE (e.g. `DASH_SDK_PLATFORM_HOST`).
pub struct Config {
    /// Hostname of the Dash Platform node to connect to
    #[serde(default)]
    pub platform_host: String,
    /// Port of the Dash Platform node grpc interface
    #[serde(default)]
    pub platform_port: u16,
    /// Port of the Dash Core RPC interface running on the Dash Platform node
    #[serde(default)]
    pub core_port: u16,
    /// Username for Dash Core RPC interface
    #[serde(default)]
    pub core_user: String,
    /// Password for Dash Core RPC interface
    #[serde(default)]
    pub core_password: String,
    /// When true, use SSL for the Dash Platform node grpc interface
    #[serde(default)]
    pub platform_ssl: bool,

    /// When platform_ssl is true, use the PEM-encoded CA certificate from provided absolute path to verify the server
    #[serde(default)]
    pub platform_ca_cert_path: Option<PathBuf>,

    /// Directory where all generated test vectors will be saved.
    ///
    /// See [SdkBuilder::with_dump_dir()](crate::SdkBuilder::with_dump_dir()) for more details.
    #[serde(default = "Config::default_dump_dir")]
    pub dump_dir: PathBuf,

    // IDs of some objects generated by the testnet
    /// ID of existing identity.
    ///
    /// Format: Base58
    #[serde(default = "Config::default_identity_id")]
    pub existing_identity_id: Identifier,
    /// ID of existing data contract.
    ///
    /// Format: Base58
    #[serde(default = "Config::default_data_contract_id")]
    pub existing_data_contract_id: Identifier,
    /// Name of document type defined for [`existing_data_contract_id`](Config::existing_data_contract_id).
    #[serde(default = "Config::default_document_type_name")]
    pub existing_document_type_name: String,
    /// ID of document of the type [`existing_document_type_name`](Config::existing_document_type_name)
    /// in [`existing_data_contract_id`](Config::existing_data_contract_id).
    #[serde(default = "Config::default_document_id")]
    pub existing_document_id: Identifier,
    // Hex-encoded ProTxHash of the existing HP masternode
    #[serde(default)]
    pub masternode_owner_pro_reg_tx_hash: String,
}

impl Config {
    /// Prefix of configuration options in the environment variables and `.env` file.
    pub const CONFIG_PREFIX: &'static str = "DASH_SDK_";
    /// Load configuration from operating system environment variables and `.env` file.
    ///
    /// Create new [Config] with data from environment variables and `${CARGO_MANIFEST_DIR}/tests/.env` file.
    /// Variable names in the environment and `.env` file must be converted to SCREAMING_SNAKE_CASE and
    /// prefixed with [DASH_SDK_](Config::CONFIG_PREFIX).
    pub fn new() -> Self {
        // load config from .env file, ignore errors

        let path: String = env!("CARGO_MANIFEST_DIR").to_owned() + "/tests/.env";
        if let Err(err) = dotenvy::from_path(&path) {
            tracing::warn!(path, ?err, "failed to load config file");
        }

        let config: Self = envy::prefixed(Self::CONFIG_PREFIX)
            .from_env()
            .expect("configuration error");

        if config.is_empty() {
            tracing::warn!(path, ?config, "some config fields are empty");
            #[cfg(not(feature = "offline-testing"))]
            panic!("invalid configuration")
        }

        config
    }

    /// Check if credentials of the config are empty.
    ///
    /// Checks if fields [platform_host](Config::platform_host), [platform_port](Config::platform_port),
    /// [core_port](Config::core_port), [core_user](Config::core_user) and [core_password](Config::core_password)
    /// are not empty.
    ///
    /// Other fields are ignored.
    pub fn is_empty(&self) -> bool {
        self.core_user.is_empty()
            || self.core_password.is_empty()
            || self.platform_host.is_empty()
            || self.platform_port == 0
            || self.core_port == 0
    }

    #[allow(unused)]
    /// Create list of Platform addresses from the configuration
    pub fn address_list(&self) -> AddressList {
        let scheme = match self.platform_ssl {
            true => "https",
            false => "http",
        };

        let address: String = format!("{}://{}:{}", scheme, self.platform_host, self.platform_port);

        AddressList::from_iter(vec![http::Uri::from_str(&address).expect("valid uri")])
    }

    /// Create new SDK instance
    ///
    /// Depending on the feature flags, it will connect to the configured platform node or mock API.
    ///
    /// ## Feature flags
    ///
    /// * `offline-testing` is not set - connect to Platform and generate
    /// new test vectors during execution
    /// * `offline-testing` is set - use mock implementation and
    /// load existing test vectors from disk
    ///
    /// ## Arguments
    ///
    /// * namespace - namespace to use when storing mock expectations; this is used to separate
    /// expectations from different tests.
    ///
    /// When empty string is provided, expectations are stored in the root of the dump directory.
    pub async fn setup_api(&self, namespace: &str) -> dash_sdk::Sdk {
        let dump_dir = match namespace.is_empty() {
            true => self.dump_dir.clone(),
            false => {
                // looks like spaces are not replaced by sanitize_filename, and we don't want them as they are confusing
                let namespace = namespace.replace(' ', "_");
                self.dump_dir.join(sanitize_filename::sanitize(namespace))
            }
        };

        if dump_dir.is_relative() {
            panic!(
                "dump dir must be absolute path to avoid mistakes, got: {}",
                dump_dir.display()
            );
        }

        if dump_dir.as_os_str().eq("/") {
            panic!("cannot use namespace with root dump dir");
        }

        let request_settings = self
            .platform_ca_cert_path
            .as_ref()
            .map(|cert| {
                RequestSettings::default()
                    .with_ca_certificate(cert)
                    .expect("failed to load CA certificate")
            })
            .unwrap_or_default();

        // offline testing takes precedence over network testing
        #[cfg(all(feature = "network-testing", not(feature = "offline-testing")))]
        let sdk = {
            // Dump all traffic to disk
            let builder = dash_sdk::SdkBuilder::new(self.address_list())
                .with_core(
                    &self.platform_host,
                    self.core_port,
                    &self.core_user,
                    &self.core_password,
                )
                .with_settings(request_settings);

            #[cfg(feature = "generate-test-vectors")]
            let builder = {
                // When we use namespaces, clean up the namespaced dump dir before starting
                // to avoid mixing expectations from different test runs
                if !namespace.is_empty() {
                    if let Err(err) = std::fs::remove_dir_all(&dump_dir) {
                        tracing::warn!(?err, ?dump_dir, "failed to remove dump dir");
                    }
                    std::fs::create_dir_all(&dump_dir)
                        .expect(format!("create dump dir {}", dump_dir.display()).as_str());
                    // ensure dump dir is committed to git
                    let gitkeep = dump_dir.join(".gitkeep");
                    std::fs::write(&gitkeep, "")
                        .expect(format!("create {} file", gitkeep.display()).as_str());
                }

                builder.with_dump_dir(&dump_dir)
            };

            builder.build().expect("cannot initialize api")
        };

        // offline testing takes precedence over network testing
        #[cfg(feature = "offline-testing")]
        let sdk = {
            let mut mock_sdk = dash_sdk::SdkBuilder::new_mock()
                .with_settings(request_settings)
                .build()
                .expect("initialize api");

            mock_sdk
                .mock()
                .quorum_info_dir(&dump_dir)
                .load_expectations(&dump_dir)
                .await
                .expect("load expectations");

            mock_sdk
        };

        sdk
    }

    fn default_identity_id() -> Identifier {
        // TODO: We don't have default system identities anymore.
        //  So now I used this manually created identity to populate test vectors.
        //  Next time we need to do it again and update this value :(. This is terrible.
        //  We should automate creation of identity for SDK tests when we have time.
        Identifier::from_string(
            "J2aTnrrc8eea3pQBY91QisM3QH5FM9JK11mQCVwxeMqj",
            Encoding::Base58,
        )
        .unwrap()
        .into()
    }

    fn default_data_contract_id() -> Identifier {
        data_contracts::dpns_contract::ID_BYTES.into()
    }

    fn default_document_type_name() -> String {
        "domain".to_string()
    }
    fn default_document_id() -> Identifier {
        DPNS_DASH_TLD_DOCUMENT_ID.into()
    }

    fn default_dump_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("vectors")
    }

    /// Return ProTxHash of an existing evo node, or None if not set
    pub fn existing_protxhash(&self) -> Result<ProTxHash, String> {
        hex::decode(&self.masternode_owner_pro_reg_tx_hash)
            .map_err(|e| e.to_string())
            .and_then(|b| ProTxHash::from_slice(&b).map_err(|e| e.to_string()))
            .map_err(|e| {
                format!(
                    "Invalid {}MASTERNODE_OWNER_PRO_REG_TX_HASH {}: {}",
                    Self::CONFIG_PREFIX,
                    self.masternode_owner_pro_reg_tx_hash,
                    e
                )
            })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
