const cbor = require('cbor');

const BlockExecutionContext = require('./BlockExecutionContext');

class BlockExecutionContextRepository {
  /**
   *
   * @param {GroveDBStore} groveDBStore
   * @param {WebAssembly.Instance} dppWasm
   */
  constructor(groveDBStore, dppWasm) {
    this.db = groveDBStore;
    this.dppWasm = dppWasm;
  }

  /**
   * Store block execution context
   *
   * @param {BlockExecutionContext} blockExecutionContext
   * @param {Object} [options]
   * @param {boolean} [options.useTransaction=false]
   * @return {this}
   */
  async store(blockExecutionContext, options = {}) {
    await this.db.putAux(
      BlockExecutionContextRepository.EXTERNAL_STORE_KEY_NAME,
      await cbor.encodeAsync(blockExecutionContext.toObject({
        skipContextLogger: true,
        skipPrepareProposalResult: true,
      })),
      options,
    );

    return this;
  }

  /**
   * Fetch block execution stack
   *
   * @param {Object} [options]
   * @param {boolean} [options.useTransaction=false]
   *
   * @return {BlockExecutionContext}
   */
  async fetch(options = {}) {
    const blockExecutionContextEncodedResult = await this.db.getAux(
      BlockExecutionContextRepository.EXTERNAL_STORE_KEY_NAME,
      options,
    );

    const blockExecutionContextEncoded = blockExecutionContextEncodedResult.getValue();

    const blockExecutionContext = new BlockExecutionContext(this.dppWasm);

    if (!blockExecutionContextEncoded) {
      return blockExecutionContext;
    }

    const rawBlockExecutionContext = cbor.decode(blockExecutionContextEncoded);

    const context = new BlockExecutionContext(this.dppWasm);

    context.fromObject(rawBlockExecutionContext);

    return context;
  }
}

BlockExecutionContextRepository.EXTERNAL_STORE_KEY_NAME = Buffer.from('blockExecutionContext');

module.exports = BlockExecutionContextRepository;
