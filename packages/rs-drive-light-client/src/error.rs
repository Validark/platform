/// Errors
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum Error {
    /// Not initialized
    #[error("not initialized: call initialize() first")]
    NotInitialized,

    #[error("already initialized: initialize() can be called only once")]
    AlreadyInitialized,

    /// Drive error
    #[error("dash drive: {error}")]
    DriveError { error: String },

    /// Dash Protocol error
    #[error("dash protocol: {error}")]
    ProtocolError { error: String },

    /// Empty response
    #[error("empty response")]
    EmptyResponse,
    /// Empty response metadata
    #[error("empty response metadata")]
    EmptyResponseMetadata,

    /// No proof in response
    #[error("no proof in response")]
    EmptyResponseProof,

    /// Document not in proof
    #[error("requested document missing in proof")]
    DocumentMissingInProof,

    /// Decode protobuf error
    #[error("decode request protobuf: {error}")]
    ProtoRequestDecodeError { error: String },

    /// Decode protobuf response error
    #[error("decode response protobuf: {error}")]
    ProtoResponseDecodeError { error: String },

    /// Encode protobuf error
    #[error("encode protobuf: {error}")]
    ProtoEncodeError { error: String },

    /// Cannot generate signature digest for data
    #[error("cannot generate signature digest for data: {error}")]
    SignDigestFailed { error: String },

    /// Error during signature verification
    #[error("error during signature verification: {error}")]
    SignatureVerificationError { error: String },

    /// Provided quorum is invalid
    #[error("invalid quorum: {error}")]
    InvalidQuorum { error: String },

    /// Signature format is invalid
    #[error("invalid signature format: {error}")]
    InvalidSignatureFormat { error: String },

    /// Public key is invalid
    #[error("invalid public key: {error}")]
    InvalidPublicKey { error: String },

    /// Invalid signature
    #[error("invalid signature: {error}")]
    InvalidSignature { error: String },

    /// Callback error
    #[error("unexpected callback error: {error}")]
    UnexpectedCallbackError { error: String, reason: String },
}

impl From<uniffi::UnexpectedUniFFICallbackError> for Error {
    fn from(value: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::UnexpectedCallbackError {
            error: value.to_string(),
            reason: value.reason,
        }
    }
}