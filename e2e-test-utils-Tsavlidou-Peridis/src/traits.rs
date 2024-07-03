use reth::rpc::types::{
    engine::{ExecutionPayloadEnvelopeV3, OptimismExecutionPayloadEnvelopeV3},
    ExecutionPayloadV3,
};

/// Trait to get an execution payload V3.
pub trait PayloadEnvelopeExt: Send + Sync + std::fmt::Debug {
    /// Returns the execution payload V3.
    fn execution_payload(&self) -> ExecutionPayloadV3;
}

/// Implementation for `OptimismExecutionPayloadEnvelopeV3`.
impl PayloadEnvelopeExt for OptimismExecutionPayloadEnvelopeV3 {
    fn execution_payload(&self) -> ExecutionPayloadV3 {
        self.execution_payload.clone()
    }
}

/// Implementation for `ExecutionPayloadEnvelopeV3`.
impl PayloadEnvelopeExt for ExecutionPayloadEnvelopeV3 {
    fn execution_payload(&self) -> ExecutionPayloadV3 {
        self.execution_payload.clone()
    }
}
