use crate::version::fee::signature::FeeSignatureVersion;
use sha2::{Digest, Sha256};

pub mod v1;

#[derive(Clone, Debug, Default)]
pub struct FeeHashingVersion {
    pub double_sha256_base: u64,
    pub double_sha256_per_block: u64,
    pub single_sha256_base: u64,
    pub single_sha256_per_block: u64,
}

impl FeeHashingVersion {
    pub(crate) fn to_hash(&self) -> u64 {
        let mut hasher = Sha256::new();
        Digest::update(&mut hasher, &self.double_sha256_base.to_be_bytes());
        Digest::update(&mut hasher, &self.double_sha256_per_block.to_be_bytes());
        Digest::update(&mut hasher, &self.single_sha256_base.to_be_bytes());
        Digest::update(&mut hasher, &self.single_sha256_per_block.to_be_bytes());

        let result = hasher.finalize();
        // Use the first 8 bytes of the hash as the u64 representation
        let hash_bytes: [u8; 8] = result[0..8].try_into().unwrap();
        u64::from_be_bytes(hash_bytes)
    }
}
