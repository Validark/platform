const {
  tendermint: {
    abci: {
      ResponseFinalizeBlock,
      RequestProcessProposal,
    },
  },
} = require('@dashevo/abci/types');

const Long = require('long');

// TODO: should we load it from other place?
const { hash } = require('@dashevo/dpp/lib/util/hash');
const getDataContractFixture = require('@dashevo/wasm-dpp/lib/test/fixtures/getDataContractFixture');

const finalizeBlockHandlerFactory = require('../../../../lib/abci/handlers/finalizeBlockHandlerFactory');
const LoggerMock = require('../../../../lib/test/mock/LoggerMock');
const BlockExecutionContextMock = require('../../../../lib/test/mock/BlockExecutionContextMock');
const GroveDBStoreMock = require('../../../../lib/test/mock/GroveDBStoreMock');
const BlockExecutionContextRepositoryMock = require('../../../../lib/test/mock/BlockExecutionContextRepositoryMock');

describe('finalizeBlockHandlerFactory', () => {
  let finalizeBlockHandler;
  let executionTimerMock;
  let latestBlockExecutionContextMock;
  let loggerMock;
  let requestMock;
  let appHash;
  let groveDBStoreMock;
  let blockExecutionContextRepositoryMock;
  let dataContract;
  let proposalBlockExecutionContextMock;
  let round;
  let block;
  let processProposalMock;
  let broadcastWithdrawalTransactions;
  let createContextLoggerMock;

  beforeEach(async function beforeEach() {
    round = 0;
    appHash = Buffer.alloc(0);
    proposalBlockExecutionContextMock = new BlockExecutionContextMock(this.sinon);

    latestBlockExecutionContextMock = new BlockExecutionContextMock(this.sinon);
    loggerMock = new LoggerMock(this.sinon);
    executionTimerMock = {
      clearTimer: this.sinon.stub(),
      startTimer: this.sinon.stub(),
      stopTimer: this.sinon.stub(),
    };

    const commit = {};

    const height = new Long(42);

    const time = {
      seconds: Math.ceil(new Date().getTime() / 1000),
    };

    const coreChainLockedHeight = 10;

    block = {
      header: {
        time,
        version: {
          app: Long.fromInt(1),
        },
        proposerProTxHash: Uint8Array.from([1, 2, 3, 4]),
        coreChainLockedHeight,
      },
      data: {
        txs: new Array(3).fill(Buffer.alloc(5, 0)),
      },
    };

    requestMock = {
      commit,
      height,
      time,
      coreChainLockedHeight,
      round,
      block,
    };

    dataContract = await getDataContractFixture();

    proposalBlockExecutionContextMock.getHeight.returns(new Long(42));
    proposalBlockExecutionContextMock.getRound.returns(round);
    proposalBlockExecutionContextMock.getDataContracts.returns([dataContract]);
    proposalBlockExecutionContextMock.getEpochInfo.returns({
      currentEpochIndex: 1,
    });
    proposalBlockExecutionContextMock.getTimeMs.returns((new Date()).getTime());

    groveDBStoreMock = new GroveDBStoreMock(this.sinon);
    groveDBStoreMock.getRootHash.resolves(appHash);

    blockExecutionContextRepositoryMock = new BlockExecutionContextRepositoryMock(
      this.sinon,
    );

    processProposalMock = this.sinon.stub();
    createContextLoggerMock = this.sinon.stub().returns(loggerMock);

    broadcastWithdrawalTransactions = this.sinon.stub();

    finalizeBlockHandler = finalizeBlockHandlerFactory(
      groveDBStoreMock,
      blockExecutionContextRepositoryMock,
      loggerMock,
      executionTimerMock,
      latestBlockExecutionContextMock,
      proposalBlockExecutionContextMock,
      processProposalMock,
      broadcastWithdrawalTransactions,
      createContextLoggerMock,
    );
  });

  it('should commit db transactions, create document dbs and return ResponseFinalizeBlock', async () => {
    const result = await finalizeBlockHandler(requestMock);

    expect(result).to.be.an.instanceOf(ResponseFinalizeBlock);

    expect(executionTimerMock.stopTimer).to.be.calledOnceWithExactly('blockExecution');

    expect(proposalBlockExecutionContextMock.reset).to.be.calledOnce();

    expect(blockExecutionContextRepositoryMock.store).to.be.calledOnceWithExactly(
      proposalBlockExecutionContextMock,
      {
        useTransaction: true,
      },
    );

    expect(groveDBStoreMock.commitTransaction).to.be.calledOnceWithExactly();

    expect(latestBlockExecutionContextMock.populate).to.be.calledOnce();
    expect(processProposalMock).to.be.not.called();

    expect(broadcastWithdrawalTransactions).to.have.been.calledOnceWith(
      proposalBlockExecutionContextMock,
      undefined,
      undefined,
    );

    expect(createContextLoggerMock).to.be.calledOnceWithExactly(
      loggerMock, {
        height: '42',
        round,
        abciMethod: 'finalizeBlock',
      },
    );
  });

  it('should send withdrawal transaction if vote extensions are present', async () => {
    const [txOneBytes, txTwoBytes] = [
      Buffer.alloc(32, 0),
      Buffer.alloc(32, 1),
    ];

    proposalBlockExecutionContextMock.getWithdrawalTransactionsMap.returns({
      [hash(txOneBytes).toString('hex')]: txOneBytes,
      [hash(txTwoBytes).toString('hex')]: txTwoBytes,
    });

    const thresholdVoteExtensions = [
      {
        extension: hash(txOneBytes),
        signature: Buffer.alloc(96, 3),
      },
      {
        extension: hash(txTwoBytes),
        signature: Buffer.alloc(96, 4),
      },
    ];

    requestMock.commit = { thresholdVoteExtensions };

    await finalizeBlockHandler(requestMock);

    expect(processProposalMock).to.be.not.called();
    expect(createContextLoggerMock).to.be.calledOnceWithExactly(
      loggerMock, {
        height: '42',
        round,
        abciMethod: 'finalizeBlock',
      },
    );
  });

  it('should call processProposal if round is not equal to execution context', async () => {
    proposalBlockExecutionContextMock.getRound.returns(round + 1);

    const result = await finalizeBlockHandler(requestMock);

    expect(result).to.be.an.instanceOf(ResponseFinalizeBlock);

    const processProposalRequest = new RequestProcessProposal({
      height: requestMock.height,
      txs: block.data.txs,
      coreChainLockedHeight: block.header.coreChainLockedHeight,
      version: block.header.version,
      proposedLastCommit: requestMock.commit,
      time: block.header.time,
      proposerProTxHash: block.header.proposerProTxHash,
      round,
    });

    expect(processProposalMock).to.be.calledOnceWithExactly(processProposalRequest, loggerMock);
    expect(createContextLoggerMock).to.be.calledOnceWithExactly(
      loggerMock, {
        height: '42',
        round,
        abciMethod: 'finalizeBlock',
      },
    );
  });
});
