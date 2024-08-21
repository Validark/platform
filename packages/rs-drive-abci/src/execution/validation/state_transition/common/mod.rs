/// A module for validating asset locks
pub mod asset_lock;
pub mod validate_identity_public_key_contract_bounds;
pub mod validate_identity_public_key_ids_dont_exist_in_state;
pub mod validate_identity_public_key_ids_exist_in_state;
pub mod validate_not_disabling_last_master_key;
pub mod validate_simple_pre_check_balance;
pub mod validate_state_transition_identity_signed;
mod validate_temporary_disabled_contested_documents;
pub mod validate_unique_identity_public_key_hashes_in_state;
