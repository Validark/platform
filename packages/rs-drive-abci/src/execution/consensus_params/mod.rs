use crate::error::serialization::SerializationError;
use crate::error::Error;
use crate::platform_types::platform::Platform;
use dpp::ProtocolError;
use prost::bytes::Bytes;
use prost::Message;
use tenderdash_abci::proto::types::ConsensusParams;

impl<C> Platform<C> {
    /// Determine consensus params that shall be returned at provided height
    ///
    /// If a file `$height.json` is found in `consensus_params_dir` directory, it will be returned.
    /// Otherwise, returns None.
    ///
    /// # Arguments
    ///
    /// * `consensus_params_dir` - Directory where consensus params are stored; if empty string, returns None
    /// * `height` - Height for which consensus params are requested
    ///
    /// # Returns
    ///
    /// * `Ok(Some(ConsensusParams))` - If file with consensus params for provided height is found
    /// * `Ok(None)` - If file with consensus params for provided height is not found
    /// * `Err(io::Error)` - If there was an error reading the file
    ///
    // TODO: Move this to correct place
    pub fn consensus_params_update_for_height(
        &self,
        height: u64,
    ) -> Result<Option<ConsensusParams>, Error> {
        let Some(bytes) = self.drive.fetch_consensus_params_bytes(height)? else {
            return Ok(None);
        };

        let params = ConsensusParams::decode(Bytes::from(bytes)).map_err(|e| {
            Error::Serialization(SerializationError::CorruptedDeserialization(e.to_string()))
        })?;

        Ok(Some(params))
    }

    pub fn store_consensus_params_update_for_height(
        &self,
        height: u64,
        json: &str,
    ) -> Result<(), Error> {
        let params: ConsensusParams = serde_json::from_str(json)
            .map_err(|e| Error::Protocol(ProtocolError::ParsingJsonError(e)))?;

        self.drive
            .store_consensus_params_bytes(height, &params.encode_to_vec())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test::helpers::setup::TestPlatformBuilder;

    #[test]
    fn test_store_and_get_consensus_params_for_specific_height() {
        let platform = TestPlatformBuilder::default()
            .build_with_mock_rpc()
            .platform;

        let height = 123456;

        let consensus_params = r#"{
            "block": {
            "max_bytes": "2097152",
            "max_gas": "40000000000"
            },
            "evidence": {
            "max_age_num_blocks": "100000",
            "max_age_duration": "172800000000000",
            "max_bytes": "0"
            },
            "validator": {
            "pub_key_types": [
                "bls12381"
            ]
            },
            "version": {
            "app_version": "1"
            },
            "synchrony": {
            "precision": "500000000",
            "message_delay": "60000000000"
            },
            "timeout": {
            "propose": "40000000000",
            "propose_delta": "5000000000",
            "vote": "40000000000",
            "vote_delta": "5000000000",
            "bypass_commit_timeout": false
            },
            "abci": {
            "recheck_tx": true
            }
        }"#;

        platform
            .store_consensus_params_update_for_height(height, consensus_params)
            .expect("should store params");

        let params = platform
            .consensus_params_update_for_height(height)
            .expect("should get params")
            .expect("should exists");

        assert_eq!(params.block.unwrap().max_bytes, 2097152);
    }
}
