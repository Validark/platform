use crate::error::query::QueryError;
use crate::error::Error;
use crate::platform_types::platform::Platform;
use crate::platform_types::platform_state::PlatformState;
use crate::query::QueryValidationResult;
use dapi_grpc::platform::v0::get_contested_vote_polls_by_end_date_request::GetContestedVotePollsByEndDateRequestV0;
use dapi_grpc::platform::v0::get_contested_vote_polls_by_end_date_response::{get_contested_vote_polls_by_end_date_response_v0, GetContestedVotePollsByEndDateResponseV0};
use dapi_grpc::platform::v0::get_contested_vote_polls_by_end_date_response::get_contested_vote_polls_by_end_date_response_v0::SerializedContestedVotePollsByTimestamp;
use dpp::check_validation_result_with_data;
use dpp::version::PlatformVersion;
use dpp::validation::ValidationResult;

use drive::error::query::QuerySyntaxError;
use drive::query::VotePollsByEndDateDriveQuery;

impl<C> Platform<C> {
    pub(super) fn query_contested_vote_polls_by_end_date_query_v0(
        &self,
        GetContestedVotePollsByEndDateRequestV0 {
            start_time_info,
            end_time_info,
            limit,
            offset,
            ascending,
            prove,
        }: GetContestedVotePollsByEndDateRequestV0,
        platform_state: &PlatformState,
        platform_version: &PlatformVersion,
    ) -> Result<QueryValidationResult<GetContestedVotePollsByEndDateResponseV0>, Error> {
        let config = &self.config.drive;

        let start_time = start_time_info.map(|start_time_info| {
            (
                start_time_info.start_time_ms,
                start_time_info.start_time_included,
            )
        });

        let end_time = end_time_info
            .map(|end_time_info| (end_time_info.end_time_ms, end_time_info.end_time_included));

        let limit = check_validation_result_with_data!(limit.map_or(
            Ok(config.default_query_limit),
            |limit| {
                let limit = u16::try_from(limit)
                    .map_err(|_| QueryError::InvalidArgument("limit out of bounds".to_string()))?;
                if limit == 0 || limit > config.default_query_limit {
                    Err(QueryError::InvalidArgument(format!(
                        "limit {} out of bounds of [1, {}]",
                        limit, config.default_query_limit
                    )))
                } else {
                    Ok(limit)
                }
            }
        ));

        let offset = check_validation_result_with_data!(offset
            .map(|offset| {
                u16::try_from(offset)
                    .map_err(|_| QueryError::InvalidArgument("offset out of bounds".to_string()))
            })
            .transpose());

        if prove && offset.is_some() && offset != Some(0) {
            return Ok(QueryValidationResult::new_with_error(QueryError::Query(
                QuerySyntaxError::RequestingProofWithOffset(format!(
                    "requesting proved contested vote polls by end date with an offset {}",
                    offset.unwrap()
                )),
            )));
        }

        let query = VotePollsByEndDateDriveQuery {
            start_time,
            limit: Some(limit),
            offset,
            order_ascending: ascending,
            end_time,
        };

        let response = if prove {
            let proof = match query.execute_with_proof(&self.drive, None, None, platform_version) {
                Ok(result) => result.0,
                Err(drive::error::Error::Query(query_error)) => {
                    return Ok(QueryValidationResult::new_with_error(QueryError::Query(
                        query_error,
                    )));
                }
                Err(e) => return Err(e.into()),
            };

            GetContestedVotePollsByEndDateResponseV0 {
                result: Some(
                    get_contested_vote_polls_by_end_date_response_v0::Result::Proof(
                        self.response_proof_v0(platform_state, proof),
                    ),
                ),
                metadata: Some(self.response_metadata_v0(platform_state)),
            }
        } else {
            let results = match query.execute_no_proof_keep_serialized(
                &self.drive,
                None,
                &mut vec![],
                platform_version,
            ) {
                Ok(result) => result,
                Err(drive::error::Error::Query(query_error)) => {
                    return Ok(QueryValidationResult::new_with_error(QueryError::Query(
                        query_error,
                    )));
                }
                Err(e) => return Err(e.into()),
            };

            let (contested_vote_polls_by_timestamps, counts): (
                Vec<SerializedContestedVotePollsByTimestamp>,
                Vec<usize>,
            ) = results
                .into_iter()
                .map(|(timestamp, contested_document_resource_vote_polls)| {
                    let len = contested_document_resource_vote_polls.len();
                    (
                        SerializedContestedVotePollsByTimestamp {
                            timestamp,
                            serialized_contested_vote_polls: contested_document_resource_vote_polls,
                        },
                        len,
                    )
                })
                .unzip();

            let count: usize = counts.into_iter().sum();

            let finished_results = if count as u16 == limit {
                let last = contested_vote_polls_by_timestamps
                    .last()
                    .expect("there should be a last one if count exists");
                let next_query = VotePollsByEndDateDriveQuery {
                    start_time: Some((last.timestamp, false)),
                    limit: Some(1),
                    offset: None,
                    order_ascending: ascending,
                    end_time,
                };

                let next_query_results = match next_query.execute_no_proof_keep_serialized(
                    &self.drive,
                    None,
                    &mut vec![],
                    platform_version,
                ) {
                    Ok(result) => result,
                    Err(drive::error::Error::Query(query_error)) => {
                        return Ok(QueryValidationResult::new_with_error(QueryError::Query(
                            query_error,
                        )));
                    }
                    Err(e) => return Err(e.into()),
                };
                next_query_results.len() == 0
            } else {
                true
            };

            GetContestedVotePollsByEndDateResponseV0 {
                result: Some(
                    get_contested_vote_polls_by_end_date_response_v0::Result::ContestedVotePollsByTimestamps(
                        get_contested_vote_polls_by_end_date_response_v0::SerializedContestedVotePollsByTimestamps {
                            contested_vote_polls_by_timestamps,
                            finished_results,
                        },
                    ),
                ),
                metadata: Some(self.response_metadata_v0(platform_state)),
            }
        };

        Ok(QueryValidationResult::new_with_data(response))
    }
}