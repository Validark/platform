use crate::data_contract::document_type::DocumentTypeRef;
use crate::ProtocolError;
use crate::version::PlatformVersion;


pub trait DocumentPlatformConversionMethodsV0 {
    /// Serializes the document.
    ///
    /// The serialization of a document follows the pattern:
    /// id 32 bytes + owner_id 32 bytes + encoded values byte arrays
    fn serialize(&self, document_type: &DocumentTypeRef, platform_version: &PlatformVersion) -> Result<Vec<u8>, ProtocolError>;
    /// Serializes and consumes the document.
    ///
    /// The serialization of a document follows the pattern:
    /// id 32 bytes + owner_id 32 bytes + encoded values byte arrays
    fn serialize_consume(
        self,
        document_type: &DocumentTypeRef,
        platform_version: &PlatformVersion
    ) -> Result<Vec<u8>, ProtocolError>;

    /// Reads a serialized document and creates a Document from it.
    fn from_bytes(
        serialized_document: &[u8],
        document_type: &DocumentTypeRef,
        platform_version: &PlatformVersion
    ) -> Result<Self, ProtocolError> where Self: Sized;
}