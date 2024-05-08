use crate::transport::{CoreGrpcClient, PlatformGrpcClient};
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, RwLock},
};

/// ConnectionPool represents already established connections to DAPI nodes.
///
/// It can be cloned and shared between threads.
/// Cloning the pool will create a new reference to the same pool.
#[derive(Debug, Clone, Default)]
pub struct ConnectionPool {
    inner: Arc<RwLock<HashMap<PoolKey, PoolItem>>>,
}

impl ConnectionPool {
    /// Create a new pool.
    /// The pool will store up to one item for each key.
    ///
    /// # Panics
    ///
    /// Panics if the capacity is zero.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConnectionPool {
    /// Get item from the pool for the given uri and settings.
    ///
    /// # Arguments
    /// * `key` - type of item to get.
    pub fn get(&self, key: PoolKey) -> Option<PoolItem> {
        let guard = self.inner.read().expect("pool lock poisoned");
        let item = guard.get(&key).cloned();

        item
    }

    /// Get value from cache or create it using provided closure.
    /// If value is already in the cache, it will be returned.
    /// If value is not in the cache, it will be created by calling `create()` and stored in the cache.
    ///
    /// # Arguments
    /// * `key` - type of item to get.
    /// * `create` - closure that creates the item if it is not in the cache.
    pub fn get_or_create(&self, key: PoolKey, create: impl FnOnce() -> PoolItem) -> PoolItem {
        if let Some(cli) = self.get(key) {
            return cli;
        }

        let cli = create();
        self.put(key, cli.clone());
        cli
    }

    /// Put item into the pool for the given uri and settings.
    pub fn put(&self, id: PoolKey, value: PoolItem) {
        self.inner
            .write()
            .expect("pool lock poisoned")
            .insert(id, value);
    }
}

/// Item stored in the pool.
///
/// We use an enum as we need to represent two different types of clients.
#[derive(Clone, Debug)]
pub enum PoolItem {
    Core(CoreGrpcClient),
    Platform(PlatformGrpcClient),
}

impl From<PlatformGrpcClient> for PoolItem {
    fn from(client: PlatformGrpcClient) -> Self {
        Self::Platform(client)
    }
}
impl From<CoreGrpcClient> for PoolItem {
    fn from(client: CoreGrpcClient) -> Self {
        Self::Core(client)
    }
}

impl From<PoolItem> for PlatformGrpcClient {
    fn from(client: PoolItem) -> Self {
        match client {
            PoolItem::Platform(client) => client,
            _ => {
                tracing::error!(
                    ?client,
                    "invalid connection fetched from pool: expected platform client"
                );
                panic!("ClientType is not Platform: {:?}", client)
            }
        }
    }
}

impl From<PoolItem> for CoreGrpcClient {
    fn from(client: PoolItem) -> Self {
        match client {
            PoolItem::Core(client) => client,
            _ => {
                tracing::error!(
                    ?client,
                    "invalid connection fetched from pool: expected core client"
                );
                panic!("ClientType is not Core: {:?}", client)
            }
        }
    }
}

/// Prefix for the item in the pool. Used to distinguish between Core and Platform clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PoolKey {
    Core,
    Platform,
}

impl Display for PoolKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolKey::Core => write!(f, "Core"),
            PoolKey::Platform => write!(f, "Platform"),
        }
    }
}
impl From<&PoolItem> for PoolKey {
    fn from(item: &PoolItem) -> Self {
        match item {
            PoolItem::Core(_) => PoolKey::Core,
            PoolItem::Platform(_) => PoolKey::Platform,
        }
    }
}
