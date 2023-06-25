use crate::contract::Contract;

use crate::drive::Drive;

use crate::error::Error;
use crate::query::{DriveQuery, InternalClauses, WhereClause, WhereOperator};
use dpp::consensus::state::document::duplicate_unique_index_error::DuplicateUniqueIndexError;
use dpp::consensus::state::state_error::StateError;
use dpp::data_contract::document_type::DocumentType;
use dpp::document::Document;
use dpp::identifier::Identifier;
use dpp::platform_value::{platform_value, Value};
use dpp::prelude::TimestampMillis;
use dpp::validation::SimpleConsensusValidationResult;
use grovedb::TransactionArg;
use std::collections::BTreeMap;
use dpp::data_contract::DataContract;
use dpp::state_transition::documents_batch_transition::document_transition::{DocumentCreateTransitionAction, DocumentReplaceTransitionAction};
use dpp::version::drive_versions::DriveVersion;

struct UniquenessOfDataRequestV0<'a> {
    contract: &'a ContractV0,
    document_type: &'a DocumentType<'a>,
    owner_id: &'a Identifier,
    document_id: &'a Identifier,
    allow_original: bool,
    created_at: &'a Option<TimestampMillis>,
    updated_at: &'a Option<TimestampMillis>,
    data: &'a BTreeMap<String, Value>,
}

impl Drive {
    /// Internal method validating uniqueness
    pub(super) fn validate_uniqueness_of_data_v0(
        &self,
        request: UniquenessOfDataRequestV0,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<SimpleConsensusValidationResult, Error> {
        let UniquenessOfDataRequestV0 {
            contract,
            document_type,
            owner_id,
            document_id,
            allow_original,
            created_at,
            updated_at,
            data,
        } = request;

        let validation_results = document_type
            .indices
            .iter()
            .filter_map(|index| {
                if !index.unique {
                    // if a index is not unique there is no issue
                    None
                } else {
                    let where_queries = index
                        .properties
                        .iter()
                        .filter_map(|property| {
                            let value = match property.name.as_str() {
                                "$ownerId" => {
                                    platform_value!(*owner_id)
                                }
                                "$createdAt" => {
                                    if let Some(created_at) = created_at {
                                        platform_value!(*created_at)
                                    } else {
                                        return None;
                                    }
                                }
                                "$updatedAt" => {
                                    if let Some(updated_at) = updated_at {
                                        platform_value!(*updated_at)
                                    } else {
                                        return None;
                                    }
                                }

                                _ => {
                                    if let Some(value) = data.get(property.name.as_str()) {
                                        value.clone()
                                    } else {
                                        return None;
                                    }
                                }
                            };
                            Some((
                                property.name.clone(),
                                WhereClause {
                                    field: property.name.clone(),
                                    operator: WhereOperator::Equal,
                                    value,
                                },
                            ))
                        })
                        .collect::<BTreeMap<String, WhereClause>>();

                    if where_queries.len() < index.properties.len() {
                        // there are empty fields, which means that the index is no longer unique
                        None
                    } else {
                        let query = DriveQuery {
                            contract,
                            document_type,
                            internal_clauses: InternalClauses {
                                primary_key_in_clause: None,
                                primary_key_equal_clause: None,
                                in_clause: None,
                                range_clause: None,
                                equal_clauses: where_queries,
                            },
                            offset: None,
                            limit: Some(1),
                            order_by: Default::default(),
                            start_at: None,
                            start_at_included: false,
                            block_time_ms: None,
                        };

                        let query_result = self.query_documents(query, None, false, transaction);
                        match query_result {
                            Ok(query_outcome) => {
                                let documents = query_outcome.documents;
                                let would_be_unique = documents.is_empty()
                                    || (allow_original
                                    && documents.len() == 1
                                    && documents[0].id == document_id);
                                if would_be_unique {
                                    Some(Ok(SimpleConsensusValidationResult::default()))
                                } else {
                                    Some(Ok(SimpleConsensusValidationResult::new_with_error(
                                        StateError::DuplicateUniqueIndexError(
                                            DuplicateUniqueIndexError::new(
                                                *document_id,
                                                index.fields(),
                                            ),
                                        )
                                            .into(),
                                    )))
                                }
                            }
                            Err(e) => Some(Err(e)),
                        }
                    }
                }
            })
            .collect::<Result<Vec<SimpleConsensusValidationResult>, Error>>()?;

        Ok(SimpleConsensusValidationResult::merge_many_errors(
            validation_results,
        ))
    }
}
