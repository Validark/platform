use std::collections::BTreeMap;
use std::ops::AddAssign;
use std::sync::Arc;
use costs::{cost_return_on_error_no_add, CostContext, CostResult, CostsExt, OperationCost};
use grovedb::{Element, TransactionArg};
use grovedb::query_result_type::{QueryResultElement, QueryResultType};
use dpp::block::epoch::Epoch;
use dpp::data_contract::DataContract;
use dpp::serialization_traits::PlatformDeserializable;
use dpp::version::drive_versions::DriveVersion;
use crate::common::decode::decode_u64;
use crate::drive::contract::{ContractFetchInfo, paths};
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use crate::fee::op::LowLevelDriveOperation::{CalculatedCostOperation, PreCalculatedFeeResult};


impl Drive {


    /// Fetches a contract along with its history.
    ///
    /// # Arguments
    ///
    /// * `contract_id` - A 32-byte array representing the unique identifier of the contract.
    ///
    /// * `transaction` - A transaction that requests the contract.
    ///
    /// * `start_at_date` - A `u64` representing the timestamp in Unix Epoch format from which to
    /// start fetching the contract's history.
    ///
    /// * `limit` - An `Option<u16>` that sets the maximum number of contract history entries
    /// to return. If `None`, the limit is set to 10. Should be between 1 and 10.
    ///
    /// * `offset` - An `Option<u16>` that sets the number of contract history entries to skip
    /// before starting to return them. If `None`, no entries are skipped.
    ///
    /// # Returns
    ///
    /// * `Result<BTreeMap<u64, Contract>, Error>` - A `Result` type, where `Ok` variant contains
    /// a `BTreeMap` with Unix timestamp as the key and contract as the value, representing
    /// the contract's history. The `Err` variant contains an `Error` in case of a failure.
    ///
    /// # Errors
    ///
    /// This function will return an `Error` in the following situations:
    ///
    /// * If the contract ID, start date, limit, or offset parameters are invalid for querying
    /// contract history.
    ///
    /// * If the contract cannot be deserialized due to protocol errors.
    ///
    /// * If the queried contract path does not refer to a contract element.
    pub(super) fn fetch_contract_with_history_v0(
        &self,
        contract_id: [u8; 32],
        transaction: TransactionArg,
        start_at_date: u64,
        limit: Option<u16>,
        offset: Option<u16>,
        drive_version: &DriveVersion,
    ) -> Result<BTreeMap<u64, Contract>, Error> {
        let mut ops = Vec::new();

        let path_query =
            Self::fetch_contract_history_query(contract_id, start_at_date, limit, offset)?;

        let (results, _cost) = self.grove_get_path_query(
            &path_query,
            transaction,
            QueryResultType::QueryKeyElementPairResultType,
            &mut ops,
            drive_version,
        )?;

        let contracts = results
            .elements
            .iter()
            .map(|el| match el {
                QueryResultElement::KeyElementPairResultItem((key, value)) => {
                    let contract_time = decode_u64(key).map_err(|_| {
                        Error::Drive(DriveError::CorruptedContractPath(
                            "contract key is not a valid u64",
                        ))
                    })?;
                    match value {
                        Element::Item(a, _flags) => {
                            let contract = DataContract::deserialize(a).map_err(Error::Protocol)?;
                            Ok((contract_time, contract))
                        }
                        _ => Err(Error::Drive(DriveError::CorruptedContractPath(
                            "contract path did not refer to a contract element",
                        ))),
                    }
                }
                _ => Err(Error::Drive(DriveError::CorruptedContractPath(
                    "contract path did not refer to a contract element",
                ))),
            })
            .collect::<Result<BTreeMap<u64, Contract>, Error>>();

        // Left like this for future additions if needed
        contracts
    }
}