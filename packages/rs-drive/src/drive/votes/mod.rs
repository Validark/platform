use crate::drive::document::contract_document_type_path;
use crate::drive::RootTree;
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use dpp::data_contract::document_type::accessors::DocumentTypeV0Getters;
use dpp::data_contract::DataContract;
use dpp::voting::votes::contested_document_resource_vote::ContestedDocumentResourceVote;
use dpp::ProtocolError;

mod cleanup;
mod insert;
mod setup;

/// The votes tree structure looks likes this
///
///     Votes
///
///     |- Decisions [key: "d"]
///     |- Contested Resource [key: "c"]
///        |- End date Queries [key: "e"]
///        |- Identifier Votes Query [key: "i"]
///
///

pub const VOTE_DECISIONS_TREE_KEY: char = 'd';

pub const CONTESTED_RESOURCE_TREE_KEY: char = 'c';

pub const END_DATE_QUERIES_TREE_KEY: char = 'e';

pub const IDENTITY_VOTES_TREE_KEY: char = 'i';

pub(in crate::drive::votes) fn vote_root_path<'a>() -> [&'a [u8]; 1] {
    [Into::<&[u8; 1]>::into(RootTree::Votes)]
}

pub(in crate::drive::votes) fn vote_root_path_vec() -> Vec<Vec<u8>> {
    vec![vec![RootTree::Votes as u8]]
}

pub(in crate::drive::votes) fn vote_decisions_tree_path<'a>() -> [&'a [u8]; 2] {
    [
        Into::<&[u8; 1]>::into(RootTree::Votes),
        &[VOTE_DECISIONS_TREE_KEY as u8],
    ]
}

pub(in crate::drive::votes) fn vote_decisions_tree_path_vec() -> Vec<Vec<u8>> {
    vec![
        vec![RootTree::Votes as u8],
        vec![VOTE_DECISIONS_TREE_KEY as u8],
    ]
}

pub(in crate::drive::votes) fn vote_contested_resource_tree_path<'a>() -> [&'a [u8]; 2] {
    [
        Into::<&[u8; 1]>::into(RootTree::Votes),
        &[CONTESTED_RESOURCE_TREE_KEY as u8],
    ]
}

pub(in crate::drive::votes) fn vote_contested_resource_tree_path_vec() -> Vec<Vec<u8>> {
    vec![
        vec![RootTree::Votes as u8],
        vec![CONTESTED_RESOURCE_TREE_KEY as u8],
    ]
}

pub(in crate::drive::votes) fn vote_contested_resource_end_date_queries_tree_path<'a>(
) -> [&'a [u8]; 3] {
    [
        Into::<&[u8; 1]>::into(RootTree::Votes),
        &[CONTESTED_RESOURCE_TREE_KEY as u8],
        &[END_DATE_QUERIES_TREE_KEY as u8],
    ]
}

pub(in crate::drive::votes) fn vote_contested_resource_end_date_queries_tree_path_vec(
) -> Vec<Vec<u8>> {
    vec![
        vec![RootTree::Votes as u8],
        vec![CONTESTED_RESOURCE_TREE_KEY as u8],
        vec![END_DATE_QUERIES_TREE_KEY as u8],
    ]
}

pub(in crate::drive::votes) fn vote_contested_resource_identity_votes_tree_path<'a>(
) -> [&'a [u8]; 3] {
    [
        Into::<&[u8; 1]>::into(RootTree::Votes),
        &[CONTESTED_RESOURCE_TREE_KEY as u8],
        &[IDENTITY_VOTES_TREE_KEY as u8],
    ]
}

pub(in crate::drive::votes) fn vote_contested_resource_identity_votes_tree_path_vec() -> Vec<Vec<u8>>
{
    vec![
        vec![RootTree::Votes as u8],
        vec![CONTESTED_RESOURCE_TREE_KEY as u8],
        vec![IDENTITY_VOTES_TREE_KEY as u8],
    ]
}

pub(in crate::drive::votes) fn vote_contested_resource_identity_votes_tree_path_for_identity<'a>(
    identity_id: &[u8; 32],
) -> [&'a [u8]; 4] {
    [
        Into::<&[u8; 1]>::into(RootTree::Votes),
        &[CONTESTED_RESOURCE_TREE_KEY as u8],
        &[IDENTITY_VOTES_TREE_KEY as u8],
        identity_id,
    ]
}

pub(in crate::drive::votes) fn vote_contested_resource_identity_votes_tree_path_for_identity_vec(
    identity_id: &[u8; 32],
) -> Vec<Vec<u8>> {
    vec![
        vec![RootTree::Votes as u8],
        vec![CONTESTED_RESOURCE_TREE_KEY as u8],
        vec![IDENTITY_VOTES_TREE_KEY as u8],
        identity_id.to_vec(),
    ]
}

pub trait TreePath {
    fn tree_path(&self, contract: &DataContract) -> Result<Vec<&[u8]>, ProtocolError>;
}

impl TreePath for ContestedDocumentResourceVote {
    fn tree_path(&self, contract: &DataContract) -> Result<Vec<&[u8]>, ProtocolError> {
        let vote_poll = self.vote_poll();
        if contract.id() != vote_poll.contract_id {
            return Err(ProtocolError::VoteError(format!(
                "contract id of votes {} does not match supplied contract {}",
                self.vote_poll().contract_id,
                contract.id()
            )));
        }
        let document_type = contract.document_type_for_name(&vote_poll.document_type_name)?;
        let index = document_type.indexes().get(&vote_poll.index_name).ok_or(ProtocolError::UnknownContestedIndexResolution(format!("no index named {} for document type {} on contract with id {}", &vote_poll.index_name, document_type.name(), contract.id())))?;
        let mut path = contract_document_type_path(
            &vote_poll.contract_id.as_bytes(),
            &vote_poll.document_type_name,
        )
        .to_vec();

        // at this point the path only contains the parts before the index

        let Some(contested_index) = &index.contested_index else {
            return Err(ProtocolError::VoteError(
                "we expect the index in a contested document resource votes type to be contested"
                    .to_string(),
            ));
        };

        let mut properties_iter = index.properties.iter();

        while let Some(index_part) = properties_iter.next() {
            let level_name = if contested_index.contested_field_name == index_part.name {
                &contested_index.contested_field_temp_replacement_name
            } else {
                &index_part.name
            };

            path.push(level_name.as_bytes());
        }
        Ok(path)
    }
}
