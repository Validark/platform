use crate::data_contract::document_type::DocumentTypeRef;
use crate::ProtocolError;

pub(in crate::document) trait DocumentPlatformSerializationMethodsV0 {
    /// Serializes the document.
    ///
    /// The serialization of a document follows the pattern:
    /// id 32 bytes + owner_id 32 bytes + encoded values byte arrays
    fn serialize_v0(&self, document_type: &DocumentTypeRef) -> Result<Vec<u8>, ProtocolError>;
    /// Serializes and consumes the document.
    ///
    /// The serialization of a document follows the pattern:
    /// id 32 bytes + owner_id 32 bytes + encoded values byte arrays
    fn serialize_consume_v0(
        self,
        document_type: &DocumentTypeRef,
    ) -> Result<Vec<u8>, ProtocolError>;
}
