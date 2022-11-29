Description: RPC collator should build blocks
Network: ./0006-rpc_collator_builds_blocks.toml
Creds: config

alice: is up
bob: is up
charlie: is up
one: is up
two: is up
three: is up
dave: is up
eve: is up

alice: parachain 2000 is registered within 225 seconds
alice: parachain 2000 block height is at least 10 within 250 seconds

dave: reports block height is at least 12 within 250 seconds
eve: reports block height is at least 12 within 250 seconds
one: pause
dave: reports block height is at least 20 within 200 seconds
one: resume
sleep 10
two: pause
three: pause
dave: is up
dave: reports block height is at least 30 within 200 seconds
