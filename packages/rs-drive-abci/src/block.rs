// MIT LICENSE
//
// Copyright (c) 2021 Dash Core Group
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//

use dpp::util::vec::vec_to_array;
use drive::grovedb::{Transaction, TransactionArg};
use tenderdash_abci::proto::abci as proto;
use tenderdash_abci::proto::serializers::timestamp::ToMilis;

use crate::abci::messages::BlockBeginRequest;
use crate::execution::fee_pools::epoch::EpochInfo;

/// Block info
pub struct BlockStateInfo {
    /// Block height
    pub block_height: u64,
    /// Block time in ms
    pub block_time_ms: u64,
    /// Previous block time in ms
    pub previous_block_time_ms: Option<u64>,
    /// Block proposer's proTxHash
    pub proposer_pro_tx_hash: [u8; 32],
    /// Core chain locked height
    pub core_chain_locked_height: u32,
}

impl BlockStateInfo {
    /// Given a `BlockBeginRequest` return `BlockInfo`
    #[deprecated = "use from_prepare_proposal_request and from_process_proposal_request"]
    pub fn from_block_begin_request(block_begin_request: &BlockBeginRequest) -> BlockStateInfo {
        BlockStateInfo {
            block_height: block_begin_request.block_height,
            block_time_ms: block_begin_request.block_time_ms,
            previous_block_time_ms: block_begin_request.previous_block_time_ms,
            proposer_pro_tx_hash: block_begin_request.proposer_pro_tx_hash,
            core_chain_locked_height: block_begin_request.core_chain_locked_height,
        }
    }
    /// Generate block state info based on Prepare Proposal request
    pub fn from_prepare_proposal_request(
        request: &proto::RequestPrepareProposal,
    ) -> BlockStateInfo {
        BlockStateInfo {
            block_height: request.height as u64,
            block_time_ms: request.time.clone().unwrap().to_milis(),
            previous_block_time_ms: None, // TODO: implement properly
            //<dyn Into<[u8; 32]>>::into()
            proposer_pro_tx_hash: vec_to_array(&request.proposer_pro_tx_hash)
                .expect("invalid proposer protxhash"),
            core_chain_locked_height: request.core_chain_locked_height,
        }
    }
}
/// Block execution context
pub struct BlockExecutionContext<'a> {
    /// Current Transaction
    pub current_transaction: Transaction<'a>,
    /// Block info
    pub block_info: BlockStateInfo,
    /// Epoch info
    pub epoch_info: EpochInfo,
    /// Total hpmn count
    pub hpmn_count: u32,
}
