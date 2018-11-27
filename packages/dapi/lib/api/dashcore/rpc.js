const RpcClient = require('@dashevo/dashd-rpc');
const constants = require('./constants');
const config = require('../../config');

const client = new RpcClient(config.dashcore.rpc);

/**
 *  Layer 1 endpoints
 *  These functions represent endpoints on the transactional layer
 *  and can be requested from any random DAPI node.
 *  Once a DAPI-client is assigned to a quorum it should exclude its quorum nodes
 *  from the set of nodes serving L1 endpoints for privacy reasons
 */

const generate = amount => new Promise((resolve, reject) => { // not exist?
  client.generate(amount, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

const getBestBlockHeight = () => new Promise((resolve, reject) => {
  client.getblockcount((err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

const getBlock = (hash, isParsed = 1) => new Promise((resolve, reject) => {
  client.getblock(hash, isParsed, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

const getBlockHash = index => new Promise((resolve, reject) => {
  client.getblockhash(index, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

const getBlockHeader = blockHash => new Promise((resolve, reject) => {
  client.getblockheader(blockHash, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

const getBlockHeaders = (offset, limit) => new Promise((resolve, reject) => {
  client.getblockheaders(offset, limit, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

const getCurrentBlockHeight = () => new Promise((resolve, reject) => {
  client.getblockcount((err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

const getHashFromHeight = height => new Promise((resolve, reject) => {
  client.getblockhash(height, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

const getMasternodesList = () => new Promise((resolve, reject) => {
  client.masternodelist((err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

const getMnListDiff = (baseBlockHash, blockHash) => new Promise((resolve, reject) => {
  client.protx(constants.DASHCORE_RPC_COMMANDS.protx.diff, baseBlockHash, blockHash, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

/*eslint-disable */
// Temporary mock result
const getQuorum = regtxid => new Promise((resolve, reject) => {

  //remove when rpc getQuorum available
  const coreFixtures = require('../../../test/fixtures/coreAPIFixture');
  coreFixtures.getQuorum(regtxid)
    .then(mockData => resolve(mockData))

  // re-add when rpc getQuorum available
  // client.getquorum(regtxid, (err, r) => {
  //   if (err) {
  //     reject(err);
  //   } else {
  //     resolve(r.result);
  //   }
  // });
});
/* eslint-enable */

const getRawTransaction = txid => new Promise((resolve, reject) => {
  client.getrawtransaction(txid, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

const getRawBlock = txid => getBlock(txid, false);

const getTransaction = txid => new Promise((resolve, reject) => {
  client.gettransaction(txid, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

// const getTransition = tsid => new Promise((resolve, reject) => { // new name?
//   client.getTransition(tsid, (err, r) => {
//     if (err) {
//       reject(err);
//     } else {
//       resolve(r.result);
//     }
//   });
// });

const getTransactionFirstInputAddress = txId => new Promise((resolve, reject) => {
  client.gettransaction(txId, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.details.address);
    }
  });
});

const getUser = tx => new Promise((resolve, reject) => { // not exist?
  client.getuser(tx, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

// Address indexing needs to be enabled
// (todo getting invalid address, perhaps this should be in SDK)
const getUTXO = addr => new Promise((resolve, reject) => {
  client.getaddressutxos(addr, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

/**
 *  Layer 2 endpoints
 *  These functions represent endpoints on the data layer
 *  and can be requested only from members of the quorum assigned to a specific DAPI-client
 */

const sendRawTransition = ts => new Promise((resolve, reject) => { // not exist?
  client.sendrawtransition(ts, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

/**
 *  Layer 1 or Layer 2 endpoints
 *  depending on context these functions are either Layer 1 or Layer 2
 *  e.g. sendRawTransaction can be used to send a normal tx => Layer 1,
 *  but can also be used like its alias sendRawTransition to send
 *  a state transition updating a BU account => Layer 2.
 *  A DAPI-client will need to know if it has already been assigned
 *  a quorum in order to choose which set of DAPI nodes to use
 *  for posting a tx to this endpoint -
 *  all DAPI nodes or just it's quorum member nodes
 */

const sendRawTransaction = tx => new Promise((resolve, reject) => {
  client.sendrawtransaction(tx, (err, r) => {
    if (err) {
      reject(err);
    } else {
      resolve(r.result);
    }
  });
});

module.exports = {
  generate,
  getBestBlockHeight,
  getBlockHash, //= =getCurrentBlockHeight
  getBlock,
  getBlockHeader,
  getBlockHeaders,
  getCurrentBlockHeight,
  getHashFromHeight,
  getMasternodesList,
  getMnListDiff,
  getQuorum,
  sendRawTransition,
  sendRawTransaction,
  getRawTransaction,
  getRawBlock,
  getTransaction,
  // getTransition,
  getTransactionFirstInputAddress,
  getUser,
  getUTXO,
};
