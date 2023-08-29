mod v0;

use crate::error::execution::ExecutionError;
use crate::error::Error;
use crate::execution::types::execution_event::ExecutionEvent::{
    PaidDriveEvent, PaidFromAssetLockDriveEvent,
};
use dpp::block::block_info::BlockInfo;
use dpp::block::epoch::Epoch;
use dpp::fee::Credits;

use dpp::identity::PartialIdentity;

use dpp::version::PlatformVersion;
use drive::drive::batch::drive_op_batch::DriveLowLevelOperationConverter;
use drive::state_transition_action::StateTransitionAction;

use drive::drive::batch::transitions::DriveHighLevelOperationConverter;
use drive::drive::batch::DriveOperation;
use drive::drive::Drive;
use drive::fee::op::LowLevelDriveOperation;
use drive::grovedb::TransactionArg;

/// An execution event
#[derive(Clone)]
pub(in crate::execution) enum ExecutionEvent<'a> {
    /// A drive event that is paid by an identity
    PaidDriveEvent {
        /// The identity requesting the event
        identity: PartialIdentity,
        /// the operations that the identity is requesting to perform
        operations: Vec<DriveOperation<'a>>,
    },
    /// A drive event that is paid from an asset lock
    PaidFromAssetLockDriveEvent {
        /// The identity requesting the event
        identity: PartialIdentity,
        /// The added balance
        added_balance: Credits,
        /// the operations that should be performed
        operations: Vec<DriveOperation<'a>>,
    },
    /// A drive event that is free
    FreeDriveEvent {
        /// the operations that should be performed
        operations: Vec<DriveOperation<'a>>,
    },
}

impl<'a> ExecutionEvent<'a> {
    /// Creates a new identity Insertion Event
    pub fn new_document_operation(
        identity: PartialIdentity,
        operation: DriveOperation<'a>,
    ) -> Self {
        Self::PaidDriveEvent {
            identity,
            operations: vec![operation],
        }
    }
    /// Creates a new identity Insertion Event
    pub fn new_contract_operation(
        identity: PartialIdentity,
        operation: DriveOperation<'a>,
    ) -> Self {
        Self::PaidDriveEvent {
            identity,
            operations: vec![operation],
        }
    }
    /// Creates a new identity Insertion Event
    pub fn new_identity_insertion(
        identity: PartialIdentity,
        operations: Vec<DriveOperation<'a>>,
    ) -> Self {
        Self::PaidDriveEvent {
            identity,
            operations,
        }
    }

    /// Return operations
    pub fn operations(&self) -> &Vec<DriveOperation<'a>> {
        match self {
            PaidDriveEvent { operations, .. } => operations,
            ExecutionEvent::PaidFromAssetLockDriveEvent { operations, .. } => operations,
            ExecutionEvent::FreeDriveEvent { operations } => operations,
        }
    }

    /// Return low level drive operations
    pub fn low_level_drive_operations(
        &self,
        drive: &Drive,
        block_info: &BlockInfo,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<Vec<Vec<LowLevelDriveOperation>>, Error> {
        self.operations()
            .iter()
            .map(|drive_op| {
                drive_op.clone()
                    .into_low_level_drive_operations(
                        drive,
                        &mut None,
                        block_info,
                        transaction,
                        platform_version,
                    )
                    .map_err(Error::Drive)
            })
            .collect()
    }
}

impl<'a> ExecutionEvent<'a> {
    pub(crate) fn create_from_state_transition_action(
        action: StateTransitionAction,
        identity: Option<PartialIdentity>,
        epoch: &Epoch,
        platform_version: &PlatformVersion,
    ) -> Result<Self, Error> {
        match &action {
            StateTransitionAction::IdentityCreateAction(identity_create_action) => {
                let identity = identity_create_action.into();
                let added_balance = identity_create_action.initial_balance_amount();
                let operations =
                    action.into_high_level_drive_operations(epoch, platform_version)?;
                Ok(PaidFromAssetLockDriveEvent {
                    identity,
                    added_balance,
                    operations,
                })
            }
            StateTransitionAction::IdentityTopUpAction(identity_top_up_action) => {
                let added_balance = identity_top_up_action.top_up_balance_amount();
                let operations =
                    action.into_high_level_drive_operations(epoch, platform_version)?;
                if let Some(identity) = identity {
                    Ok(PaidFromAssetLockDriveEvent {
                        identity,
                        added_balance,
                        operations,
                    })
                } else {
                    Err(Error::Execution(ExecutionError::CorruptedCodeExecution(
                        "partial identity should be present",
                    )))
                }
            }
            _ => {
                let operations =
                    action.into_high_level_drive_operations(epoch, platform_version)?;
                if let Some(identity) = identity {
                    Ok(PaidDriveEvent {
                        identity,
                        operations,
                    })
                } else {
                    Err(Error::Execution(ExecutionError::CorruptedCodeExecution(
                        "partial identity should be present",
                    )))
                }
            }
        }
    }
}
