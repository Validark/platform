use crate::block::block_info::BlockInfo;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Extended Block information
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub struct ExtendedBlockInfoV0 {
    /// Basic block info
    pub basic_info: BlockInfo,
    /// App hash
    pub app_hash: [u8; 32],
    /// Quorum Hash
    pub quorum_hash: [u8; 32],
    /// Signature
    #[serde(with = "signature_serializer")]
    pub signature: [u8; 96],
    /// Round
    pub round: u32,
}

/// Trait for getting values from `ExtendedBlockInfoV0`
pub trait ExtendedBlockInfoV0Getters {
    /// Returns a reference to the basic block info.
    fn basic_info(&self) -> &BlockInfo;

    /// Returns a mutable reference to the basic block info.
    fn basic_info_mut(&mut self) -> &mut BlockInfo;

    /// Returns an owned copy of the basic block info.
    fn basic_info_owned(self) -> BlockInfo;

    /// Returns the app hash.
    fn app_hash(&self) -> &[u8; 32];

    /// Returns the quorum hash.
    fn quorum_hash(&self) -> &[u8; 32];

    /// Returns the signature.
    fn signature(&self) -> &[u8; 96];

    /// Returns the round.
    fn round(&self) -> u32;
}

/// Trait for setting values in `ExtendedBlockInfoV0`
pub trait ExtendedBlockInfoV0Setters {
    /// Sets the basic block info.
    fn set_basic_info(&mut self, info: BlockInfo);

    /// Sets the app hash.
    fn set_app_hash(&mut self, hash: [u8; 32]);

    /// Sets the quorum hash.
    fn set_quorum_hash(&mut self, hash: [u8; 32]);

    /// Sets the signature.
    fn set_signature(&mut self, signature: [u8; 96]);

    /// Sets the round.
    fn set_round(&mut self, round: u32);
}

impl ExtendedBlockInfoV0Getters for ExtendedBlockInfoV0 {
    fn basic_info(&self) -> &BlockInfo {
        &self.basic_info
    }

    fn basic_info_mut(&mut self) -> &mut BlockInfo {
        &mut self.basic_info
    }

    fn basic_info_owned(self) -> BlockInfo {
        self.basic_info
    }

    fn app_hash(&self) -> &[u8; 32] {
        &self.app_hash
    }

    fn quorum_hash(&self) -> &[u8; 32] {
        &self.quorum_hash
    }

    fn signature(&self) -> &[u8; 96] {
        &self.signature
    }

    fn round(&self) -> u32 {
        self.round
    }
}

impl ExtendedBlockInfoV0Setters for ExtendedBlockInfoV0 {
    fn set_basic_info(&mut self, info: BlockInfo) {
        self.basic_info = info;
    }

    fn set_app_hash(&mut self, hash: [u8; 32]) {
        self.app_hash = hash;
    }

    fn set_quorum_hash(&mut self, hash: [u8; 32]) {
        self.quorum_hash = hash;
    }

    fn set_signature(&mut self, signature: [u8; 96]) {
        self.signature = signature;
    }

    fn set_round(&mut self, round: u32) {
        self.round = round;
    }
}

mod signature_serializer {
    use super::*;
    use serde::de::Error;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(signature: &[u8; 96], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(signature)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 96], D::Error>
    where
        D: Deserializer<'de>,
    {
        let buf: Vec<u8> = Deserialize::deserialize(deserializer)?;
        if buf.len() != 96 {
            return Err(Error::invalid_length(buf.len(), &"array of length 96"));
        }
        let mut arr = [0u8; 96];
        arr.copy_from_slice(&buf);
        Ok(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciborium::{de::from_reader, ser::into_writer};

    #[test]
    fn test_extended_block_info_v0_serde_ciborium() {
        let block_info = ExtendedBlockInfoV0 {
            basic_info: BlockInfo::default(),
            app_hash: [1; 32],
            quorum_hash: [2; 32],
            signature: [3; 96],
            round: 1,
        };

        // Serialize into a vector
        let mut encoded: Vec<u8> = vec![];
        into_writer(&block_info, &mut encoded).unwrap();

        // Deserialize from the vector
        let decoded: ExtendedBlockInfoV0 = from_reader(&encoded[..]).unwrap();

        assert_eq!(block_info, decoded);
    }
}
