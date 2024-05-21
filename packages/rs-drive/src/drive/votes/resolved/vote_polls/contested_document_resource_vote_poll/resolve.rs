use grovedb::TransactionArg;
use dpp::voting::vote_polls::contested_document_resource_vote_poll::ContestedDocumentResourceVotePoll;
use platform_version::version::PlatformVersion;
use crate::drive::Drive;
use crate::drive::object_size_info::{DataContractOwnedResolvedInfo, DataContractResolvedInfo};
use crate::drive::votes::resolved::vote_polls::contested_document_resource_vote_poll::{ContestedDocumentResourceVotePollWithContractInfo, ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed};
use crate::error::contract::DataContractError;
use crate::error::Error;

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

    fn resolve_owned(
        self,
        drive: &Drive,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfo, Error>;

    fn resolve_allow_borrowed<'a>(
        &self,
        drive: &Drive,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed<'a>, Error>;

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
        Ok(ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
            contract: DataContractResolvedInfo::DataContractFetchInfo(contract),
            document_type_name: document_type_name.clone(),
            index_name: index_name.clone(),
            index_values: index_values.clone(),
        })
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
        Ok(ContestedDocumentResourceVotePollWithContractInfoAllowBorrowed {
            contract: DataContractResolvedInfo::DataContractFetchInfo(contract),
            document_type_name,
            index_name,
            index_values,
        })
    }
}