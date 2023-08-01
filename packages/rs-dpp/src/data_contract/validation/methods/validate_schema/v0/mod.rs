// use std::collections::BTreeMap;
// use crate::data_contract::enrich_with_base_schema::PREFIX_BYTE_0;
// use crate::data_contract::validation::data_contract_validation::BASE_DOCUMENT_SCHEMA;
// use crate::data_contract::validation::multi_validator;
// use crate::data_contract::validation::multi_validator::{
//     byte_array_has_no_items_as_parent_validator, pattern_is_valid_regex_validator,
// };
// use crate::data_contract::validation::validate_data_contract_max_depth::validate_data_contract_max_depth;
// use crate::data_contract::{DataContract, DefinitionName, DocumentName, JsonSchema, property_names};
// use crate::prelude::ConsensusValidationResult;
// use crate::validation::{JsonSchemaValidator, SimpleConsensusValidationResult};
// use crate::version::PlatformVersion;
// use crate::ProtocolError;
// use log::trace;
// use platform_value::{platform_value, Value, ValueMap};
// use crate::consensus::basic::value_error::ValueError;
// use crate::data_contract::accessors::v0::DataContractV0Getters;
// use crate::data_contract::conversion::platform_value_conversion::v0::DataContractValueConversionMethodsV0;
// use crate::data_contract::document_type::accessors::DocumentTypeV0Getters;
//
// impl DataContract {
//     /// Validate the data contract from a raw value
//     pub(super) fn validate_schema_v0(
//         documents: BTreeMap<DocumentName, JsonSchema>,
//         defs: Option<BTreeMap<DefinitionName, JsonSchema>>,
//         platform_version: &PlatformVersion,
//     ) -> Result<SimpleConsensusValidationResult, ProtocolError> {
//         let mut result = SimpleConsensusValidationResult::default();
//
//         let mut schema = Value::Map(Vec::new());
//         schema.insert(property_names::DOCUMENTS.to_string(), Value::from(documents)).map_err(ValueError::new)?;
//
//         if let Some(defs) = defs {
//             schema.insert(property_names::DEFINITIONS.to_string(), Value::from(defs)).map_err(ValueError::new)?;
//         }
//
//
//
//         let enriched_data_contract =
//             enrich_with_base_schema(&BASE_DOCUMENT_SCHEMA, PREFIX_BYTE_0, &[])?;
//
//         for (_, document_schema) in enriched_data_contract.documents()?.iter() {
//             let json_schema_validation_result =
//                 JsonSchemaValidator::validate_schema(document_schema, platform_version)?;
//             result.merge(json_schema_validation_result);
//         }
//         if !result.is_valid() {
//             return Ok(result);
//         }
//
//         for (document_name, document_type) in self.document_types().iter() {
//             if document_type.indices().is_empty() {
//                 continue;
//             }
//
//             let document_schema = self.documents()?.get(document_name).ok_or(
//                 ProtocolError::CorruptedCodeExecution(
//                     "there should be a document schema".to_string(),
//                 ),
//             )?;
//
//             //todo: re-enable
//             // let validation_result = DataContract::validate_index_naming_duplicates(
//             //     &document_type.indices(),
//             //     document_name,
//             // );
//             // result.merge(validation_result);
//             //
//             // let validation_result =
//             //     DataContract::validate_max_unique_indices(&document_type.indices(), document_name);
//             // result.merge(validation_result);
//             //
//             // let (validation_result, should_stop_further_validation) =
//             //     DataContract::validate_index_definitions(
//             //         &document_type.indices(),
//             //         document_name,
//             //         document_schema,
//             //     );
//             // result.merge(validation_result);
//             // if should_stop_further_validation {
//             //     return Ok(result);
//             // }
//         }
//
//         Ok(result)
//
//
//
//
//
//
//
//
//
//
//
//
//         let mut result = ConsensusValidationResult::default();
//         // trace!("validating against data contract meta validator");
//         // result.merge(JsonSchemaValidator::validate_data_contract_schema(
//         //     &raw_data_contract
//         //         .try_to_validating_json()
//         //         .map_err(ProtocolError::ValueError)?, platform_version,
//         // )?);
//         // if !result.is_valid() {
//         //     return Ok(result);
//         // }
//         //
//         // // todo: reenable version validation
//         // // trace!("validating by protocol protocol version validator");
//         // // result.merge(
//         // //     self.protocol_version_validator.validate(
//         // //         raw_data_contract
//         // //             .get_integer("protocolVersion")
//         // //             .map_err(ProtocolError::ValueError)?,
//         // //     )?,
//         // // );
//         // // if !result.is_valid() {
//         // //     return Ok(result);
//         // // }
//         //
//         // trace!("validating data contract max depth");
//         // result.merge(validate_data_contract_max_depth(raw_data_contract));
//         // if !result.is_valid() {
//         //     return Ok(result);
//         // }
//         //
//         // trace!("validating data contract patterns & byteArray parents");
//         // result.merge(multi_validator::validate(
//         //     raw_data_contract,
//         //     &[
//         //         pattern_is_valid_regex_validator,
//         //         byte_array_has_no_items_as_parent_validator,
//         //     ],
//         // ));
//         // if !result.is_valid() {
//         //     return Ok(result);
//         // }
//         //
//         // let data_contract = Self::from_object(raw_data_contract.clone())?;
//         // let enriched_data_contract =
//         //     data_contract.enrich_with_base_schema(&BASE_DOCUMENT_SCHEMA, PREFIX_BYTE_0, &[])?;
//         //
//         // trace!("validating the documents");
//         // for (document_type, document_schema) in enriched_data_contract.documents.iter() {
//         //     trace!("validating document schema '{}'", document_type);
//         //     let json_schema_validation_result =
//         //         JsonSchemaValidator::validate_schema(document_schema, platform_version)?;
//         //     result.merge(json_schema_validation_result);
//         // }
//         // if !result.is_valid() {
//         //     return Ok(result);
//         // }
//         //
//         // trace!("indices validation");
//         // for (document_type, document_schema) in enriched_data_contract
//         //     .documents
//         //     .iter()
//         //     .filter(|(_, value)| value.get("indices").is_some())
//         // {
//         //     trace!("validating indices in {}", document_type);
//         //     let indices = document_schema.get_indices::<Vec<_>>()?;
//         //
//         //     trace!("\t validating duplicates");
//         //     let validation_result = Self::validate_index_naming_duplicates(&indices, document_type);
//         //     result.merge(validation_result);
//         //
//         //     trace!("\t validating uniqueness");
//         //     let validation_result = Self::validate_max_unique_indices(&indices, document_type);
//         //     result.merge(validation_result);
//         //
//         //     trace!("\t validating indices");
//         //     let (validation_result, should_stop_further_validation) =
//         //         Self::validate_index_definitions(&indices, document_type, document_schema);
//         //     result.merge(validation_result);
//         //     if should_stop_further_validation {
//         //         return Ok(result);
//         //     }
//         // }
//
//         Ok(result)
//     }
// }
