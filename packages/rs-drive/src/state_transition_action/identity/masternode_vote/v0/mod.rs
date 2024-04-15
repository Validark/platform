mod transformer;

use dpp::platform_value::Identifier;
use dpp::voting::Vote;

/// action v0
#[derive(Default, Debug, Clone)]
pub struct MasternodeVoteTransitionActionV0 {
    /// the pro tx hash identifier of the masternode
    pub pro_tx_hash: Identifier,
    /// the resource vote
    pub vote: Vote,
}
