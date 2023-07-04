use crate::drive::identity::key::fetch::{
    IdentityKeysRequest, KeyIDIdentityPublicKeyPairBTreeMap,
    KeyIDOptionalIdentityPublicKeyPairBTreeMap,
};
use crate::drive::Drive;
use crate::error::Error;
use dpp::fee::default_costs::KnownCostItem::FetchIdentityBalanceProcessingCost;
use crate::fee::op::LowLevelDriveOperation;
use crate::fee::result::FeeResult;
use dpp::block::epoch::Epoch;
use dpp::identifier::Identifier;
use dpp::identity::PartialIdentity;
use grovedb::TransactionArg;

use dpp::fee::default_costs::EpochCosts;
use std::collections::{BTreeMap, BTreeSet};
use dpp::state_transition::fee::fee_result::FeeResult;
use dpp::version::drive_versions::DriveVersion;

impl Drive {
    /// Fetches the Identity's balance as PartialIdentityInfo from the backing store
    /// Passing apply as false get the estimated cost instead
    pub(super) fn fetch_identity_with_balance_v0(
        &self,
        identity_id: [u8; 32],
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<Option<PartialIdentity>, Error> {
        let mut drive_operations: Vec<LowLevelDriveOperation> = vec![];
        Ok(self
            .fetch_identity_balance_operations(
                identity_id,
                true,
                transaction,
                &mut drive_operations,
                drive_version,
            )?
            .map(|balance| PartialIdentity {
                id: Identifier::new(identity_id),
                loaded_public_keys: Default::default(),
                balance: Some(balance),
                revision: None,
                not_found_public_keys: Default::default(),
            }))
    }

    /// Fetches the Identity's balance as PartialIdentityInfo from the backing store
    /// Passing apply as false get the estimated cost instead
    pub(super) fn fetch_identity_with_balance_with_cost_v0(
        &self,
        identity_id: [u8; 32],
        apply: bool,
        epoch: &Epoch,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<(Option<PartialIdentity>, FeeResult), Error> {
        let balance_cost = epoch.cost_for_known_cost_item(FetchIdentityBalanceProcessingCost);
        if !apply {
            return Ok((None, FeeResult::new_from_processing_fee(balance_cost)));
        }
        let mut drive_operations: Vec<LowLevelDriveOperation> = vec![];
        Ok((
            self.fetch_identity_balance_operations(
                identity_id,
                apply,
                transaction,
                &mut drive_operations,
                drive_version,
            )?
                .map(|balance| PartialIdentity {
                    id: Identifier::new(identity_id),
                    loaded_public_keys: Default::default(),
                    balance: Some(balance),
                    revision: None,
                    not_found_public_keys: Default::default(),
                }),
            FeeResult::new_from_processing_fee(balance_cost),
        ))
    }
}

impl Drive {

    /// Fetches the Identity's balance with keys as PartialIdentityInfo from the backing store
    /// Passing apply as false get the estimated cost instead
    pub(super) fn fetch_identity_balance_with_keys_v0(
        &self,
        identity_key_request: IdentityKeysRequest,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<Option<PartialIdentity>, Error> {
        let id = Identifier::new(identity_key_request.identity_id);
        let balance = self.fetch_identity_balance(identity_key_request.identity_id, transaction, drive_version)?;
        let Some(balance) = balance else {
            return Ok(None);
        };

        let public_keys_with_optionals = self
            .fetch_identity_keys::<KeyIDOptionalIdentityPublicKeyPairBTreeMap>(
                identity_key_request,
                transaction,
                drive_version
            )?;

        let mut loaded_public_keys = BTreeMap::new();
        let mut not_found_public_keys = BTreeSet::new();

        public_keys_with_optionals
            .into_iter()
            .for_each(|(key, value)| {
                match value {
                    None => {
                        not_found_public_keys.insert(key);
                    }
                    Some(value) => {
                        loaded_public_keys.insert(key, value);
                    }
                };
            });
        Ok(Some(PartialIdentity {
            id,
            loaded_public_keys,
            balance: Some(balance),
            revision: None,
            not_found_public_keys,
        }))
    }

    /// Fetches the Identity's balance with keys as PartialIdentityInfo from the backing store
    /// Passing apply as false get the estimated cost instead
    pub(super) fn fetch_identity_balance_with_keys_and_revision_v0(
        &self,
        identity_key_request: IdentityKeysRequest,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<Option<PartialIdentity>, Error> {
        let id = Identifier::new(identity_key_request.identity_id);
        let balance = self.fetch_identity_balance(identity_key_request.identity_id, transaction, drive_version)?;
        let Some(balance) = balance else {
            return Ok(None);
        };

        //todo: deal with apply
        let revision =
            self.fetch_identity_revision(identity_key_request.identity_id, true, transaction, drive_version)?;
        let Some(revision) = revision else {
            return Ok(None);
        };

        let public_keys_with_optionals = self
            .fetch_identity_keys::<KeyIDOptionalIdentityPublicKeyPairBTreeMap>(
                identity_key_request,
                transaction,
                drive_version,
            )?;

        let mut loaded_public_keys = BTreeMap::new();
        let mut not_found_public_keys = BTreeSet::new();

        public_keys_with_optionals
            .into_iter()
            .for_each(|(key, value)| {
                match value {
                    None => {
                        not_found_public_keys.insert(key);
                    }
                    Some(value) => {
                        loaded_public_keys.insert(key, value);
                    }
                };
            });
        Ok(Some(PartialIdentity {
            id,
            loaded_public_keys,
            balance: Some(balance),
            revision: Some(revision),
            not_found_public_keys,
        }))
    }

    /// Fetches the Identity's balance with keys as PartialIdentityInfo from the backing store
    /// Passing apply as false get the estimated cost instead
    pub(super) fn fetch_identity_balance_with_keys_with_cost_v0(
        &self,
        identity_key_request: IdentityKeysRequest,
        apply: bool,
        epoch: &Epoch,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<(Option<PartialIdentity>, FeeResult), Error> {
        let balance_cost = epoch.cost_for_known_cost_item(FetchIdentityBalanceProcessingCost);
        if !apply {
            let keys_cost = identity_key_request.processing_cost(epoch)?;
            return Ok((
                None,
                FeeResult::new_from_processing_fee(balance_cost + keys_cost),
            ));
        }
        // let's start by getting the balance
        let id = Identifier::new(identity_key_request.identity_id);
        let balance = self.fetch_identity_balance(identity_key_request.identity_id, transaction, drive_version)?;
        let Some(balance) = balance else {
            return Ok((None, FeeResult::new_from_processing_fee(balance_cost)));
        };

        let keys_cost = identity_key_request.processing_cost(epoch)?;

        let loaded_public_keys = self.fetch_identity_keys::<KeyIDIdentityPublicKeyPairBTreeMap>(
            identity_key_request,
            transaction,
            drive_version
        )?;
        Ok((
            Some(PartialIdentity {
                id,
                loaded_public_keys,
                balance: Some(balance),
                revision: None,
                not_found_public_keys: Default::default(),
            }),
            FeeResult::new_from_processing_fee(balance_cost + keys_cost),
        ))
    }
}
