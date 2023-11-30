//! Wallet for managing keys assets in Dash Core and Platform.

use async_trait::async_trait;
use dashcore_rpc::dashcore_rpc_json::ListUnspentResultEntry;
use dpp::{
    bls_signatures::PrivateKey,
    identity::{
        signer::Signer, state_transition::asset_lock_proof::AssetLockProof, IdentityPublicKey,
    },
    platform_value::BinaryData,
    ProtocolError,
};
use std::fmt::Debug;

use crate::{
    wallet::{CoreWallet, PlatformWallet, Wallet},
    Error,
};

use self::{core::CoreGrpcWallet, platform::PlatformSignerWallet};
pub mod cache;
pub mod core;
pub mod core_client;
pub mod platform;

/// Mock wallet that uses core grpc wallet and platform signer to implement wallet trait.
///
/// It provides contextual information about the state of application using
/// core wallet (connecting ) and cache for data contracts and quorum public keys
pub type MockWallet = CompositeWallet<CoreGrpcWallet, PlatformSignerWallet>;
impl Wallet for MockWallet {}

/// Wallet that combines separate Core and Platform wallets into one.
///
/// ## See also
///
/// * [CoreGrpcWallet](crate::mock::wallet::CoreGrpcWallet)
/// * [PlatformSignerWallet](crate::mock::wallet::PlatformSignerWallet)
#[derive(Debug)]
pub struct CompositeWallet<C: CoreWallet, P: PlatformWallet>
where
    C: Debug,
    P: Debug,
{
    core_wallet: C,
    platform_wallet: P,
}

impl<C: CoreWallet, P: PlatformWallet> CompositeWallet<C, P>
where
    C: Debug,
    P: Debug,
{
    /// Create new composite wallet.
    ///
    /// Create new composite wallet comprising of Core wallet and Platform wallet.
    ///
    /// Note that if the `sdk` is `None`, the wallet will not be able to fetch data contracts.
    pub fn new(core_wallet: C, platform_wallet: P) -> Self {
        Self {
            core_wallet,
            platform_wallet,
        }
    }

    /// Return Core wallet client.
    pub fn core(&self) -> &C {
        &self.core_wallet
    }

    /// Return Platform wallet client.
    pub fn platform(&self) -> &P {
        &self.platform_wallet
    }
}
#[async_trait]
impl<C: CoreWallet, P: PlatformWallet> CoreWallet for CompositeWallet<C, P>
where
    C: Debug,
    P: Debug,
{
    async fn lock_assets(&self, amount: u64) -> Result<(AssetLockProof, PrivateKey), Error> {
        self.core_wallet.lock_assets(amount).await
    }
    /// Return balance of the wallet, in satoshis.
    async fn core_balance(&self) -> Result<u64, Error> {
        self.core_wallet.core_balance().await
    }

    async fn core_utxos(&self, sum: Option<u64>) -> Result<Vec<ListUnspentResultEntry>, Error> {
        self.core_wallet.core_utxos(sum).await
    }
}

impl<C: CoreWallet, P: PlatformWallet> PlatformWallet for CompositeWallet<C, P>
where
    C: Debug,
    P: Debug,
{
    fn identity_public_key(&self, purpose: &dpp::identity::Purpose) -> Option<IdentityPublicKey> {
        self.platform_wallet.identity_public_key(purpose)
    }
}

#[async_trait]
impl<C: CoreWallet, P: PlatformWallet> Signer for CompositeWallet<C, P>
where
    C: Debug,
    P: Debug,
{
    fn sign(
        &self,
        pubkey: &IdentityPublicKey,
        message: &[u8],
    ) -> Result<BinaryData, ProtocolError> {
        self.platform_wallet.sign(pubkey, message)
    }
}
