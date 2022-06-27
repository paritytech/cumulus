const chai = require('chai');
var should = require('chai').should();
const BN = require('bn.js');
chai.use(require('chai-bn')(BN));
import { expect } from 'chai';
import { PaymentInfo } from '../../src/interfaces';
import { getPaymentInfoForExtrinsic } from '../../src/utils';

const checkSenderBalances = async (context, ...args) => {
  const {
    balances: {
      before: {
        data: { free: senderBefore },
      },
      after: {
        data: { free: senderAfter },
      },
    },
    amount,
    fees,
  } = args[0];

  let amountSent = BigInt(amount);
  let previousBalance = BigInt(senderBefore);
  let currentBalance = BigInt(senderAfter);
  let expectedBalance: bigint;
  let fee = BigInt(0);

  expectedBalance = previousBalance - amountSent;

  if (fees) {
    const { from: extrinsic, index } = fees;
    let paymentInfo: PaymentInfo = await getPaymentInfoForExtrinsic(
      context,
      extrinsic[index]
    );
    const { partialFee } = paymentInfo;
    fee = BigInt(partialFee);
    // expectedBalance = previousBalance - amountSent - fee
  }

  // Assert
  // chai.assert.equal(currentBalance, expectedBalance)
  // expect(currentBalance).to.be.lt(expectedBalance)
  new BN(currentBalance).should.be.a.bignumber.that.is.lessThan(
    new BN(expectedBalance)
  );
};

export default checkSenderBalances;
