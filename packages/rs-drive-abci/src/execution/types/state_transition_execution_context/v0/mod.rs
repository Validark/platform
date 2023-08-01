use crate::execution::types::execution_operation::ExecutionOperation;

#[derive(Debug, Clone, Default)]
pub struct StateTransitionExecutionContextV0 {
    operations: Vec<ExecutionOperation>,
}
