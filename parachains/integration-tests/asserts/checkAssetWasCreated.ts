const chai = require('chai');
var should = require('chai').should();

const checkSenderBalances = async (context, ...args) => {
  let asset = args[0];

  let assetExist = asset ? true : false;

  chai.assert.equal(assetExist, true);
};

export default checkSenderBalances;
