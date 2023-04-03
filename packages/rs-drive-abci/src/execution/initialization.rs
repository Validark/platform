use crate::error::execution::ExecutionError;
use crate::error::Error;
use crate::platform::Platform;
use dpp::identity::TimestampMillis;
use tenderdash_abci::proto::abci::RequestInitChain;
use tenderdash_abci::proto::serializers::timestamp::ToMilis;

impl<C> Platform<C> {
    /// Initialize the chain
    pub fn init_chain(&self, request: RequestInitChain) -> Result<(), Error> {
        let transaction = self.drive.grove.start_transaction();
        let genesis_time = request
            .time
            .ok_or(Error::Execution(ExecutionError::InitializationError(
                "genesis time is required in init chain",
            )))?
            .to_milis() as TimestampMillis;

        self.create_genesis_state(
            genesis_time,
            self.config.abci.keys.clone().into(),
            Some(&transaction),
        )?;

        self.drive
            .commit_transaction(transaction)
            .map_err(Error::Drive)
    }
}
