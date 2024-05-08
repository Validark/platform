use crate::identity::state_transition::asset_lock_proof::{Decode, Encode};
use crate::voting::vote_polls::contested_document_resource_vote_poll::ContestedDocumentResourceVotePoll;
use crate::ProtocolError;
use derive_more::From;
use platform_value::Identifier;
use platform_version::version::PlatformVersion;
use serde::{Deserialize, Serialize};

pub mod contested_document_resource_vote_poll;

#[derive(Debug, Clone, Encode, Decode, PartialEq, From)]
#[cfg_attr(
    feature = "state-transition-serde-conversion",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum VotePoll {
    ContestedDocumentResourceVotePoll(ContestedDocumentResourceVotePoll),
}

impl Default for VotePoll {
    fn default() -> Self {
        ContestedDocumentResourceVotePoll::default().into()
    }
}

impl VotePoll {
    pub fn specialized_balance_id(&self) -> Result<Option<Identifier>, ProtocolError> {
        match self {
            VotePoll::ContestedDocumentResourceVotePoll(contested_document_resource_vote_poll) => {
                Ok(Some(
                    contested_document_resource_vote_poll.specialized_balance_id()?,
                ))
            }
        }
    }
}
