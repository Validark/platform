use std::sync::Arc;
use crate::drive::object_size_info::{DataContractOwnedResolvedInfo, DataContractResolvedInfo};
use crate::drive::votes::resolved::vote_polls::contested_document_resource_vote_poll::{
    ContestedDocumentResourceVotePollWithContractInfo,
    ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed,
};
use crate::drive::Drive;
use crate::error::contract::DataContractError;
use crate::error::Error;
use dpp::voting::vote_polls::contested_document_resource_vote_poll::ContestedDocumentResourceVotePoll;
use grovedb::TransactionArg;
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use dpp::identifier::Identifier;
use dpp::prelude::DataContract;
use platform_version::version::PlatformVersion;

/// A trait for resolving information related to a contested document resource vote poll.
///
/// This trait defines a method to resolve and retrieve the necessary contract and index
/// information associated with a contested document resource vote poll.
pub trait ContestedDocumentResourceVotePollResolver {
    /// Resolves the contested document resource vote poll information.
    ///
    /// This method fetches the contract, document type name, index name, and index values
    /// required to process a contested document resource vote poll.
    ///
    /// # Parameters
    ///
    /// * `drive`: A reference to the `Drive` object used for database interactions.
    /// * `transaction`: The transaction argument used to ensure consistency during the resolve operation.
    /// * `platform_version`: The platform version to ensure compatibility.
    ///
    /// # Returns
    ///
    /// * `Ok(ContestedDocumentResourceVotePollWithContractInfo)` - The resolved information needed for the vote poll.
    /// * `Err(Error)` - An error if the resolution process fails.
    ///
    /// # Errors
    ///
    /// This method returns an `Error` variant if there is an issue resolving the contested document resource vote poll
    /// information. The specific error depends on the underlying problem encountered during resolution.
    fn resolve(
        &self,
        drive: &Drive,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfo, Error>;

    /// Resolve owned
    fn resolve_owned(
        self,
        drive: &Drive,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfo, Error>;

    /// Resolve into a struct that allows for a borrowed contract
    fn resolve_allow_borrowed<'a>(
        &self,
        drive: &Drive,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed<'a>, Error>;

    /// Resolves into a struct, the contract itself will be held with Arc
    fn resolve_with_known_contracts_provider<'a>(
        &self,
        known_contracts_provider_fn: &impl Fn(&Identifier) -> Result<Option<Arc<DataContract>>, Error>,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed<'a>, Error>;

    /// Resolve by providing the contract
    fn resolve_with_provided_borrowed_contract<'a>(
        &self,
        data_contract: &'a DataContract,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed<'a>, Error>;

    /// Resolve owned into a struct that allows for a borrowed contract
    fn resolve_owned_allow_borrowed<'a>(
        self,
        drive: &Drive,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed<'a>, Error>;
}

impl ContestedDocumentResourceVotePollResolver for ContestedDocumentResourceVotePoll {
    fn resolve(
        &self,
        drive: &Drive,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfo, Error> {
        let ContestedDocumentResourceVotePoll {
            contract_id,
            document_type_name,
            index_name,
            index_values,
        } = self;

        let contract = drive.fetch_contract(contract_id.to_buffer(), None, None, transaction, platform_version).unwrap()?.ok_or(Error::DataContract(DataContractError::MissingContract("data contract not found when trying to resolve contested document resource vote poll".to_string())))?;
        Ok(ContestedDocumentResourceVotePollWithContractInfo {
            contract: DataContractOwnedResolvedInfo::DataContractFetchInfo(contract),
            document_type_name: document_type_name.clone(),
            index_name: index_name.clone(),
            index_values: index_values.clone(),
        })
    }

    fn resolve_owned(
        self,
        drive: &Drive,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfo, Error> {
        let ContestedDocumentResourceVotePoll {
            contract_id,
            document_type_name,
            index_name,
            index_values,
        } = self;

        let contract = drive.fetch_contract(contract_id.to_buffer(), None, None, transaction, platform_version).unwrap()?.ok_or(Error::DataContract(DataContractError::MissingContract("data contract not found when trying to resolve contested document resource vote poll".to_string())))?;
        Ok(ContestedDocumentResourceVotePollWithContractInfo {
            contract: DataContractOwnedResolvedInfo::DataContractFetchInfo(contract),
            document_type_name,
            index_name,
            index_values,
        })
    }

    fn resolve_allow_borrowed<'a>(
        &self,
        drive: &Drive,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed<'a>, Error> {
        let ContestedDocumentResourceVotePoll {
            contract_id,
            document_type_name,
            index_name,
            index_values,
        } = self;

        let contract = drive.fetch_contract(contract_id.to_buffer(), None, None, transaction, platform_version).unwrap()?.ok_or(Error::DataContract(DataContractError::MissingContract("data contract not found when trying to resolve contested document resource vote poll".to_string())))?;
        Ok(
            ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
                contract: DataContractResolvedInfo::ArcDataContractFetchInfo(contract),
                document_type_name: document_type_name.clone(),
                index_name: index_name.clone(),
                index_values: index_values.clone(),
            },
        )
    }

    fn resolve_with_known_contracts_provider<'a>(
        &self,
        known_contracts_provider_fn: &impl Fn(&Identifier) -> Result<Option<Arc<DataContract>>, Error>,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed<'a>, Error> {
        let ContestedDocumentResourceVotePoll {
            contract_id,
            document_type_name,
            index_name,
            index_values,
        } = self;

        let contract = known_contracts_provider_fn(contract_id)?.ok_or(Error::DataContract(DataContractError::MissingContract(format!("data contract with id {} can not be provided", contract_id))))?;
        Ok(
            ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
                contract: DataContractResolvedInfo::ArcDataContract(contract),
                document_type_name: document_type_name.clone(),
                index_name: index_name.clone(),
                index_values: index_values.clone(),
            },
        )
    }

    fn resolve_with_provided_borrowed_contract<'a>(
        &self,
        data_contract: &'a DataContract,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed<'a>, Error> {
        let ContestedDocumentResourceVotePoll {
            contract_id,
            document_type_name,
            index_name,
            index_values,
        } = self;

        if contract_id != data_contract.id_ref() {
            return Err(Error::DataContract(DataContractError::ProvidedContractMismatch(format!("data contract provided {} is not the one required {}", data_contract.id_ref(), contract_id))));
        }
        Ok(
            ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
                contract: DataContractResolvedInfo::BorrowedDataContract(data_contract),
                document_type_name: document_type_name.clone(),
                index_name: index_name.clone(),
                index_values: index_values.clone(),
            },
        )
    }

    fn resolve_owned_allow_borrowed<'a>(
        self,
        drive: &Drive,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed<'a>, Error> {
        let ContestedDocumentResourceVotePoll {
            contract_id,
            document_type_name,
            index_name,
            index_values,
        } = self;

        let contract = drive.fetch_contract(contract_id.to_buffer(), None, None, transaction, platform_version).unwrap()?.ok_or(Error::DataContract(DataContractError::MissingContract("data contract not found when trying to resolve contested document resource vote poll".to_string())))?;
        Ok(
            ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
                contract: DataContractResolvedInfo::ArcDataContractFetchInfo(contract),
                document_type_name,
                index_name,
                index_values,
            },
        )
    }
}
