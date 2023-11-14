use super::broadcast_request::BroadcastRequestForStateTransition;
use crate::{Error, Sdk};
use dpp::state_transition::proof_result::StateTransitionProofResult;
use dpp::state_transition::StateTransition;
use rs_dapi_client::{DapiRequest, RequestSettings};

#[async_trait::async_trait]
pub trait BroadcastStateTransition {
    async fn broadcast(&self, sdk: &mut Sdk) -> Result<(), Error>;
    async fn broadcast_and_wait(
        &self,
        sdk: &mut Sdk,
        time_out_ms: Option<u64>,
    ) -> Result<StateTransitionProofResult, Error>;
}

#[async_trait::async_trait]
impl BroadcastStateTransition for StateTransition {
    async fn broadcast(&self, sdk: &mut Sdk) -> Result<(), Error> {
        let request = self.broadcast_request_for_state_transition()?;

        request.execute(sdk, RequestSettings::default()).await?;

        // response is empty for a broadcast, result comes from the stream wait for state transition result

        Ok(())
    }

    async fn broadcast_and_wait(
        &self,
        sdk: &mut Sdk,
        time_out_ms: Option<u64>,
    ) -> Result<StateTransitionProofResult, Error> {
        let request = self.broadcast_request_for_state_transition()?;

        request
            .clone()
            .execute(sdk, RequestSettings::default())
            .await?;

        let request = self.wait_for_state_transition_result_request()?;

        let response = request.execute(sdk, RequestSettings::default()).await?;

        todo!("not finished yet");
    }
}
