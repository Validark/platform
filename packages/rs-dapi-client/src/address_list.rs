//! Subsystem to manage DAPI nodes.

use dapi_grpc::tonic::transport::Endpoint;
use dapi_grpc::tower::discover::Change;
use http::Uri;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::time::Duration;
use std::time::{self, Instant};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::Sender;

const DEFAULT_BASE_BAN_PERIOD: Duration = Duration::from_secs(60);

/// DAPI address.
#[derive(Debug, Clone)]
pub struct Address {
    ban_count: usize,
    banned_until: Option<time::Instant>,
    uri: Uri,
}

impl Eq for Address {}

impl PartialEq<Self> for Address {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

impl PartialEq<Uri> for Address {
    fn eq(&self, other: &Uri) -> bool {
        self.uri == *other
    }
}

impl Hash for Address {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
    }
}

impl From<Uri> for Address {
    fn from(uri: Uri) -> Self {
        Address {
            ban_count: 0,
            banned_until: None,
            uri,
        }
    }
}

impl Address {
    /// Ban the [Address] so it won't be available through [AddressList::get_live_address] for some time.
    fn ban(&mut self, base_ban_period: &Duration) {
        let coefficient = (self.ban_count as f64).exp();
        let ban_period = Duration::from_secs_f64(base_ban_period.as_secs_f64() * coefficient);

        self.banned_until = Some(time::Instant::now() + ban_period);
        self.ban_count += 1;
    }

    /// Check if [Address] is banned.
    pub fn is_banned(&self) -> bool {
        self.ban_count > 0
    }

    /// Clears ban record.
    fn unban(&mut self) {
        self.ban_count = 0;
        self.banned_until = None;
    }

    /// Get [Uri] of a node.
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Get the time when the address should be unbanned
    pub(crate) fn banned_until(&self) -> Option<Instant> {
        self.banned_until
    }
}

/// [AddressList] errors
#[derive(Debug, thiserror::Error)]
pub enum AddressListError {
    /// Address not found in the list.
    #[error("address {0} not found in the list")]
    AddressNotFound(Uri),
}

/// Subscription to address list updates.
pub type Subscriber = Sender<Change<Uri, Endpoint>>;

/// Allow subscribing to address list events.
///
/// This trait is implemented by [AddressList] and can be used to subscribe to address list updates.
/// Updates are sent when an endpoint should be added or removed from the list of active endpoints.
pub trait AddressEventProvider {
    /// Subscribe to events of the address list.
    ///
    /// The subscriber will receive updates about operations executed by the address list:
    /// * adding new item,
    /// * removing an item.
    ///
    /// Calling this method multiple times will add multiple subscribers.
    ///
    /// Subscription will be removed when the subscriber is closed,
    /// that is the send method returns [TrySendError::Closed].
    ///
    /// When the subscriber is full (returns [TrySendError::Full]), the change will be dropped.
    fn subscribe(&mut self, sub: Subscriber);
}

/// A structure to manage DAPI addresses to select from
/// for [DapiRequest](crate::DapiRequest) execution.

#[derive(Debug)]
pub struct AddressList {
    addresses: BTreeMap<String, Address>,
    base_ban_period: Duration,
    subscriptions: Vec<Subscriber>,
}

impl Default for AddressList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.uri.fmt(f)
    }
}

impl AddressList {
    /// Creates an empty [AddressList] with default base ban time.
    pub fn new() -> Self {
        AddressList::with_settings(DEFAULT_BASE_BAN_PERIOD)
    }

    /// Creates an empty [AddressList] with adjustable base ban time.
    pub fn with_settings(base_ban_period: Duration) -> Self {
        AddressList {
            addresses: Default::default(),
            base_ban_period,
            subscriptions: Vec::new(),
        }
    }

    /// Sends a change notification to a subscriber.
    fn subs_send(sub: &mut Subscriber, uri: &Uri, add: bool) {
        let change = if add {
            let endpoint = Endpoint::from(uri.clone());
            Change::Insert(uri.clone(), endpoint)
        } else {
            Change::Remove(uri.clone())
        };

        match sub.try_send(change) {
            Err(e) => {
                match e {
                    TrySendError::Closed(_) => {}
                    TrySendError::Full(_) => {
                        tracing::error!(
                            ?uri, add, subscriber = ?sub,
                            "subscriber is full, dropping change; deadlocked?"
                        );
                    }
                }
                tracing::error!("Error sending change to subscriber: {:?}", e);
            }
            Ok(_) => {
                tracing::trace!(?uri, add, "sent change to subscriber");
            }
        }
    }

    /// Sends a change to all subscribers.
    /// If `add` is true, the address will be added to the list.
    /// If `add` is false, the address will be removed from the list.
    fn subs_broadcast(&mut self, address: &Address, add: bool) {
        let uri = address.uri();

        for sub in self.subscriptions.iter_mut() {
            Self::subs_send(sub, uri, add);
        }

        // Remove closed subscriptions
        self.subscriptions.retain(|s| !s.is_closed());
    }

    /// Bans address.
    /// Returns banned address with updated ban time.
    pub(crate) fn ban_address(&mut self, address: &Address) -> Result<Address, AddressListError> {
        if self.addresses.remove(&address.uri().to_string()).is_none() {
            return Err(AddressListError::AddressNotFound(address.uri.clone()));
        };

        let mut banned_address = address.clone();
        banned_address.ban(&self.base_ban_period);

        self.add(banned_address.clone());

        Ok(banned_address)
    }

    /// Clears address' ban record
    pub(crate) fn unban_address(&mut self, address: &Address) -> Result<(), AddressListError> {
        if self.addresses.remove(&address.uri().to_string()).is_none() {
            return Err(AddressListError::AddressNotFound(address.uri.clone()));
        };

        let mut unbanned_address = address.clone();
        unbanned_address.unban();
        self.add(unbanned_address);

        Ok(())
    }

    /// Adds a node [Address] to [AddressList]
    /// Returns false if the address is already in the list.
    pub fn add(&mut self, address: Address) -> bool {
        self.subs_broadcast(&address, !address.is_banned());
        self.addresses
            .insert(address.uri().to_string(), address)
            .is_none()
    }

    /// Get [Address] by [Uri].
    pub fn get(&self, uri: &Uri) -> Option<&Address> {
        self.addresses.get(&uri.to_string())
    }

    // TODO: this is the most simple way to add an address
    //  however we need to support bulk loading (e.g. providing a network name)
    //  and also fetch updated from SML.
    /// Add a node [Address] to [AddressList] by [Uri].
    /// Returns false if the address is already in the list.
    pub fn add_uri(&mut self, uri: Uri) -> bool {
        self.add(uri.into())
    }

    // /// Randomly select a not banned address.
    // pub fn get_live_address(&self) -> Option<&Address> {
    //     let mut rng = SmallRng::from_entropy();

    //     self.unbanned().into_iter().choose(&mut rng)
    // }

    /// Get all addresses that are not banned.
    fn unbanned(&self) -> Vec<&Address> {
        let now = time::Instant::now();

        self.addresses
            .iter()
            .filter(|(_uri, addr)| {
                addr.banned_until
                    .map(|banned_until| banned_until < now)
                    .unwrap_or(true)
            })
            .map(|(_, addr)| addr)
            .collect()
    }

    // /// Get banned addresses, together with ban time.
    // pub(crate) fn banned(&self) -> BTreeMap<Instant, &Address> {
    //     self.addresses
    //         .iter()
    //         .filter(|(_uri, addr)| addr.is_banned() && addr.banned_until.is_some())
    //         .map(|(_uri, addr)| {
    //             (
    //                 addr.banned_until.expect("banned_until must be set here"),
    //                 addr,
    //             )
    //         })
    //         .collect()
    // }

    /// Get number of available, not banned addresses.
    pub fn available(&self) -> usize {
        self.unbanned().len()
    }

    /// Get number of all addresses, both banned and not banned.
    pub fn len(&self) -> usize {
        self.addresses.len()
    }

    /// Check if the list is empty.
    /// Returns true if there are no addresses in the list.
    /// Returns false if there is at least one address in the list.
    /// Banned addresses are also counted.
    pub fn is_empty(&self) -> bool {
        self.addresses.is_empty()
    }
}
impl AddressEventProvider for AddressList {
    fn subscribe(&mut self, mut sub: Subscriber) {
        for addr in self.unbanned() {
            Self::subs_send(&mut sub, addr.uri(), true);
        }
        self.subscriptions.push(sub);
    }
}

impl From<&str> for AddressList {
    fn from(value: &str) -> Self {
        let uri_list: Vec<Uri> = value
            .split(',')
            .map(|uri| Uri::from_str(uri).expect("invalid uri"))
            .collect();

        Self::from_iter(uri_list)
    }
}

impl FromIterator<Uri> for AddressList {
    fn from_iter<T: IntoIterator<Item = Uri>>(iter: T) -> Self {
        let mut address_list = Self::new();
        for uri in iter {
            address_list.add_uri(uri);
        }

        address_list
    }
}
