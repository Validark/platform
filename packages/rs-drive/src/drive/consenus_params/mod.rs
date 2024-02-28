use crate::drive::Drive;
use crate::error::Error;
use dpp::platform_value::Value;

const CONSENSUS_PARAMS_KEY_PREFIX: &str = "consensus_params_";

impl Drive {
    /// Fetch consensus params for specific height
    pub fn fetch_consensus_params_bytes(&self, height: u64) -> Result<Option<Vec<u8>>, Error> {
        self.grove
            .get_aux(key_with_height(height), None)
            .unwrap()
            .map_err(Error::from)
    }

    /// Store consensus params for specific height
    pub fn store_consensus_params_bytes(&self, height: u64, params: &[u8]) -> Result<(), Error> {
        self.grove
            .put_aux(key_with_height(height), params, None, None)
            .unwrap()
            .map_err(Error::from)
    }
}

fn key_with_height(height: u64) -> String {
    format!("{}{}", CONSENSUS_PARAMS_KEY_PREFIX, height)
}
