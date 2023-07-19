use dapi_grpc::platform::v0::{Proof, ResponseMetadata};
use dpp::{bls_signatures, dashcore::hashes::hex::ToHex};
pub use drive::drive::verify::RootHash;
use tenderdash_abci::{
    proto::types::{CanonicalVote, SignedMsgType, StateId},
    signatures::{SignBytes, SignDigest},
};

use crate::Error;

use super::from_proof::QuorumInfoProvider;

pub(crate) fn verify_tenderdash_proof(
    proof: &Proof,
    mtd: &ResponseMetadata,
    root_hash: &[u8],
    provider: &Box<dyn QuorumInfoProvider>,
) -> Result<(), Error> {
    let block_id_hash = proof.block_id_hash.to_vec();

    let version = mtd.protocol_version as u64;
    let height = mtd.height as u64;
    let round = proof.round as u32;
    let quorum_hash = TryInto::<[u8; 32]>::try_into(proof.quorum_hash.as_slice()).map_err(|e| {
        Error::InvalidQuorum {
            error: "invalid quorum hash size: ".to_string() + &e.to_string(),
        }
    })?;

    // Now, lookup quorum details
    let chain_id = mtd.chain_id.clone();
    let quorum_type = proof.quorum_type;
    let pubkey_bytes = provider.get_quorum_public_key(quorum_type, quorum_hash.to_vec())?;

    let state_id = StateId {
        app_version: version,
        core_chain_locked_height: mtd.core_chain_locked_height,
        time: mtd.time_ms,
        app_hash: root_hash.into(),
        height,
    };

    let state_id_hash = state_id
        .sha256(&chain_id, mtd.height as i64, proof.round as i32)
        .expect("failed to calculate state id hash");

    let commit = CanonicalVote {
        r#type: SignedMsgType::Precommit.into(),
        block_id: block_id_hash,
        chain_id: chain_id.clone(),
        height: mtd.height as i64,
        round: proof.round as i64,
        state_id: state_id_hash,
    };

    // Verify signature
    let sign_digest = commit
        .sign_digest(
            &chain_id,
            quorum_type.try_into().expect("quorum type out of range"),
            &quorum_hash,
            height as i64,
            round as i32,
        )
        .map_err(|e| Error::SignDigestFailed {
            error: e.to_string(),
        })?;

    let signature = TryInto::<[u8; 96]>::try_into(proof.signature.as_slice()).map_err(|e| {
        Error::InvalidSignatureFormat {
            error: "invalid signature size: ".to_string() + &e.to_string(),
        }
    })?;

    let pubkey = bls_signatures::PublicKey::from_bytes(&pubkey_bytes).map_err(|e| {
        Error::InvalidPublicKey {
            error: e.to_string(),
        }
    })?;

    tracing::trace!(
        ?state_id,
        signature = signature.to_hex(),
        sign_digest = sign_digest.to_hex(),
        ?pubkey,
        "verify signature"
    );

    match verify_signature_digest(&sign_digest, &signature, &pubkey)? {
        true => Ok(()),
        false => Err(Error::InvalidSignature {
            error: "signature mismatch".to_string(),
        }),
    }
}

/// Verify signature for [sign_digest](tenderdash_abci::signatures::SignDigest), using provided `public_key`
pub fn verify_signature_digest(
    sign_digest: &[u8],
    signature: &[u8; 96],
    public_key: &bls_signatures::PublicKey,
) -> Result<bool, Error> {
    if signature == &[0; 96] {
        return Err(Error::SignatureVerificationError {
            error: "empty signature".to_string(),
        });
    }
    let signature = bls_signatures::Signature::from_bytes(signature).map_err(|e| {
        Error::SignatureVerificationError {
            error: e.to_string(),
        }
    })?;

    Ok(public_key.verify(&signature, sign_digest))
}
