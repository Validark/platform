use dapi_grpc::platform::VersionedGrpcResponse;
use dpp::dashcore::Address;

use dpp::identity::core_script::CoreScript;
use dpp::identity::signer::Signer;
use dpp::identity::Identity;

use dpp::state_transition::identity_credit_withdrawal_transition::IdentityCreditWithdrawalTransition;

use crate::platform::transition::broadcast_request::BroadcastRequestForStateTransition;
use crate::{Error, Sdk};
use dpp::state_transition::identity_credit_withdrawal_transition::methods::IdentityCreditWithdrawalTransitionMethodsV0;
use dpp::state_transition::identity_topup_transition::methods::IdentityTopUpTransitionMethodsV0;
use dpp::state_transition::proof_result::StateTransitionProofResult;
use dpp::withdrawal::Pooling;
use drive::drive::Drive;
use rs_dapi_client::{DapiRequest, RequestSettings};

#[async_trait::async_trait]
pub trait WithdrawFromIdentity {
    async fn withdraw<S: Signer + Send>(
        &self,
        sdk: &Sdk,
        address: Address,
        amount: u64,
        core_fee_per_byte: Option<u32>,
        signer: S,
    ) -> Result<u64, Error>;
}

#[async_trait::async_trait]
impl WithdrawFromIdentity for Identity {
    async fn withdraw<S: Signer + Send>(
        &self,
        sdk: &Sdk,
        address: Address,
        amount: u64,
        core_fee_per_byte: Option<u32>,
        signer: S,
    ) -> Result<u64, Error> {
        let state_transition = IdentityCreditWithdrawalTransition::try_from_identity(
            self,
            CoreScript::new(address.script_pubkey()),
            amount,
            Pooling::Never,
            core_fee_per_byte.unwrap_or(1),
            signer,
            sdk.version(),
            None,
        )?;

        let request = state_transition.broadcast_request_for_state_transition()?;

        request
            .clone()
            .execute(sdk, RequestSettings::default())
            .await?;

        let request = state_transition.wait_for_state_transition_result_request()?;

        let response = request.execute(sdk, RequestSettings::default()).await?;

        let proof = response.proof_owned()?;

        let (_, result) = Drive::verify_state_transition_was_executed_with_proof(
            &state_transition,
            proof.grovedb_proof.as_slice(),
            &|_| Ok(None),
            sdk.version(),
        )?;

        match result {
            StateTransitionProofResult::VerifiedPartialIdentity(identity) => {
                identity.balance.ok_or(Error::DapiClientError(
                    "expected an identity balance".to_string(),
                ))
            }
            _ => Err(Error::DapiClientError("proved a non identity".to_string())),
        }
    }
}
