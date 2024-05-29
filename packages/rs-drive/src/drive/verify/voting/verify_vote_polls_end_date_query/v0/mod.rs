use crate::drive::verify::RootHash;
use dpp::prelude::TimestampMillis;
use dpp::serialization::PlatformDeserializable;
use grovedb::GroveDb;
use std::collections::BTreeMap;

use crate::error::Error;

use crate::common::encode::decode_u64;
use crate::error::drive::DriveError;
use crate::query::VotePollsByEndDateDriveQuery;
use dpp::voting::vote_polls::contested_document_resource_vote_poll::ContestedDocumentResourceVotePoll;
use dpp::voting::vote_polls::VotePoll;

impl VotePollsByEndDateDriveQuery {
    /// Verifies a proof and extracts voting poll documents based on their end date.
    ///
    /// This function takes a slice of bytes `proof` containing a serialized proof,
    /// verifies the proof against a constructed path query, and deserializes the documents associated
    /// with each proof into `ContestedDocumentResourceVotePoll` instances organized by their
    /// timestamps.
    ///
    /// # Arguments
    ///
    /// * `proof` - A byte slice representing the serialized proof to be verified.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// * A tuple with a root hash and a `BTreeMap` where the keys are timestamps (in milliseconds since
    ///   the epoch) and the values are vectors of `ContestedDocumentResourceVotePoll` objects,
    ///   representing voting polls ending at those times. The map entries are sorted by the timestamps.
    /// * An `Error` variant, in case the proof verification fails or if there are deserialization
    ///   errors while parsing the documents.
    ///
    /// # Errors
    ///
    /// This function may return an `Error` if:
    /// 1. The proof verification against the constructed path query fails.
    /// 2. There is an error in deserializing the byte slices into `ContestedDocumentResourceVotePoll`
    ///    instances.
    /// 3. A required path component (timestamp) is missing in any of the paths returned in the proof,
    ///    indicating a potentially corrupted state.
    #[inline(always)]
    pub(super) fn verify_vote_polls_by_end_date_proof_v0(
        &self,
        proof: &[u8],
    ) -> Result<(RootHash, BTreeMap<TimestampMillis, Vec<VotePoll>>), Error> {
        let path_query = self.construct_path_query();
        let (root_hash, proved_key_values) = GroveDb::verify_query(proof, &path_query)?;
        let vote_polls_by_end_date = proved_key_values
            .into_iter()
            .filter_map(|(path, _, element)| Some((path, element?)))
            .map(|(path, element)| {
                let Some(last_path_component) = path.last() else {
                    return Err(Error::Drive(DriveError::CorruptedDriveState(
                        "we should always have a path not be null".to_string(),
                    )));
                };
                let timestamp = decode_u64(last_path_component).map_err(Error::from)?;
                let vote_poll_bytes = element.into_item_bytes().map_err(Error::from)?;
                let vote_poll = VotePoll::deserialize_from_bytes(&vote_poll_bytes)?;
                Ok((timestamp, vote_poll))
            })
            .collect::<Result<Vec<_>, Error>>()?
            .into_iter()
            .fold(
                BTreeMap::new(),
                |mut acc: BTreeMap<u64, Vec<VotePoll>>, (timestamp, vote_poll)| {
                    acc.entry(timestamp).or_default().push(vote_poll);
                    acc
                },
            );

        Ok((root_hash, vote_polls_by_end_date))
    }
}
