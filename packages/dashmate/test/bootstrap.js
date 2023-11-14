import sinon from 'sinon';
import { expect, use } from 'chai';
import sinonChai from 'sinon-chai';
import dirtyChai from 'dirty-chai';
import chaiAsPromised from 'chai-as-promised';
import { default as loadWasmDpp } from '@dashevo/wasm-dpp';

use(sinonChai);
use(chaiAsPromised);
use(dirtyChai);

process.env.NODE_ENV = 'test';

exports.mochaHooks = {
  beforeAll: loadWasmDpp,
  beforeEach() {
    if (!this.sinon) {
      this.sinon = sinon.createSandbox();
    } else {
      this.sinon.restore();
    }
  },

  afterEach() {
    this.sinon.restore();
  },
};

global.expect = expect;
