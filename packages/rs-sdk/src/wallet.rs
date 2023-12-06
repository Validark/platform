//! Wallet for managing keys assets in Dash Core and Platform.

use async_trait::async_trait;
use dashcore_rpc::dashcore::Address;
use dashcore_rpc::dashcore::Txid;
pub use dashcore_rpc::dashcore_rpc_json::ListUnspentResultEntry;
use dashcore_rpc::dashcore_rpc_json::SignRawTransactionInput;
use dpp::dashcore::PrivateKey;
use dpp::dashcore::Transaction;
use dpp::identity::{signer::Signer, IdentityPublicKey, Purpose};
use dpp::platform_value::BinaryData;
use dpp::prelude::AssetLockProof;
use dpp::ProtocolError;
use std::sync::Arc;
use tokio::runtime::Handle;

use crate::mock::wallet::WalletError;

/// Wallet used by Dash Platform SDK.
///
/// Wallet is used to manage keys and addresses, and sign transactions.
/// It must provide access to operations utilizing private keys, without
/// exposing them to the Sdk.
///
/// It should also add additional level of validation and security on top of
/// Sdk, to verify that security-related operations are executed as intended by the user.
/// It can, for example, notify the user if they are about to sign a transaction
/// that will spend all their funds.
///
/// Sdk calls wallet methods whenever operations utilizing private keys are required.
///
/// Wallet should be thread-safe, as it can be used from multiple threads.
/// It should manage timeouts and other concurrency-related issues internally, as the Sdk
/// will block on wallet calls.
#[async_trait]
pub trait Wallet: Send + Sync {
    // PLATFORM WALLET FUNCTIONS

    /// Sign message using private key associated with provided public key
    async fn platform_sign(
        &self,
        pubkey: &IdentityPublicKey,
        message: &[u8],
    ) -> Result<BinaryData, WalletError>;

    /// Return default identity public key for the provided purpose.
    async fn identity_public_key(&self, purpose: &Purpose) -> Option<IdentityPublicKey>;

    // CORE WALLET FUNCTIONS

    /// Create new asset lock transaction that locks some amount of Dash to be used in Platform.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount of Dash to lock.
    ///
    /// # Returns
    ///
    /// * `AssetLockProof` - Asset lock proof.
    /// * `PrivateKey` - One-time private key used to use locked Dash in Platform.
    /// This key should be used to sign Platform transactions.
    async fn lock_assets(&self, amount: u64) -> Result<(AssetLockProof, PrivateKey), WalletError>;

    /// Return balance of the wallet, in satoshis.
    async fn core_balance(&self) -> Result<u64, WalletError>;

    /// Return list of unspent transactions with summarized balance at least `sum`
    async fn core_utxos(
        &self,
        sum: Option<u64>,
    ) -> Result<Vec<ListUnspentResultEntry>, WalletError>;

    async fn core_sign_tx(
        &self,
        tx: &Transaction,
        utxos: Option<&[SignRawTransactionInput]>,
    ) -> Result<BinaryData, WalletError>;

    /// Estimate fee for a transaction on the Core chain (L1)
    ///
    /// # Arguments
    ///
    /// * `confirmation_target` - Confirmation target in blocks
    ///
    /// # Returns
    ///
    /// * `u64` - Estimated fee in satoshis
    async fn core_estimate_fee(&self, confirmation_target: u64) -> Result<u64, WalletError>;

    /// Return change address for the wallet
    async fn core_change_address(&self) -> Result<Address, WalletError> {
        todo!("Not yet implemented")
    }

    /// Broadcast some transaction without waiing for confirmation
    async fn core_broadcast_tx(&self, signed_tx: &[u8]) -> Result<Txid, WalletError>;
}

// struct WalletSigner<W>(W);
// impl<W: Wallet> From<W> for WalletSigner<W> {
//     fn from(wallet: W) -> Self {
//         Self(wallet)
//     }
// }
// impl AsRef<dyn Signer> for dyn Wallet
// where
//     Self: Send + Sync + Wallet,
// {
//     fn as_ref(&self) -> &'static dyn Signer {
//         &WalletSigner(self)
//     }
// }

impl Signer for Box<dyn Wallet> {
    fn sign(
        &self,
        identity_public_key: &IdentityPublicKey,
        data: &[u8],
    ) -> Result<BinaryData, ProtocolError> {
        let wallet = self.as_ref();
        tokio::task::block_in_place(|| {
            Handle::current().block_on(wallet.platform_sign(identity_public_key, data))
        })
        .map_err(|e| ProtocolError::Generic(e.to_string()))
    }
}

// impl Signer for WalletSigner<&dyn Wallet>
// where
//     Self: Send + Sync,
// {
//     fn sign(
//         &self,
//         pubkey: &IdentityPublicKey,
//         message: &[u8],
//     ) -> Result<BinaryData, ProtocolError> {
//         let wallet = self.0;
//         // TODO: check if this tokio construct works
//         tokio::task::block_in_place(|| {
//             Handle::current().block_on(wallet.platform_sign(pubkey, message))
//         })
//         .map_err(|e| ProtocolError::Generic(e.to_string()))
//     }
// }
// impl<W: AsRef<dyn Wallet>> Signer for WalletSigner<W>
// where
//     W: Send + Sync,
// {
//     fn sign(
//         &self,
//         pubkey: &IdentityPublicKey,
//         message: &[u8],
//     ) -> Result<BinaryData, ProtocolError> {
//         let wallet = self.0.as_ref();
//         // TODO: check if this tokio construct works
//         tokio::task::block_in_place(|| {
//             Handle::current().block_on(wallet.platform_sign(pubkey, message))
//         })
//         .map_err(|e| ProtocolError::Generic(e.to_string()))
//     }
// }

#[async_trait]
//impl<W: AsRef<dyn Wallet>> Wallet for W
impl Wallet for &'static dyn Wallet
where
    Self: Send + Sync,
{
    async fn platform_sign(
        &self,
        pubkey: &IdentityPublicKey,
        message: &[u8],
    ) -> Result<BinaryData, WalletError> {
        (*self).platform_sign(pubkey, message).await
    }

    async fn identity_public_key(&self, purpose: &Purpose) -> Option<IdentityPublicKey> {
        (*self).identity_public_key(purpose).await
    }

    async fn lock_assets(&self, amount: u64) -> Result<(AssetLockProof, PrivateKey), WalletError> {
        (*self).lock_assets(amount).await
    }

    async fn core_balance(&self) -> Result<u64, WalletError> {
        (*self).core_balance().await
    }

    async fn core_utxos(
        &self,
        sum: Option<u64>,
    ) -> Result<Vec<ListUnspentResultEntry>, WalletError> {
        (*self).core_utxos(sum).await
    }

    async fn core_estimate_fee(&self, confirmation_target: u64) -> Result<u64, WalletError> {
        (*self).core_estimate_fee(confirmation_target).await
    }

    async fn core_sign_tx(
        &self,
        tx: &Transaction,
        utxos: Option<&[SignRawTransactionInput]>,
    ) -> Result<BinaryData, WalletError> {
        (*self).core_sign_tx(tx, utxos).await
    }

    async fn core_broadcast_tx(&self, signed_tx: &[u8]) -> Result<Txid, WalletError> {
        (*self).core_broadcast_tx(signed_tx).await
    }
}

#[async_trait]
impl<W: Wallet> Wallet for Arc<W> {
    /// Sign message using private key associated with provided public key
    async fn platform_sign(
        &self,
        pubkey: &IdentityPublicKey,
        message: &[u8],
    ) -> Result<BinaryData, WalletError> {
        self.as_ref().platform_sign(pubkey, message).await
    }

    async fn identity_public_key(&self, purpose: &Purpose) -> Option<IdentityPublicKey> {
        self.as_ref().identity_public_key(purpose).await
    }

    async fn lock_assets(&self, amount: u64) -> Result<(AssetLockProof, PrivateKey), WalletError> {
        self.as_ref().lock_assets(amount).await
    }

    async fn core_balance(&self) -> Result<u64, WalletError> {
        self.as_ref().core_balance().await
    }

    async fn core_utxos(
        &self,
        sum: Option<u64>,
    ) -> Result<Vec<ListUnspentResultEntry>, WalletError> {
        self.as_ref().core_utxos(sum).await
    }

    async fn core_estimate_fee(&self, confirmation_target: u64) -> Result<u64, WalletError> {
        self.as_ref().core_estimate_fee(confirmation_target).await
    }

    async fn core_sign_tx(
        &self,
        tx: &Transaction,
        utxos: Option<&[SignRawTransactionInput]>,
    ) -> Result<BinaryData, WalletError> {
        self.as_ref().core_sign_tx(tx, utxos).await
    }

    async fn core_broadcast_tx(&self, signed_tx: &[u8]) -> Result<Txid, WalletError> {
        self.as_ref().core_broadcast_tx(signed_tx).await
    }
}
