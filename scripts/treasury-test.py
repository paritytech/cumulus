#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Created on Thu Sep 16 20:41:13 2021

@author: brenzi
"""

from substrateinterface import SubstrateInterface, Keypair
from substrateinterface.utils.ss58 import ss58_encode

def get_balance(who):
    return substrate.query('System', 'Account', params=[who]).value['data']['free']

substrate = SubstrateInterface(
        url="ws://127.0.0.1:9944",
        type_registry_preset='kusama'
    )
alice = Keypair.create_from_uri('//Alice')
dave = Keypair.create_from_uri('//Dave')
treasury = ss58_encode('0x' + b'modlpy/trsry'.hex() + '0000000000000000000000000000000000000000')

alicebefore = get_balance(alice.ss58_address)
treasurybefore = get_balance(treasury)
totalissuancebefore = substrate.query('Balances', 'TotalIssuance')
print('total issuance', totalissuancebefore)

amount = 10 * 10**9 #milli

call = substrate.compose_call(
    call_module='Balances',
    call_function='transfer',
    call_params={
        'dest': dave.ss58_address,
        'value': amount
    }
)

payment_info = substrate.get_payment_info(call=call, keypair=alice)
print("Payment info: ", payment_info)

extrinsic = substrate.create_signed_extrinsic(
    call=call,
    keypair=alice,
    era={'period': 64}
)
receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
print('extrinsic sent')

totalissuanceafter = substrate.query('Balances', 'TotalIssuance')
print('difference in total issuance: ', totalissuancebefore.value - totalissuanceafter.value)

aliceafter = get_balance(alice.ss58_address)

paidfee = alicebefore - aliceafter - amount
print('fee paid : ', paidfee)

treasuryafter = get_balance(treasury)

print('treasury balance is ', treasuryafter, ' and has increased by', treasuryafter-treasurybefore)