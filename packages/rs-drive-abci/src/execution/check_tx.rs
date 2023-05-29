use crate::dpp::state_repository::DPPStateRepository;
use crate::error::Error;
#[cfg(test)]
use crate::execution::execution_event::ExecutionResult;
#[cfg(test)]
use crate::execution::execution_event::ExecutionResult::ConsensusExecutionError;
use crate::platform::{Platform, PlatformRef};
use crate::rpc::core::CoreRPCLike;
use crate::validation::state_transition::process_state_transition;
use dpp::block::block_info::BlockInfo;
use dpp::consensus::ConsensusError;
use dpp::serialization_traits::PlatformDeserializable;
use dpp::state_repository::StateRepositoryLike;
use dpp::state_transition::StateTransition;
#[cfg(test)]
use dpp::validation::SimpleConsensusValidationResult;
use dpp::validation::ValidationResult;
use dpp::{BlsModule, DashPlatformProtocol, NativeBlsModule};
use drive::fee::result::FeeResult;
#[cfg(test)]
use drive::grovedb::Transaction;

impl<C> Platform<C>
where
    C: CoreRPCLike,
{
    #[cfg(test)]
    pub fn execute_tx(
        &self,
        raw_tx: Vec<u8>,
        block_info: &BlockInfo,
        dpp: &DashPlatformProtocol<DPPStateRepository<C>, NativeBlsModule>,
        transaction: &Transaction,
    ) -> Result<ExecutionResult, Error> {
        let state_transition =
            StateTransition::deserialize(raw_tx.as_slice()).map_err(Error::Protocol)?;
        let state_read_guard = self.state.read().unwrap();
        let platform_ref = PlatformRef {
            drive: &self.drive,
            state: &state_read_guard,
            config: &self.config,
            core_rpc: &self.core_rpc,
        };

        let state_transition_execution_event =
            process_state_transition(state_transition, &dpp, &platform_ref, Some(transaction))?;

        if state_transition_execution_event.is_valid() {
            let execution_event = state_transition_execution_event.into_data()?;
            self.execute_event(execution_event, block_info, transaction)
        } else {
            Ok(ConsensusExecutionError(
                SimpleConsensusValidationResult::new_with_errors(
                    state_transition_execution_event.errors,
                ),
            ))
        }
    }

    /// Checks a state transition to determine if it should be added to the mempool.
    ///
    /// This function performs a few checks, including validating the state transition and ensuring that the
    /// user can pay for it. It may be inaccurate in rare cases, so the proposer needs to re-check transactions
    /// before proposing a block.
    ///
    /// # Arguments
    ///
    /// * `raw_tx` - A raw transaction represented as a vector of bytes.
    /// * `dpp` - Dash Platform Protocol.
    ///
    /// # Returns
    ///
    /// * `Result<ValidationResult<FeeResult, ConsensusError>, Error>` - If the state transition passes all
    ///   checks, it returns a `ValidationResult` with fee information. If any check fails, it returns an `Error`.
    pub fn check_tx<SR, BLS>(
        &self,
        raw_tx: &[u8],
        dpp: &DashPlatformProtocol<SR, BLS>,
    ) -> Result<ValidationResult<FeeResult, ConsensusError>, Error>
    where
        SR: StateRepositoryLike + Clone,
        BLS: BlsModule + Clone,
    {
        let state_transition = StateTransition::deserialize(raw_tx).map_err(Error::Protocol)?;
        let state_read_guard = self.state.read().unwrap();
        let platform_ref = PlatformRef {
            drive: &self.drive,
            state: &state_read_guard,
            config: &self.config,
            core_rpc: &self.core_rpc,
        };
        let execution_event = process_state_transition(state_transition, dpp, &platform_ref, None)?;

        // We should run the execution event in dry run to see if we would have enough fees for the transaction

        // We need the approximate block info
        if let Some(block_info) = state_read_guard.last_committed_block_info.as_ref() {
            // We do not put the transaction, because this event happens outside of a block
            execution_event.and_then_borrowed_validation(|execution_event| {
                self.validate_fees_of_event(execution_event, &block_info.basic_info, None)
            })
        } else {
            execution_event.and_then_borrowed_validation(|execution_event| {
                self.validate_fees_of_event(execution_event, &BlockInfo::default(), None)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::block::{BlockExecutionContext, BlockStateInfo};
    use crate::config::{PlatformConfig, PlatformTestConfig};
    use crate::dpp::state_repository::DPPStateRepository;
    use crate::execution::execution_event::ExecutionResult::SuccessfulPaidExecution;
    use crate::platform::{Platform, PlatformWithBlockContextRef};
    use crate::test::helpers::setup::TestPlatformBuilder;
    use dpp::block::block_info::BlockInfo;
    use dpp::consensus::basic::BasicError;
    use dpp::consensus::signature::SignatureError;
    use dpp::consensus::state::state_error::StateError;
    use dpp::consensus::ConsensusError;
    use dpp::dashcore::secp256k1::{Secp256k1, SecretKey};
    use dpp::dashcore::{signer, KeyPair, PrivateKey};
    use dpp::data_contracts::dpns_contract;
    use dpp::data_contracts::SystemDataContract::DPNS;
    use dpp::identity::state_transition::identity_public_key_transitions::IdentityPublicKeyInCreation;
    use dpp::identity::state_transition::identity_update_transition::identity_update_transition::IdentityUpdateTransition;
    use dpp::identity::{Identity, KeyType, Purpose, SecurityLevel};
    use dpp::prelude::{Identifier, IdentityPublicKey};
    use dpp::serialization_traits::{PlatformDeserializable, PlatformSerializable, Signable};
    use dpp::state_repository::MockStateRepositoryLike;
    use dpp::state_transition::StateTransition::DataContractCreate;
    use dpp::state_transition::{StateTransition, StateTransitionType};
    use dpp::version::LATEST_VERSION;
    use dpp::{DPPOptions, DashPlatformProtocol, NativeBlsModule};
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::BTreeMap;
    use std::ops::Deref;
    use std::sync::{Arc, RwLock};
    use tenderdash_abci::proto::abci::RequestInitChain;
    use tenderdash_abci::proto::google::protobuf::Timestamp;

    #[test]
    fn data_contract_create_check_tx() {
        let serialized = hex::decode("00010001e2716cafada3bbe565025e4597276424c7a1d2b19bb67d3f44fa111f4e7696e300000000013468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374019e71b47e5b533e2c53366158f0d7548ba79ca6cbde04401fd7c79597bef6fb2c030f696e6465786564446f63756d656e74160512047479706512066f626a6563741207696e64696365731506160312046e616d651206696e64657831120a70726f70657274696573150216011208246f776e6572496412036173631601120966697273744e616d6512036173631206756e697175651301160312046e616d651206696e64657832120a70726f70657274696573150216011208246f776e657249641203617363160112086c6173744e616d6512036173631206756e697175651301160212046e616d651206696e64657833120a70726f706572746965731501160112086c6173744e616d651203617363160212046e616d651206696e64657834120a70726f7065727469657315021601120a2463726561746564417412036173631601120a247570646174656441741203617363160212046e616d651206696e64657835120a70726f7065727469657315011601120a247570646174656441741203617363160212046e616d651206696e64657836120a70726f7065727469657315011601120a246372656174656441741203617363120a70726f706572746965731602120966697273744e616d6516021204747970651206737472696e6712096d61784c656e677468023f12086c6173744e616d6516021204747970651206737472696e6712096d61784c656e677468023f120872657175697265641504120966697273744e616d65120a24637265617465644174120a2475706461746564417412086c6173744e616d6512146164646974696f6e616c50726f7065727469657313000c6e696365446f63756d656e74160412047479706512066f626a656374120a70726f70657274696573160112046e616d6516011204747970651206737472696e67120872657175697265641501120a2463726561746564417412146164646974696f6e616c50726f7065727469657313000e7769746842797465417272617973160512047479706512066f626a6563741207696e64696365731501160212046e616d651206696e64657831120a70726f7065727469657315011601120e6279746541727261794669656c641203617363120a70726f706572746965731602120e6279746541727261794669656c641603120474797065120561727261791209627974654172726179130112086d61784974656d730210120f6964656e7469666965724669656c64160512047479706512056172726179120962797465417272617913011210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e74696669657212086d696e4974656d73022012086d61784974656d730220120872657175697265641501120e6279746541727261794669656c6412146164646974696f6e616c50726f706572746965731300005e236018181816974c2d73a407a1d4ce4e936cb3431e0ea94e2777fb3e87f9e80141204039fe689533d8bc9137ef298c3e6a5f7c40e8001bb670a370149b1c7f3f4b326e311e3df862f4ea6f146ad858d1ef2b3a02949284cdb3b09f63974d5912b04e").expect("expected to decode");
        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        let key = IdentityPublicKey::random_authentication_key(1, Some(1));

        platform
            .drive
            .create_initial_state_structure(None)
            .expect("expected to create state structure");

        let dpp = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::new(Arc::new(PlatformWithBlockContextRef::from(
                platform.deref(),
            ))),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        let identity = Identity {
            protocol_version: 1,
            id: Identifier::new([
                158, 113, 180, 126, 91, 83, 62, 44, 83, 54, 97, 88, 240, 215, 84, 139, 167, 156,
                166, 203, 222, 4, 64, 31, 215, 199, 149, 151, 190, 246, 251, 44,
            ]),
            public_keys: BTreeMap::from([(1, key)]),
            balance: 100000,
            revision: 0,
            asset_lock_proof: None,
            metadata: None,
        };
        platform
            .drive
            .add_new_identity(identity, &BlockInfo::default(), true, None)
            .expect("expected to insert identity");

        let validation_result = platform
            .check_tx(serialized.as_slice(), &dpp)
            .expect("expected to check tx");

        //todo fix
        // assert!(validation_result.errors.is_empty());
    }

    #[test]
    fn document_update_check_tx() {
        let identity_create = hex::decode("03020200000000002102b50c44b3a3bd342d620919352097ce39e31bb2b4a485583a933827920ee5aa9d4120e3e3432eba9c7c41c789b2ca4483ef1d5c6657ab83fb4164c6517edd3944c1246461ffe2b1eee25f79fcee2fcebee5e08aa264c8b5cd42cf050eb850874823bb01000002002103f85402738806681a6e6fd14ed8e70899bd831da5bd9302e84edf3caf03bcd744411ff87579b07c9b295c6f18ad0c98986e9f2437918f0fe34047e74ac957c2976edf50788d2b16fcaea9bbbd0aa90569efca11de669d7f08ef3b5c2db1a2bb7ccc470000c601018424a8d386812dfc24fe286915b1a14763c8fb3701d6b8991912b7a54088c30d00000000a9f543365d8345621e724755bcce89f77c374038e9a7d0a0d5c7e3e9481045cb506ebd6bcf46f5db0953dd84386d9942ac4e30372fbc7456b479b65a08b8801695f7078bb338c75272b849661d26e86e40e59a283767d24f361b7172f9256b6bc5ec9d33f1a5742dd3df24f96c45f56e15a781964b9d7cc29087dc8adf4f63b7b4a1cffcf8495a0b13cea819e80355307541a18912e74210fffb367b7b3a13b0de03000000018424a8d386812dfc24fe286915b1a14763c8fb3701d6b8991912b7a54088c30d000000006a47304402201e1f8c93f0ad0b4c43d5e1c03a912e9c8463bd9409b495bb3156638dbc03094102202aaf998d0e533f0aac62fb1ab3bd3885fc96201a6f282a22801df1f9180b6b7e012102e7568866ff0c53561fc0e577b76f3614b38aa548a89adbbfc715c2a85860aaa6ffffffff0240420f0000000000166a14b6c869d709e095182d9c49a347d67632606de3a428230000000000001976a9147d863019b137a7fc55d6399b0ae7c525933fb14388ac0000000000014120f138e7c9cec7ac53f74d141141adb463b57d87599e40bc6251678d8892e7a0c4100f64be98035e46fa2b400bab870069f0f612db3f6c98a85e6dcb15ca56d7b87e27f11ff8d2539a71fea96ffb95ea7226599329fecb7d67c2332b2749d0eca0").expect("expected to decode");

        let data_contract_create = hex::decode("000100014b707b7ffe0c6170dc0b1cce6abcfd3db41480de837b8a4038ec2e044c9ec61300000000013468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374017e27f11ff8d2539a71fea96ffb95ea7226599329fecb7d67c2332b2749d0eca0030f696e6465786564446f63756d656e74160512047479706512066f626a6563741207696e64696365731506160312046e616d651206696e64657831120a70726f70657274696573150216011208246f776e6572496412036173631601120966697273744e616d6512036173631206756e697175651301160312046e616d651206696e64657832120a70726f70657274696573150216011208246f776e657249641203617363160112086c6173744e616d6512036173631206756e697175651301160212046e616d651206696e64657833120a70726f706572746965731501160112086c6173744e616d651203617363160212046e616d651206696e64657834120a70726f7065727469657315021601120a2463726561746564417412036173631601120a247570646174656441741203617363160212046e616d651206696e64657835120a70726f7065727469657315011601120a247570646174656441741203617363160212046e616d651206696e64657836120a70726f7065727469657315011601120a246372656174656441741203617363120a70726f706572746965731602120966697273744e616d6516021204747970651206737472696e6712096d61784c656e677468023f12086c6173744e616d6516021204747970651206737472696e6712096d61784c656e677468023f120872657175697265641504120966697273744e616d65120a24637265617465644174120a2475706461746564417412086c6173744e616d6512146164646974696f6e616c50726f7065727469657313000c6e696365446f63756d656e74160412047479706512066f626a656374120a70726f70657274696573160112046e616d6516011204747970651206737472696e67120872657175697265641501120a2463726561746564417412146164646974696f6e616c50726f7065727469657313000e7769746842797465417272617973160512047479706512066f626a6563741207696e64696365731501160212046e616d651206696e64657831120a70726f7065727469657315011601120e6279746541727261794669656c641203617363120a70726f706572746965731602120e6279746541727261794669656c641603120474797065120561727261791209627974654172726179130112086d61784974656d730210120f6964656e7469666965724669656c64160512047479706512056172726179120962797465417272617913011210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e74696669657212086d696e4974656d73022012086d61784974656d730220120872657175697265641501120e6279746541727261794669656c6412146164646974696f6e616c50726f70657274696573130000bdb5befd57710c51ecfbc4331d180bc22ded67a208cab00b62f7a0594115b88b01411f7f373da43c0be79b9394245ae0493bcef97fa5ed90e937548a5e565ee51426e742a3da4bf115f59e810e1e7fedeb17689d51fbeccf526c32164cc9d44ecf5c3e").expect("expected to decode");

        let document_create = hex::decode("0201017e27f11ff8d2539a71fea96ffb95ea7226599329fecb7d67c2332b2749d0eca00100bc7ad590a5a0d1bcd29c780d981e9ebcc4f00f39caa3591df870f03723e7aba40f696e6465786564446f63756d656e74004b707b7ffe0c6170dc0b1cce6abcfd3db41480de837b8a4038ec2e044c9ec613014b707b7ffe0c6170dc0b1cce6abcfd3db41480de837b8a4038ec2e044c9ec61300000000013468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374017e27f11ff8d2539a71fea96ffb95ea7226599329fecb7d67c2332b2749d0eca0030f696e6465786564446f63756d656e74160512047479706512066f626a6563741207696e64696365731506160312046e616d651206696e64657831120a70726f70657274696573150216011208246f776e6572496412036173631601120966697273744e616d6512036173631206756e697175651301160312046e616d651206696e64657832120a70726f70657274696573150216011208246f776e657249641203617363160112086c6173744e616d6512036173631206756e697175651301160212046e616d651206696e64657833120a70726f706572746965731501160112086c6173744e616d651203617363160212046e616d651206696e64657834120a70726f7065727469657315021601120a2463726561746564417412036173631601120a247570646174656441741203617363160212046e616d651206696e64657835120a70726f7065727469657315011601120a247570646174656441741203617363160212046e616d651206696e64657836120a70726f7065727469657315011601120a246372656174656441741203617363120a70726f706572746965731602120966697273744e616d6516021204747970651206737472696e6712096d61784c656e677468023f12086c6173744e616d6516021204747970651206737472696e6712096d61784c656e677468023f120872657175697265641504120966697273744e616d65120a24637265617465644174120a2475706461746564417412086c6173744e616d6512146164646974696f6e616c50726f7065727469657313000c6e696365446f63756d656e74160412047479706512066f626a656374120a70726f70657274696573160112046e616d6516011204747970651206737472696e67120872657175697265641501120a2463726561746564417412146164646974696f6e616c50726f7065727469657313000e7769746842797465417272617973160512047479706512066f626a6563741207696e64696365731501160212046e616d651206696e64657831120a70726f7065727469657315011601120e6279746541727261794669656c641203617363120a70726f706572746965731602120e6279746541727261794669656c641603120474797065120561727261791209627974654172726179130112086d61784974656d730210120f6964656e7469666965724669656c64160512047479706512056172726179120962797465417272617913011210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e74696669657212086d696e4974656d73022012086d61784974656d730220120872657175697265641501120e6279746541727261794669656c6412146164646974696f6e616c50726f70657274696573130000f22bd29939dc6a8a9b892a942e66e8a6c628309887cededb42aadeb29e21458f01fd000001882425a06101fd000001882425a06101020966697273744e616d6512066d794e616d65086c6173744e616d6512086c6173744e616d65010101411faf90e1f9f909b32e2273e88ba3bdd88db3bebfd6dbb47bb61ebfeb19bd27bbe428eb694bfb0ba96460ebcf40b38464a2242b79878a161d0572b90efc09f65817").expect("expected to decode");

        let document_update = hex::decode("0201017e27f11ff8d2539a71fea96ffb95ea7226599329fecb7d67c2332b2749d0eca00101bc7ad590a5a0d1bcd29c780d981e9ebcc4f00f39caa3591df870f03723e7aba40f696e6465786564446f63756d656e74014b707b7ffe0c6170dc0b1cce6abcfd3db41480de837b8a4038ec2e044c9ec613014b707b7ffe0c6170dc0b1cce6abcfd3db41480de837b8a4038ec2e044c9ec61300000000013468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374017e27f11ff8d2539a71fea96ffb95ea7226599329fecb7d67c2332b2749d0eca0030f696e6465786564446f63756d656e74160512047479706512066f626a6563741207696e64696365731506160312046e616d651206696e64657831120a70726f70657274696573150216011208246f776e6572496412036173631601120966697273744e616d6512036173631206756e697175651301160312046e616d651206696e64657832120a70726f70657274696573150216011208246f776e657249641203617363160112086c6173744e616d6512036173631206756e697175651301160212046e616d651206696e64657833120a70726f706572746965731501160112086c6173744e616d651203617363160212046e616d651206696e64657834120a70726f7065727469657315021601120a2463726561746564417412036173631601120a247570646174656441741203617363160212046e616d651206696e64657835120a70726f7065727469657315011601120a247570646174656441741203617363160212046e616d651206696e64657836120a70726f7065727469657315011601120a246372656174656441741203617363120a70726f706572746965731602120966697273744e616d6516021204747970651206737472696e6712096d61784c656e677468023f12086c6173744e616d6516021204747970651206737472696e6712096d61784c656e677468023f120872657175697265641504120966697273744e616d65120a24637265617465644174120a2475706461746564417412086c6173744e616d6512146164646974696f6e616c50726f7065727469657313000c6e696365446f63756d656e74160412047479706512066f626a656374120a70726f70657274696573160112046e616d6516011204747970651206737472696e67120872657175697265641501120a2463726561746564417412146164646974696f6e616c50726f7065727469657313000e7769746842797465417272617973160512047479706512066f626a6563741207696e64696365731501160212046e616d651206696e64657831120a70726f7065727469657315011601120e6279746541727261794669656c641203617363120a70726f706572746965731602120e6279746541727261794669656c641603120474797065120561727261791209627974654172726179130112086d61784974656d730210120f6964656e7469666965724669656c64160512047479706512056172726179120962797465417272617913011210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e74696669657212086d696e4974656d73022012086d61784974656d730220120872657175697265641501120e6279746541727261794669656c6412146164646974696f6e616c50726f706572746965731300000201fd000001882425afcc01020966697273744e616d65120b757064617465644e616d65086c6173744e616d6512086c6173744e616d65010101411f4f9fc7012bc132a39be07749050795a59fa26598fee49f46deaa597e3f4192aa60f412a7d53a877b80c9469d2ac5797d457f44336b1b655e1ae534ac573ec582").expect("expected to decode");
        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        platform
            .drive
            .create_initial_state_structure(None)
            .expect("expected to create state structure");

        // Init transactional DPP

        let transaction_arc = Arc::new(RwLock::new(Some(platform.drive.grove.start_transaction())));
        let transaction_guard = transaction_arc.read().unwrap();
        let transaction_ref = transaction_guard.as_ref().unwrap();

        let platform_ref = Arc::new(PlatformWithBlockContextRef::from(platform.deref()));
        let dpp_transactional = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::with_transaction(platform_ref, transaction_arc.clone()),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Set block execution context

        let block_info = BlockInfo::default_with_time(1684233625697);
        let block_execution_context = BlockExecutionContext {
            block_state_info: BlockStateInfo {
                block_time_ms: block_info.time_ms,
                ..Default::default()
            },
            ..Default::default()
        };
        platform
            .block_execution_context
            .write()
            .unwrap()
            .replace(block_execution_context);

        // Execute STs

        let validation_result = platform
            .execute_tx(
                identity_create,
                &block_info,
                &dpp_transactional,
                transaction_ref,
            )
            .expect("expected to execute identity_create tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        let validation_result = platform
            .execute_tx(
                data_contract_create,
                &block_info,
                &dpp_transactional,
                transaction_ref,
            )
            .expect("expected to execute data_contract_create tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        let validation_result = platform
            .execute_tx(
                document_create,
                &block_info,
                &dpp_transactional,
                transaction_ref,
            )
            .expect("expected to execute document_create tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        // Commit transaction

        drop(transaction_guard);
        let transaction = transaction_arc.write().unwrap().take().unwrap();

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");

        // Init non-transactional DPP

        let dpp = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::new(Arc::new(PlatformWithBlockContextRef::from(
                platform.deref(),
            ))),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Check txs

        let validation_result = platform
            .check_tx(document_update.as_slice(), &dpp)
            .expect("expected to check tx");

        dbg!(&validation_result.errors);
        //todo fix
        // assert!(validation_result.errors.is_empty());
    }

    #[test]
    fn identity_top_up_check_tx() {
        let identity_top_up = hex::decode("04030000c601018c047719bb8b287e33b788671131b16b1f355d1b3ba6c4917396d0d7bf41e681000000007f1df760772c7ab48c042c01319bd553b7a635936e9a06fa382eb5037638e6ba077a524aa82c6b20e7b8dcadafa46f8ecc59b2dea8c3d6269a24cd5cad74b712ae5a460d11242bd345e168028b3e8442439a63847aa736057a6cd587ae9f7bca1f59f3045566233566142cbca5a7b525085bf96c621ba39f838d6c5c31b116e756753177aa303a8ea712e17ad1ff5dfb0b1504c03d5c225c5cbdb1ee8f6636f0df03000000018c047719bb8b287e33b788671131b16b1f355d1b3ba6c4917396d0d7bf41e681000000006b483045022100d71b565e319a0b85725d1eca250da27d846c6b015e601254e3f8aeb11c0feab60220381c92a46467d6c5270d424b666b989e444e72955f3d5b77d8be9965335b43bd01210222150e3b66410341308b646234bff9c203172c6720b2ecc838c71d94f670066affffffff02e093040000000000166a144cf5fee3ebdce0f51540a3504091c0dccb0f7d343832963b000000001976a914f3b05a1dda565b0013cb9857e708d840bcd47bef88ac00000000003012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac014120d56826c39c07eaea7157b8b717fdcef73fbc99cc680e34f695e0c763d79531691d8ea117cd4623e96a25cbf673e5b1da6e43a96d5bb2a65fe82c2efd4dc2c6dc").expect("expected to decode");

        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        let genesis_time = 0;

        platform
            .create_genesis_state(genesis_time, platform.config.abci.keys.clone().into(), None)
            .expect("expected to create genesis state");

        // Init non-transactional DPP

        let dpp = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::new(Arc::new(PlatformWithBlockContextRef::from(
                platform.deref(),
            ))),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Set block execution context

        let block_info = BlockInfo::default_with_time(1684233625697);
        let block_execution_context = BlockExecutionContext {
            block_state_info: BlockStateInfo {
                block_time_ms: block_info.time_ms,
                ..Default::default()
            },
            ..Default::default()
        };
        platform
            .block_execution_context
            .write()
            .unwrap()
            .replace(block_execution_context);

        // Check txs

        let validation_result = platform
            .check_tx(identity_top_up.as_slice(), &dpp)
            .expect("expected to check tx");

        assert!(validation_result.errors.is_empty());

        // Init transactional DPP

        let transaction_arc = Arc::new(RwLock::new(Some(platform.drive.grove.start_transaction())));
        let transaction_guard = transaction_arc.read().unwrap();
        let transaction_ref = transaction_guard.as_ref().unwrap();

        let platform_ref = Arc::new(PlatformWithBlockContextRef::from(platform.deref()));
        let dpp_transactional = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::with_transaction(platform_ref, transaction_arc.clone()),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Set block execution context

        let block_info = BlockInfo::default_with_time(1684233625697);
        let block_execution_context = BlockExecutionContext {
            block_state_info: BlockStateInfo {
                block_time_ms: block_info.time_ms,
                ..Default::default()
            },
            ..Default::default()
        };
        platform
            .block_execution_context
            .write()
            .unwrap()
            .replace(block_execution_context);

        // Execute txs

        let validation_result = platform
            .execute_tx(
                identity_top_up,
                &BlockInfo::default(),
                &dpp_transactional,
                &transaction_ref,
            )
            .expect("expected to execute identity top up tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        // Commit transaction

        drop(transaction_guard);
        let transaction = transaction_arc.write().unwrap().take().unwrap();

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");
    }

    #[test]
    fn identity_cant_double_top_up() {
        let identity_top_up = hex::decode("04030000c601018c047719bb8b287e33b788671131b16b1f355d1b3ba6c4917396d0d7bf41e681000000007f1df760772c7ab48c042c01319bd553b7a635936e9a06fa382eb5037638e6ba077a524aa82c6b20e7b8dcadafa46f8ecc59b2dea8c3d6269a24cd5cad74b712ae5a460d11242bd345e168028b3e8442439a63847aa736057a6cd587ae9f7bca1f59f3045566233566142cbca5a7b525085bf96c621ba39f838d6c5c31b116e756753177aa303a8ea712e17ad1ff5dfb0b1504c03d5c225c5cbdb1ee8f6636f0df03000000018c047719bb8b287e33b788671131b16b1f355d1b3ba6c4917396d0d7bf41e681000000006b483045022100d71b565e319a0b85725d1eca250da27d846c6b015e601254e3f8aeb11c0feab60220381c92a46467d6c5270d424b666b989e444e72955f3d5b77d8be9965335b43bd01210222150e3b66410341308b646234bff9c203172c6720b2ecc838c71d94f670066affffffff02e093040000000000166a144cf5fee3ebdce0f51540a3504091c0dccb0f7d343832963b000000001976a914f3b05a1dda565b0013cb9857e708d840bcd47bef88ac00000000003012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac014120d56826c39c07eaea7157b8b717fdcef73fbc99cc680e34f695e0c763d79531691d8ea117cd4623e96a25cbf673e5b1da6e43a96d5bb2a65fe82c2efd4dc2c6dc").expect("expected to decode");

        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        let genesis_time = 0;

        platform
            .create_genesis_state(genesis_time, platform.config.abci.keys.clone().into(), None)
            .expect("expected to create genesis state");

        // Init transactional DPP

        let transaction_arc = Arc::new(RwLock::new(Some(platform.drive.grove.start_transaction())));
        let transaction_guard = transaction_arc.read().unwrap();
        let transaction_ref = transaction_guard.as_ref().unwrap();

        let platform_ref = Arc::new(PlatformWithBlockContextRef::from(platform.deref()));
        let dpp_transactional = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::with_transaction(platform_ref, transaction_arc.clone()),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Set block execution context

        let block_info = BlockInfo::default_with_time(1684233625697);
        let block_execution_context = BlockExecutionContext {
            block_state_info: BlockStateInfo {
                block_time_ms: block_info.time_ms,
                ..Default::default()
            },
            ..Default::default()
        };
        platform
            .block_execution_context
            .write()
            .unwrap()
            .replace(block_execution_context);

        // Execute txs

        let validation_result = platform
            .execute_tx(
                identity_top_up.clone(),
                &BlockInfo::default(),
                &dpp_transactional,
                &transaction_ref,
            )
            .expect("expected to execute identity top up tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        // Commit transaction

        drop(transaction_guard);
        let transaction = transaction_arc.write().unwrap().take().unwrap();

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");

        // Init non-transactional DPP

        let dpp = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::new(Arc::new(PlatformWithBlockContextRef::from(
                platform.deref(),
            ))),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Check txs

        let validation_result = platform
            .check_tx(identity_top_up.as_slice(), &dpp)
            .expect("expected to check tx");

        assert!(matches!(
            validation_result.errors.first().expect("expected an error"),
            ConsensusError::BasicError(
                BasicError::IdentityAssetLockTransactionOutPointAlreadyExistsError(_)
            )
        ));
    }

    #[test]
    fn identity_update_doesnt_panic() {
        let identity_top_up = hex::decode("0601054e683919ac96d2e9b099162d845f7540fb1e776eadaca5d84b28235e298d9224020101000002002103a106d1b2fbe4f47c0f9a6cf89b7ed625b5f5972798c9af73475fb179bcb047364120db77e92f250ff1c1114b26355d0a186ab439cbd26ac18ed89c7c63e32b3aea4b339b10feeb2dffd7efa1bdb3e48332a6cdea1951071fb41ef30011a267eb6bbb000000411f56f03e48506fef87be778167838128eb06edc541667c7f010344bb69e54ba1df2c2818db073cc1f7c3966d1d99f0aa1c5e4e1d21959da7f4b89e6c19c123a8b9").expect("expected to decode");

        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        let genesis_time = 0;

        platform
            .create_genesis_state(genesis_time, platform.config.abci.keys.clone().into(), None)
            .expect("expected to create genesis state");

        // Init non-transactional DPP

        let dpp = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::new(Arc::new(PlatformWithBlockContextRef::from(
                platform.deref(),
            ))),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Check txs

        let validation_result = platform
            .check_tx(identity_top_up.as_slice(), &dpp)
            .expect("expected to check tx");

        assert!(matches!(
            validation_result.errors.first().expect("expected an error"),
            ConsensusError::SignatureError(SignatureError::IdentityNotFoundError(_))
        ));
    }

    #[test]
    fn identity_cant_create_with_used_outpoint() {
        let identity_create = hex::decode("03020200000000002102b60f8631519ee2245dfbd7ff107540d378b9d9ca3e8356d1f791703b3027d71b412039feb0213146906bbe7b5c3edd127d567780258f11fd33475af3d2a48679c05e22c446a312d047ed61e94664a787bcac3aae2ab82417232e4a8edae519c525c4010000020021035f06136d7de4240dcfaa885f0c8236d752f906b3e1e8eb157cb816b5a58d6cd141202a6be48392e482a283380fc4679c0147589a898767897e958b3b85abf5703ef734c2041e172794199ecac63ee37f2744e13b6fc9e7d8265b9f2d0ea5a05e9be30000c601013f4fdb109bcd46a4f8eaf72c8bfca482b028e51a8e136519107c7b2c525a5f4100000000de33c4662f152d963eb1ee779cd11891d77890009e4aeabbfce29af45c402b846cc532593e62741065732e91762ac89822b1a6295c36c82863baf59bc0a0ad0b82f3a8f5d956f005dd505ad7e27cbb75c0196fa92f45f504079edb0ba5b64c2e93830f13a63cf9b7cba5a82836a979c1016378237e48859057bd6d2a09c47f78f0aa56f259aba31f205f68adc9f7b5931ea8929806dcd497c23fca262898a163de03000000013f4fdb109bcd46a4f8eaf72c8bfca482b028e51a8e136519107c7b2c525a5f41000000006a47304402207e065274128f612325de5c4e8332e8d3f49891ef2cd1ce4e57da7520484af7290220095d40dfa4490ea336155e82e969ee0f4b59d12126bec6ec7bfe1fee4cbf27d6012102faf354bb3b1487f939e7d7e2e25ec71ad6e3d9804f649e3765ea143d65042bdfffffffff02a086010000000000166a1445399c0424ad422c9dd1f1798ac570666a23f519d8230900000000001976a9143f10001d7dfcdbe673eccecd0cc5dabf208eb78388ac000000000001411f656fd575598f79545029603db7978810292c4b6d923898a09aaeaee2276c32da66f46fbb9ce3230c2485ae803ed98d3bf415fc31f4f1d8f17c44413580c8f7b6563829f5f8f22a6d3dba3143ac38cff96c1fcef4c18b2dbb7da0a2be80757955").expect("expected to decode");

        let platform = TestPlatformBuilder::new()
            .with_config(PlatformConfig::default())
            .build_with_mock_rpc();

        let genesis_time = 0;

        platform
            .create_genesis_state(genesis_time, platform.config.abci.keys.clone().into(), None)
            .expect("expected to create genesis state");

        // Init transactional DPP

        let transaction_arc = Arc::new(RwLock::new(Some(platform.drive.grove.start_transaction())));
        let transaction_guard = transaction_arc.read().unwrap();
        let transaction_ref = transaction_guard.as_ref().unwrap();

        let platform_ref = Arc::new(PlatformWithBlockContextRef::from(platform.deref()));
        let dpp_transactional = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::with_transaction(platform_ref, transaction_arc.clone()),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Set block execution context

        let block_info = BlockInfo::default_with_time(1684233625697);
        let block_execution_context = BlockExecutionContext {
            block_state_info: BlockStateInfo {
                block_time_ms: block_info.time_ms,
                ..Default::default()
            },
            ..Default::default()
        };
        platform
            .block_execution_context
            .write()
            .unwrap()
            .replace(block_execution_context);

        // Execute txs

        let validation_result = platform
            .execute_tx(
                identity_create.clone(),
                &BlockInfo::default(),
                &dpp_transactional,
                &transaction_ref,
            )
            .expect("expected to execute identity create tx");

        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        // Commit transaction

        drop(transaction_guard);
        let transaction = transaction_arc.write().unwrap().take().unwrap();

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");

        // Init non-transactional DPP

        let dpp = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::new(Arc::new(PlatformWithBlockContextRef::from(
                platform.deref(),
            ))),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Check txs

        let validation_result = platform
            .check_tx(identity_create.as_slice(), &dpp)
            .expect("expected to check tx");

        assert!(matches!(
            validation_result.errors.first().expect("expected an error"),
            ConsensusError::StateError(StateError::IdentityAlreadyExistsError(_))
        ));
    }

    #[test]
    fn dpns_owner_top_up() {
        let identity_top_up_first = hex::decode("04030000c60101ba15a200ad72279b3a5e060305660091412babfe6f4bed3c17af33da920baa93000000000460721b022a879c9afcf005fe36136e436f966a0700adf48aa7cf5b45be712e18cf6fd4c5423912f637a99e7cdc2723a42b6ee6d2ea450521a030319bcef330870eb1f0f6918d30e3ab3cfb653b5c087acd52af99efc1c997cd9999401cd323c47858b726f62ef02cf6672a05c7d4b308a59904256d5d1aed723eea5e6666622986750c8011a78b91ddfdccd519af3cb154b90bff6de975b9b3e38c3d9845d5df0300000001ba15a200ad72279b3a5e060305660091412babfe6f4bed3c17af33da920baa93000000006b4830450221008d13e35776dfc5f3dae9427dad1f205e069cac1f3681bc0bca55c434fd7782b002207712496a6eb693527f47f60d39c5cfd5ba99ff9df1ac4e9fe1a5f40c42e5645f0121032eb006412f801abfbfa25e3387cf15583b553d3b89cd30823579232ac021fc33ffffffff02e093040000000000166a14fec95dcf766d232571c45eb86cffda4605e891403832963b000000001976a914cf9e180ad4289a61cc0af0fb376ff7f9a04b8b2888ac00000000003012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac0141207ec67cdad799fcbb072b0dd8734b46e1ee1b3cf07c41ba32f29b745ae0ef89532348fee9ec7fa1e4a6224fcd54d9671452cb0a35b8fd0a5fdaba0447567b84d7").expect("expected to decode");
        let identity_top_up_second = hex::decode("04030000c601018fc4202f1b0a147582ceaefcf622929ae52d9e8a12644d44e59bf198189cbb7300000000c16956695f3911e34b560533ea7b2d1bc2e37e4b9e3c483c27d1038c7072604618cf6fd4c5423912f637a99e7cdc2723a42b6ee6d2ea450521a030319bcef330805526988577953190e7eba053d9ecee1a55e26c2a4fe6783d339cd215ae6dc98e43ed69c86cd601ebdcddccf51349e10e2b09a59a883660aac5ddbe02ea8cee5df6e67115fc040e23ae2316a7c632a662879656a8f61b72daf651898a740a18df03000000018fc4202f1b0a147582ceaefcf622929ae52d9e8a12644d44e59bf198189cbb73000000006b483045022100b421af2c60117484db254c4030a10bddde04a476f0cda484f41357a0074609aa0220328a17b66dff1d8a70ba1c5d4c445c21d45a7548cd09368ccfad3acccbf0f590012102a7dd6bf8188bf9d2aed01dd4c06981e6f8e7df94f79c6dcfa23d56c3787bf228ffffffff02f401000000000000166a14fc7a4f5bdd8d74f31309f1a4d7857484a2ac2ae834210000000000001976a914e13db56a35ee5cef6e40c7608d2fa1372f0af5b188ac00000000003012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac01411f8f5a83ada0b784e352bfc0b7030a4aea966c8cff62c07b9763928added5d72ac0d1cb965e6afcf478be7532ecd42ff87111f3e79d802becf7838aca79acc63ac").expect("expected to decode");
        let dpns_preorder_document = hex::decode("0201013012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac0100a027b57dde6f5ab32aea94cca60a5fbf72a9eb85b7514b3f4a1a23b86563bf74087072656f7264657200e668c659af66aee1e72c186dde7b5b7e0a1d712a09c40d5721f622bf53c5315501e668c659af66aee1e72c186dde7b5b7e0a1d712a09c40d5721f622bf53c5315500000000013468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374013012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac0206646f6d61696e160612047479706512066f626a6563741207696e64696365731503160312046e616d651212706172656e744e616d65416e644c6162656c120a70726f7065727469657315021601121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d6512036173631601120f6e6f726d616c697a65644c6162656c12036173631206756e697175651301160312046e616d65120e646173684964656e746974794964120a70726f7065727469657315011601121c7265636f7264732e64617368556e697175654964656e74697479496412036173631206756e697175651301160212046e616d65120964617368416c696173120a70726f7065727469657315011601121b7265636f7264732e64617368416c6961734964656e7469747949641203617363120a70726f70657274696573160612056c6162656c16051204747970651206737472696e6712077061747465726e122a5e5b612d7a412d5a302d395d5b612d7a412d5a302d392d5d7b302c36317d5b612d7a412d5a302d395d2412096d696e4c656e677468020312096d61784c656e677468023f120b6465736372697074696f6e1219446f6d61696e206c6162656c2e20652e672e2027426f62272e120f6e6f726d616c697a65644c6162656c16051204747970651206737472696e6712077061747465726e12215e5b612d7a302d395d5b612d7a302d392d5d7b302c36317d5b612d7a302d395d2412096d61784c656e677468023f120b6465736372697074696f6e1250446f6d61696e206c6162656c20696e206c6f7765726361736520666f7220636173652d696e73656e73697469766520756e697175656e6573732076616c69646174696f6e2e20652e672e2027626f6227120824636f6d6d656e7412694d75737420626520657175616c20746f20746865206c6162656c20696e206c6f776572636173652e20546869732070726f70657274792077696c6c20626520646570726563617465642064756520746f206361736520696e73656e73697469766520696e6469636573121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d6516061204747970651206737472696e6712077061747465726e12265e247c5e5b612d7a302d395d5b612d7a302d392d5c2e5d7b302c36317d5b612d7a302d395d2412096d696e4c656e677468020012096d61784c656e677468023f120b6465736372697074696f6e125e412066756c6c20706172656e7420646f6d61696e206e616d6520696e206c6f7765726361736520666f7220636173652d696e73656e73697469766520756e697175656e6573732076616c69646174696f6e2e20652e672e20276461736827120824636f6d6d656e74128c4d7573742065697468657220626520657175616c20746f20616e206578697374696e6720646f6d61696e206f7220656d70747920746f20637265617465206120746f70206c6576656c20646f6d61696e2e204f6e6c7920746865206461746120636f6e7472616374206f776e65722063616e2063726561746520746f70206c6576656c20646f6d61696e732e120c7072656f7264657253616c741605120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d730220120b6465736372697074696f6e122253616c74207573656420696e20746865207072656f7264657220646f63756d656e7412077265636f726473160612047479706512066f626a656374120a70726f706572746965731602121464617368556e697175654964656e7469747949641607120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d7302201210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572120b6465736372697074696f6e123e4964656e7469747920494420746f206265207573656420746f2063726561746520746865207072696d617279206e616d6520746865204964656e74697479120824636f6d6d656e7412234d75737420626520657175616c20746f2074686520646f63756d656e74206f776e6572121364617368416c6961734964656e7469747949641607120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d7302201210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572120b6465736372697074696f6e123d4964656e7469747920494420746f206265207573656420746f2063726561746520616c696173206e616d657320666f7220746865204964656e74697479120824636f6d6d656e7412234d75737420626520657175616c20746f2074686520646f63756d656e74206f776e6572120824636f6d6d656e741290436f6e73747261696e742077697468206d617820616e64206d696e2070726f7065727469657320656e737572652074686174206f6e6c79206f6e65206964656e74697479207265636f72642069732075736564202d206569746865722061206064617368556e697175654964656e74697479496460206f722061206064617368416c6961734964656e74697479496460120d6d696e50726f706572746965730201120d6d617850726f70657274696573020112146164646974696f6e616c50726f706572746965731300120e737562646f6d61696e52756c6573160512047479706512066f626a656374120a70726f706572746965731601120f616c6c6f77537562646f6d61696e7316031204747970651207626f6f6c65616e120b6465736372697074696f6e125b54686973206f7074696f6e20646566696e65732077686f2063616e2063726561746520737562646f6d61696e733a2074727565202d20616e796f6e653b2066616c7365202d206f6e6c792074686520646f6d61696e206f776e6572120824636f6d6d656e74124f4f6e6c792074686520646f6d61696e206f776e657220697320616c6c6f77656420746f2063726561746520737562646f6d61696e7320666f72206e6f6e20746f702d6c6576656c20646f6d61696e73120b6465736372697074696f6e1242537562646f6d61696e2072756c657320616c6c6f7720646f6d61696e206f776e65727320746f20646566696e652072756c657320666f7220737562646f6d61696e7312146164646974696f6e616c50726f706572746965731300120872657175697265641501120f616c6c6f77537562646f6d61696e7312087265717569726564150612056c6162656c120f6e6f726d616c697a65644c6162656c121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d65120c7072656f7264657253616c7412077265636f726473120e737562646f6d61696e52756c657312146164646974696f6e616c50726f706572746965731300120824636f6d6d656e7412fb0137496e206f7264657220746f207265676973746572206120646f6d61696e20796f75206e65656420746f206372656174652061207072656f726465722e20546865207072656f726465722073746570206973206e656564656420746f2070726576656e74206d616e2d696e2d7468652d6d6964646c652061747461636b732e206e6f726d616c697a65644c6162656c202b20272e27202b206e6f726d616c697a6564506172656e74446f6d61696e206d757374206e6f74206265206c6f6e676572207468616e20323533206368617273206c656e67746820617320646566696e65642062792052464320313033352e20446f6d61696e20646f63756d656e74732061726520696d6d757461626c653a206d6f64696669636174696f6e20616e642064656c6574696f6e206172652072657374726963746564087072656f72646572160612047479706512066f626a6563741207696e64696365731501160312046e616d65120a73616c74656448617368120a70726f7065727469657315011601121073616c746564446f6d61696e4861736812036173631206756e697175651301120a70726f706572746965731601121073616c746564446f6d61696e486173681605120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d730220120b6465736372697074696f6e1259446f75626c65207368612d323536206f662074686520636f6e636174656e6174696f6e206f66206120333220627974652072616e646f6d2073616c7420616e642061206e6f726d616c697a656420646f6d61696e206e616d65120872657175697265641501121073616c746564446f6d61696e4861736812146164646974696f6e616c50726f706572746965731300120824636f6d6d656e74124a5072656f7264657220646f63756d656e74732061726520696d6d757461626c653a206d6f64696669636174696f6e20616e642064656c6574696f6e20617265207265737472696374656400d802aae1a3f029af9ef35dd1f4388fab0f04aa459131fbf4561101302010d71c000001011073616c746564446f6d61696e486173680a2012e6b5cacd027fab24ddfb4755e76f8cf840072b44c8e850fec213472bdd217d010101411f49a8787acb91c42e6b32fbceae5191505398cdfe3a2c730cf14f7b8002f3c8bb4dbb0cf4db23f858f34e69b145f59d10056782dc404c1005349a34bb04f7cce6").expect("expected to decode");
        let dpns_domain_document = hex::decode("0201013012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac01005cfbbbd4226c16dafbbf5c614701bc2502603c5662a661ef38a8ae7b14ac6ead06646f6d61696e00e668c659af66aee1e72c186dde7b5b7e0a1d712a09c40d5721f622bf53c5315501e668c659af66aee1e72c186dde7b5b7e0a1d712a09c40d5721f622bf53c5315500000000013468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374013012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac0206646f6d61696e160612047479706512066f626a6563741207696e64696365731503160312046e616d651212706172656e744e616d65416e644c6162656c120a70726f7065727469657315021601121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d6512036173631601120f6e6f726d616c697a65644c6162656c12036173631206756e697175651301160312046e616d65120e646173684964656e746974794964120a70726f7065727469657315011601121c7265636f7264732e64617368556e697175654964656e74697479496412036173631206756e697175651301160212046e616d65120964617368416c696173120a70726f7065727469657315011601121b7265636f7264732e64617368416c6961734964656e7469747949641203617363120a70726f70657274696573160612056c6162656c16051204747970651206737472696e6712077061747465726e122a5e5b612d7a412d5a302d395d5b612d7a412d5a302d392d5d7b302c36317d5b612d7a412d5a302d395d2412096d696e4c656e677468020312096d61784c656e677468023f120b6465736372697074696f6e1219446f6d61696e206c6162656c2e20652e672e2027426f62272e120f6e6f726d616c697a65644c6162656c16051204747970651206737472696e6712077061747465726e12215e5b612d7a302d395d5b612d7a302d392d5d7b302c36317d5b612d7a302d395d2412096d61784c656e677468023f120b6465736372697074696f6e1250446f6d61696e206c6162656c20696e206c6f7765726361736520666f7220636173652d696e73656e73697469766520756e697175656e6573732076616c69646174696f6e2e20652e672e2027626f6227120824636f6d6d656e7412694d75737420626520657175616c20746f20746865206c6162656c20696e206c6f776572636173652e20546869732070726f70657274792077696c6c20626520646570726563617465642064756520746f206361736520696e73656e73697469766520696e6469636573121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d6516061204747970651206737472696e6712077061747465726e12265e247c5e5b612d7a302d395d5b612d7a302d392d5c2e5d7b302c36317d5b612d7a302d395d2412096d696e4c656e677468020012096d61784c656e677468023f120b6465736372697074696f6e125e412066756c6c20706172656e7420646f6d61696e206e616d6520696e206c6f7765726361736520666f7220636173652d696e73656e73697469766520756e697175656e6573732076616c69646174696f6e2e20652e672e20276461736827120824636f6d6d656e74128c4d7573742065697468657220626520657175616c20746f20616e206578697374696e6720646f6d61696e206f7220656d70747920746f20637265617465206120746f70206c6576656c20646f6d61696e2e204f6e6c7920746865206461746120636f6e7472616374206f776e65722063616e2063726561746520746f70206c6576656c20646f6d61696e732e120c7072656f7264657253616c741605120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d730220120b6465736372697074696f6e122253616c74207573656420696e20746865207072656f7264657220646f63756d656e7412077265636f726473160612047479706512066f626a656374120a70726f706572746965731602121464617368556e697175654964656e7469747949641607120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d7302201210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572120b6465736372697074696f6e123e4964656e7469747920494420746f206265207573656420746f2063726561746520746865207072696d617279206e616d6520746865204964656e74697479120824636f6d6d656e7412234d75737420626520657175616c20746f2074686520646f63756d656e74206f776e6572121364617368416c6961734964656e7469747949641607120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d7302201210636f6e74656e744d656469615479706512216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572120b6465736372697074696f6e123d4964656e7469747920494420746f206265207573656420746f2063726561746520616c696173206e616d657320666f7220746865204964656e74697479120824636f6d6d656e7412234d75737420626520657175616c20746f2074686520646f63756d656e74206f776e6572120824636f6d6d656e741290436f6e73747261696e742077697468206d617820616e64206d696e2070726f7065727469657320656e737572652074686174206f6e6c79206f6e65206964656e74697479207265636f72642069732075736564202d206569746865722061206064617368556e697175654964656e74697479496460206f722061206064617368416c6961734964656e74697479496460120d6d696e50726f706572746965730201120d6d617850726f70657274696573020112146164646974696f6e616c50726f706572746965731300120e737562646f6d61696e52756c6573160512047479706512066f626a656374120a70726f706572746965731601120f616c6c6f77537562646f6d61696e7316031204747970651207626f6f6c65616e120b6465736372697074696f6e125b54686973206f7074696f6e20646566696e65732077686f2063616e2063726561746520737562646f6d61696e733a2074727565202d20616e796f6e653b2066616c7365202d206f6e6c792074686520646f6d61696e206f776e6572120824636f6d6d656e74124f4f6e6c792074686520646f6d61696e206f776e657220697320616c6c6f77656420746f2063726561746520737562646f6d61696e7320666f72206e6f6e20746f702d6c6576656c20646f6d61696e73120b6465736372697074696f6e1242537562646f6d61696e2072756c657320616c6c6f7720646f6d61696e206f776e65727320746f20646566696e652072756c657320666f7220737562646f6d61696e7312146164646974696f6e616c50726f706572746965731300120872657175697265641501120f616c6c6f77537562646f6d61696e7312087265717569726564150612056c6162656c120f6e6f726d616c697a65644c6162656c121a6e6f726d616c697a6564506172656e74446f6d61696e4e616d65120c7072656f7264657253616c7412077265636f726473120e737562646f6d61696e52756c657312146164646974696f6e616c50726f706572746965731300120824636f6d6d656e7412fb0137496e206f7264657220746f207265676973746572206120646f6d61696e20796f75206e65656420746f206372656174652061207072656f726465722e20546865207072656f726465722073746570206973206e656564656420746f2070726576656e74206d616e2d696e2d7468652d6d6964646c652061747461636b732e206e6f726d616c697a65644c6162656c202b20272e27202b206e6f726d616c697a6564506172656e74446f6d61696e206d757374206e6f74206265206c6f6e676572207468616e20323533206368617273206c656e67746820617320646566696e65642062792052464320313033352e20446f6d61696e20646f63756d656e74732061726520696d6d757461626c653a206d6f64696669636174696f6e20616e642064656c6574696f6e206172652072657374726963746564087072656f72646572160612047479706512066f626a6563741207696e64696365731501160312046e616d65120a73616c74656448617368120a70726f7065727469657315011601121073616c746564446f6d61696e4861736812036173631206756e697175651301120a70726f706572746965731601121073616c746564446f6d61696e486173681605120474797065120561727261791209627974654172726179130112086d696e4974656d73022012086d61784974656d730220120b6465736372697074696f6e1259446f75626c65207368612d323536206f662074686520636f6e636174656e6174696f6e206f66206120333220627974652072616e646f6d2073616c7420616e642061206e6f726d616c697a656420646f6d61696e206e616d65120872657175697265641501121073616c746564446f6d61696e4861736812146164646974696f6e616c50726f706572746965731300120824636f6d6d656e74124a5072656f7264657220646f63756d656e74732061726520696d6d757461626c653a206d6f64696669636174696f6e20616e642064656c6574696f6e206172652072657374726963746564000e3fc8950e82fb0ccbd0ed86e593bab85212bcf9bc012f92980bb4aa2532aafb00000106056c6162656c121437396661333531626562343362303736653538380f6e6f726d616c697a65644c6162656c121437396661333531626562343362303736653538381a6e6f726d616c697a6564506172656e74446f6d61696e4e616d6512000c7072656f7264657253616c740a2004e085d9967c0081bb616ef2df8d993c9d315853ef49dee44ea5367e604870f4077265636f7264731601121364617368416c6961734964656e746974794964103012c19b98ec0033addb36cd64b7f510670f2a351a4304b5f6994144286efdac0e737562646f6d61696e52756c65731601120f616c6c6f77537562646f6d61696e731301010101411fb015b8f9a09a4b9199fd1eea97ff210be521794c4d42b08fe2369fc298977acf3887ac514c4688ae1fd1d5277c7164e3a4758e4d4c3e91759e81a8d4261140b4").expect("expected to decode");

        let mut config = PlatformConfig::default();

        config.abci.keys.dpns_master_public_key =
            base64::decode("A08spLsoYnvWMLhBiaK+G4pybyu+5NmJefh3MdMfqkg/")
                .expect("expected to decode");
        config.abci.keys.dpns_second_public_key =
            base64::decode("A47zP5s3jmW7dPBzmzEXieUNnM34YPtDR1FpbqMaAVFR")
                .expect("expected to decode");

        let platform = TestPlatformBuilder::new()
            .with_config(config)
            .build_with_mock_rpc();

        let genesis_time = 0;

        platform
            .create_genesis_state(genesis_time, platform.config.abci.keys.clone().into(), None)
            .expect("expected to create genesis state");

        // Init transactional DPP

        let transaction_arc = Arc::new(RwLock::new(Some(platform.drive.grove.start_transaction())));
        let transaction_guard = transaction_arc.read().unwrap();
        let transaction_ref = transaction_guard.as_ref().unwrap();

        let platform_ref = Arc::new(PlatformWithBlockContextRef::from(platform.deref()));
        let dpp_transactional = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::with_transaction(platform_ref, transaction_arc.clone()),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Set block execution context

        let block_info = BlockInfo::default_with_time(1684233625697);
        let block_execution_context = BlockExecutionContext {
            block_state_info: BlockStateInfo {
                block_time_ms: block_info.time_ms,
                ..Default::default()
            },
            ..Default::default()
        };
        platform
            .block_execution_context
            .write()
            .unwrap()
            .replace(block_execution_context);

        // Execute STs

        let validation_result = platform
            .execute_tx(
                identity_top_up_first,
                &BlockInfo::default(),
                &dpp_transactional,
                &transaction_ref,
            )
            .expect("expected to execute identity_top_up_first tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        let validation_result = platform
            .execute_tx(
                identity_top_up_second,
                &BlockInfo::default(),
                &dpp_transactional,
                &transaction_ref,
            )
            .expect("expected to execute identity_top_up_second tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        let validation_result = platform
            .execute_tx(
                dpns_preorder_document,
                &BlockInfo::default(),
                &dpp_transactional,
                &transaction_ref,
            )
            .expect("expected to execute identity_create tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        // Commit transaction

        drop(transaction_guard);
        let transaction = transaction_arc.write().unwrap().take().unwrap();

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");

        // Init non-transactional DPP

        let dpp = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::new(Arc::new(PlatformWithBlockContextRef::from(
                platform.deref(),
            ))),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Check txs

        let validation_result = platform
            .check_tx(dpns_domain_document.as_slice(), &dpp)
            .expect("expected to check tx");

        assert!(validation_result.errors.is_empty());

        // Execute txs

        transaction_arc
            .write()
            .unwrap()
            .replace(platform.drive.grove.start_transaction());
        let transaction_guard = transaction_arc.read().unwrap();
        let transaction_ref = transaction_guard.as_ref().unwrap();

        let validation_result = platform
            .execute_tx(
                dpns_domain_document,
                &BlockInfo::default(),
                &dpp_transactional,
                &transaction_ref,
            )
            .expect("expected to execute identity top up tx");
        assert!(matches!(validation_result, SuccessfulPaidExecution(..)));

        // Commit transaction

        drop(transaction_guard);
        let transaction = transaction_arc.write().unwrap().take().unwrap();

        platform
            .drive
            .grove
            .commit_transaction(transaction)
            .unwrap()
            .expect("expected to commit transaction");
    }

    #[test]
    fn identity_update_with_non_master_key_check_tx() {
        let mut config = PlatformConfig::default();

        let mut rng = StdRng::seed_from_u64(1);

        let secp = Secp256k1::new();

        let master_key_pair = KeyPair::new(&secp, &mut rng);

        let master_secret_key = master_key_pair.secret_key();

        let master_public_key = master_key_pair.public_key();

        config.abci.keys.dpns_master_public_key = master_public_key.serialize().to_vec();

        let high_key_pair = KeyPair::new(&secp, &mut rng);

        let high_secret_key = high_key_pair.secret_key();

        let high_public_key = high_key_pair.public_key();

        config.abci.keys.dpns_second_public_key = high_public_key.serialize().to_vec();

        let platform = TestPlatformBuilder::new()
            .with_config(config)
            .build_with_mock_rpc();

        let genesis_time = 0;

        platform
            .create_genesis_state(genesis_time, platform.config.abci.keys.clone().into(), None)
            .expect("expected to create genesis state");

        // Init non-transactional DPP

        let dpp = DashPlatformProtocol::new(
            DPPOptions::default(),
            DPPStateRepository::new(Arc::new(PlatformWithBlockContextRef::from(
                platform.deref(),
            ))),
            NativeBlsModule::default(),
        )
        .expect("should create dpp");

        // Set block execution context

        let block_info = BlockInfo::default_with_time(1684233625697);
        let block_execution_context = BlockExecutionContext {
            block_state_info: BlockStateInfo {
                block_time_ms: block_info.time_ms,
                ..Default::default()
            },
            ..Default::default()
        };
        platform
            .block_execution_context
            .write()
            .unwrap()
            .replace(block_execution_context);

        // Check txs

        let new_key_pair = KeyPair::new(&secp, &mut rng);

        let mut new_key = IdentityPublicKeyInCreation {
            id: 2,
            purpose: Purpose::AUTHENTICATION,
            security_level: SecurityLevel::HIGH,
            key_type: KeyType::ECDSA_SECP256K1,
            read_only: false,
            data: new_key_pair.public_key().serialize().to_vec().into(),
            signature: Default::default(),
        };

        let signable_bytes = new_key
            .signable_bytes()
            .expect("expected to get signable bytes");
        let secret = new_key_pair.secret_key();
        let signature =
            signer::sign(&signable_bytes, &secret.secret_bytes()).expect("expected to sign");

        new_key.signature = signature.to_vec().into();

        let mut update_transition = IdentityUpdateTransition {
            protocol_version: LATEST_VERSION,
            transition_type: StateTransitionType::IdentityUpdate,
            identity_id: dpns_contract::OWNER_ID_BYTES.into(),
            revision: 1,
            add_public_keys: vec![new_key],
            disable_public_keys: vec![],
            public_keys_disabled_at: None,
            signature_public_key_id: 1,
            signature: Default::default(),
        };

        let signature = signer::sign(
            &update_transition
                .signable_bytes()
                .expect("expected signable bytes"),
            &high_secret_key.secret_bytes(),
        )
        .expect("expected to sign");

        update_transition.signature = signature.to_vec().into();

        let transition: StateTransition = update_transition.into();

        let update_transition_bytes = transition.serialize().expect("expected to serialize");

        let validation_result = platform
            .check_tx(update_transition_bytes.as_slice(), &dpp)
            .expect("expected to execute identity top up tx");

        // Only master keys can sign an update

        validation_result.errors.first().expect("expected an error");
    }
}
