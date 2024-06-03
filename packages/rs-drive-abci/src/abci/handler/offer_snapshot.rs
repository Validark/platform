use drive::grovedb::replication::CURRENT_STATE_SYNC_VERSION;
use tenderdash_abci::proto::abci as proto;

use crate::abci::app::{SnapshotFetchingApplication, SnapshotManagerApplication};
use crate::abci::AbciError;
use crate::error::Error;

pub fn offer_snapshot<'a, 'db: 'a, A, C: 'db>(
    app: &'a A,
    request: proto::RequestOfferSnapshot,
) -> Result<proto::ResponseOfferSnapshot, Error>
where
    A: SnapshotManagerApplication + SnapshotFetchingApplication<'db, C> + 'db,
{
    let request_app_hash: [u8; 32] = request.app_hash.try_into().map_err(|_| {
        AbciError::StateSyncBadRequest("offer_snapshot invalid app_hash length".to_string())
    })?;
    let offered_snapshot = request.snapshot.ok_or(AbciError::StateSyncBadRequest(
        "offer_snapshot empty snapshot in request".to_string(),
    ))?;
    let mut session_write_guard = app.snapshot_fetching_session().write().map_err(|_| {
        AbciError::StateSyncInternalError(
            "offer_snapshot unable to lock session (poisoned)".to_string(),
        )
    })?;
    let session = session_write_guard
        .as_mut()
        .ok_or(AbciError::StateSyncInternalError(
            "offer_snapshot unable to lock session".to_string(),
        ))?;
    if offered_snapshot.height <= session.snapshot.height {
        return Err(Error::Abci(AbciError::StateSyncBadRequest(
            "offer_snapshot already syncing newest height".to_string(),
        )));
    }
    app.platform().drive.grove.wipe().map_err(|e| {
        AbciError::StateSyncInternalError(format!("offer_snapshot unable to wipe grovedb:{}", e))
    })?;
    let state_sync_info = app
        .platform()
        .drive
        .grove
        .start_snapshot_syncing(request_app_hash, CURRENT_STATE_SYNC_VERSION)
        .map_err(|e| {
            AbciError::StateSyncInternalError(format!(
                "offer_snapshot unable to start snapshot syncing session:{}",
                e
            ))
        })?;
    session.snapshot = offered_snapshot;
    session.app_hash = request_app_hash.to_vec();
    session.state_sync_info = state_sync_info;

    let response = proto::ResponseOfferSnapshot::default();
    Ok(response)
}
