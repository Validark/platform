use crate::error::Error;
use crate::platform_types::platform::Platform;
use drive::grovedb::Transaction;
use std::sync::{Arc, RwLock};

mod check_tx;
/// Block update
mod consensus;
mod full;

use crate::rpc::core::DefaultCoreRPC;
pub use check_tx::CheckTxAbciApplication;
pub use consensus::ConsensusAbciApplication;
pub use full::FullAbciApplication;

/// Platform-based ABCI application
pub trait PlatformApplication<C = DefaultCoreRPC> {
    /// Returns Platform
    fn platform(&self) -> &Platform<C>;
}

/// ABCI application with name
pub trait NamedApplication {
    /// Returns Platform
    fn name(&self) -> String;
}

/// Transactional ABCI application
pub trait TransactionalApplication<'a> {
    /// Creates and keeps a new transaction
    fn start_transaction(&self);

    /// Returns the current transaction
    fn transaction(&self) -> &RwLock<Option<Transaction<'a>>>;

    /// Commits created transaction
    fn commit_transaction(&self) -> Result<(), Error>;
}
