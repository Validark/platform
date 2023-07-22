use crate::execution::validation::state_transition::documents_batch::data_triggers::{
    data_trigger_bindings_list, DataTriggerExecutionContext, DataTriggerExecutionResult,
    DataTriggerExecutor,
};
use dpp::state_transition_action::document::documents_batch::document_transition::DocumentTransitionAction;
use dpp::version::PlatformVersion;
use dpp::ProtocolError;

pub fn execute_data_triggers(
    document_transition_actions: Vec<DocumentTransitionAction>,
    context: &DataTriggerExecutionContext,
    platform_version: &PlatformVersion,
) -> Result<DataTriggerExecutionResult, ProtocolError> {
    let data_trigger_bindings = data_trigger_bindings_list(platform_version)?;

    for document_transition_action in document_transition_actions {
        let data_trigger_execution_result =
            document_transition_action.validate_with_data_triggers(context, data_trigger_bindings);

        if !data_trigger_execution_result.is_valid() {
            return Ok(data_trigger_execution_result);
        }
    }
}
