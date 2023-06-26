/// Storage of the ephemeral state
pub(in crate::execution) mod store_ephemeral_state;
/// Validator set update
pub(in crate::execution) mod validator_set_update;
/// Updating the state cache happens as the final part of block finalization
pub(in crate::execution) mod update_state_cache;
