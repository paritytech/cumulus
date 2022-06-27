const chai = require('chai');
var should = require('chai').should();
const BN = require('bn.js');
chai.use(require('chai-bn')(BN));

const checkAssetWasBurned = async (context, ...args) => {

  const {
    balances: {
      before: previousBalance,
      after: {
        balance: currentBalance,
      }
    }
  } = args[0];

  // Assert
  new BN(currentBalance).should.be.a.bignumber.that.is.lessThan(
    new BN(previousBalance)
  );

};

export default checkAssetWasBurned;
