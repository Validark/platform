use dpp::consensus::state::data_trigger::data_trigger_condition_error::DataTriggerConditionError;
use dpp::state_transition_action::document::documents_batch::document_transition::document_base_transition_action::DocumentBaseTransitionActionAccessorsV0;
use crate::error::Error;
use crate::execution::validation::state_transition::documents_batch::data_triggers::DataTriggerExecutionResult;
use dpp::state_transition_action::document::documents_batch::document_transition::DocumentTransitionAction;
use dpp::version::PlatformVersion;

use super::DataTriggerExecutionContext;

/// Creates a data trigger for handling document rejections.
///
/// The trigger is executed whenever a document is rejected on the blockchain.
/// It performs various actions depending on the state of the document and the context in which it was rejected.
///
/// # Arguments
///
/// * `document_transition` - A reference to the document transition that triggered the data trigger.
/// * `context` - A reference to the data trigger execution context.
/// * `_top_level_identity` - An unused parameter for the top-level identity associated with the rejected document
///   (which is not needed for this trigger).
///
/// # Returns
///
/// A `SimpleValidationResult` containing either a `DataTriggerActionError` indicating the failure of the trigger
/// or an empty result indicating the success of the trigger.
pub fn reject_data_trigger_v0(
    document_transition: &DocumentTransitionAction,
    context: &DataTriggerExecutionContext<'_>,
    _platform_version: &PlatformVersion,
) -> Result<DataTriggerExecutionResult, Error> {
    let mut result = DataTriggerExecutionResult::default();

    let err = DataTriggerConditionError::new(
        context.data_contract.id(),
        document_transition.base().id(),
        "Action is not allowed".to_string(),
    );

    result.add_error(err);

    Ok(result)
}