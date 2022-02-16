const Long = require('long');

const {
  tendermint: {
    abci: {
      ResponseInitChain,
      ValidatorSetUpdate,
    },
  },
} = require('@dashevo/abci/types');

const initChainHandlerFactory = require('../../../../lib/abci/handlers/initChainHandlerFactory');
const LoggerMock = require('../../../../lib/test/mock/LoggerMock');

describe('initChainHandlerFactory', () => {
  let initChainHandler;
  let updateSimplifiedMasternodeListMock;
  let initialCoreChainLockedHeight;
  let validatorSetMock;
  let createValidatorSetUpdateMock;
  let loggerMock;
  let validatorSetUpdate;
  let synchronizeMasternodeIdentitiesMock;
  let registerSystemDataContractsMock;
  let createInitialStateStructureMock;
  let groveDBStoreMock;
  let appHashFixture;

  beforeEach(function beforeEach() {
    initialCoreChainLockedHeight = 1;

    appHashFixture = Buffer.alloc(0);

    updateSimplifiedMasternodeListMock = this.sinon.stub();

    const quorumHash = Buffer.alloc(64).fill(1).toString('hex');
    validatorSetMock = {
      initialize: this.sinon.stub(),
      getQuorum: this.sinon.stub().returns({
        quorumHash,
      }),
    };

    validatorSetUpdate = new ValidatorSetUpdate();

    createValidatorSetUpdateMock = this.sinon.stub().returns(validatorSetUpdate);
    synchronizeMasternodeIdentitiesMock = this.sinon.stub();

    loggerMock = new LoggerMock(this.sinon);

    registerSystemDataContractsMock = this.sinon.stub();
    createInitialStateStructureMock = this.sinon.stub();

    groveDBStoreMock = {
      startTransaction: this.sinon.stub(),
      commitTransaction: this.sinon.stub(),
      getRootHash: this.sinon.stub().resolves(appHashFixture),
    };

    initChainHandler = initChainHandlerFactory(
      updateSimplifiedMasternodeListMock,
      initialCoreChainLockedHeight,
      validatorSetMock,
      createValidatorSetUpdateMock,
      synchronizeMasternodeIdentitiesMock,
      loggerMock,
      createInitialStateStructureMock,
      registerSystemDataContractsMock,
      groveDBStoreMock,
    );
  });

  it('should update height, start transactions and return ResponseBeginBlock', async () => {
    const request = {
      initialHeight: Long.fromInt(1),
      chainId: 'test',
      time: {
        seconds: Long.fromInt((new Date()).getTime() / 1000),
      },
    };

    const response = await initChainHandler(request);

    expect(response).to.be.an.instanceOf(ResponseInitChain);
    expect(response.validatorSetUpdate).to.be.equal(validatorSetUpdate);
    expect(response.initialCoreHeight).to.be.equal(initialCoreChainLockedHeight);
    expect(response.appHash).to.deep.equal(appHashFixture);

    expect(updateSimplifiedMasternodeListMock).to.be.calledOnceWithExactly(
      initialCoreChainLockedHeight,
      {
        logger: loggerMock,
      },
    );

    expect(groveDBStoreMock.startTransaction).to.be.calledOnce();

    expect(createInitialStateStructureMock).to.be.calledOnce();

    expect(registerSystemDataContractsMock).to.be.calledOnceWithExactly(loggerMock, request.time);

    expect(synchronizeMasternodeIdentitiesMock).to.be.calledOnceWithExactly(
      initialCoreChainLockedHeight,
    );

    expect(groveDBStoreMock.commitTransaction).to.be.calledOnce();

    expect(groveDBStoreMock.getRootHash).to.be.calledOnce();

    expect(validatorSetMock.initialize).to.be.calledOnceWithExactly(
      initialCoreChainLockedHeight,
    );

    expect(validatorSetMock.getQuorum).to.be.calledOnce();

    expect(createValidatorSetUpdateMock).to.be.calledOnceWithExactly(validatorSetMock);
  });
});
