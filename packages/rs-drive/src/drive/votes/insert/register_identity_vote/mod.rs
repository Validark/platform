mod v0;

use std::collections::HashMap;
use grovedb::batch::KeyInfoPath;
use crate::drive::Drive;

use crate::error::drive::DriveError;
use crate::error::Error;

use dpp::fee::fee_result::FeeResult;

use dpp::version::PlatformVersion;
use dpp::voting::votes::Vote;
use grovedb::{EstimatedLayerInformation, TransactionArg};
use dpp::block::block_info::BlockInfo;
use crate::fee::op::LowLevelDriveOperation;

impl Drive {
    pub fn register_identity_vote(
        &self,
        voter_pro_tx_hash: [u8;32],
        vote: Vote,
        block_info: &BlockInfo,
        apply: bool,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<FeeResult, Error> {
        match platform_version
            .drive
            .methods
            .vote
            .contested_resource_insert
            .register_identity_vote
        {
            0 => self.register_identity_vote_v0(voter_pro_tx_hash, vote, block_info, apply, transaction, platform_version),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "register_identity_vote".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }

    pub fn register_identity_vote_operations(
        &self,
        voter_pro_tx_hash: [u8;32],
        vote: Vote,
        block_info: &BlockInfo,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<Vec<LowLevelDriveOperation>, Error> {
        match platform_version
            .drive
            .methods
            .vote
            .contested_resource_insert
            .register_identity_vote
        {
            0 => self.register_identity_vote_operations_v0(voter_pro_tx_hash, vote, block_info, estimated_costs_only_with_layer_info, transaction, platform_version),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "register_identity_vote_operations".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}
