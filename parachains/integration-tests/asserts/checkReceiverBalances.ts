const chai = require('chai');
var should = require('chai').should();
const BN = require('bn.js');
chai.use(require('chai-bn')(BN));

const checkReceiverBalances = async (context, ...args) => {
  const {
    balances: {
      before: {
        data: { free: receiverBefore },
      },
      after: {
        data: { free: receiverAfter },
      },
    },
    amount,
    fees,
  } = args[0];

  let previousBalance = BigInt(receiverBefore);
  let currentBalance = BigInt(receiverAfter);

  // Assert
  new BN(currentBalance).should.be.a.bignumber.that.is.greaterThan(
    new BN(previousBalance)
  );
};

export default checkReceiverBalances;
