use crate::voting::vote_choices::resource_vote_choice::ResourceVoteChoice;
use crate::voting::vote_polls::VotePoll;
use crate::ProtocolError;
use bincode::{Decode, Encode};
use platform_serialization_derive::{PlatformDeserialize, PlatformSerialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Encode, Decode, PlatformDeserialize, PlatformSerialize, PartialEq)]
#[cfg_attr(
    feature = "state-transition-serde-conversion",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
#[platform_serialize(unversioned)]
pub struct ResourceVoteV0 {
    pub vote_poll: VotePoll,
    pub resource_vote_choice: ResourceVoteChoice,
}

impl Default for ResourceVoteV0 {
    fn default() -> Self {
        ResourceVoteV0 {
            vote_poll: VotePoll::default(),
            resource_vote_choice: ResourceVoteChoice::Abstain,
        }
    }
}