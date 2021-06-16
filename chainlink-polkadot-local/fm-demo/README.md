# Substrate FM demo

This directory contains a demonstration of using the Chainlink External Initiator with FluxMonitor to run a price feed
on a substrate chain using the chainlink-feed pallet.

For the simplicity of this demo, a single Chainlink node (+ Postgres DB, External Initiator) is used, but with multiple
accounts. In a real environment, each of these accounts would run their own Chainlink node.

> This is in active development. Some Docker images may be outdated and may require to be built from source.

## Setup

1. Start up the substrate chain

```bash
./scripts/run-chain.sh
```

> The next step requires using the Polkadot JS web interface (https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/settings/developer).
> Make sure you set the additional types from the chainlink feed pallet [types.json](../substrate-node-example/types.json).

2. Create a new Chainlink feed by submitting a `chainlinkFeed.createFeed()` extrinsic:

```
Payment: 0.01
Timeout: 600
Submission Value Bounds: (0, 99999999999999999999999999999999)
Min submissions: 1
Decimals: 8
Description: 0x444f54202f20555344 (DOT / USD)
Restart delay: 0
Oracles:
 - 5EsiCstpHTxarfafS3tvG7WDwbrp9Bv6BbyRvpwt3fY8PCtN
 - 5CDogos4Dy2tSCvShBHkeFeMscwx9Wi2vFRijjTRRFau3vkJ
```

3. Fund the oracles addresses above

4. Start the Chainlink node (and adapters)

```bash
./scripts/run-chainlink.sh
```

5. Add the bridges to the Chainlink node:

- Go to the Chainlink node GUI `http://localhost:6688/` and log in with `notreal@fakeemail.ch:twochains`

- Go to the Bridges page, and add the two Substrate adapters:

```
Name: substrate-adapter1
URL: http://substrate-adapter1:8080/

Name: substrate-adapter2
URL: http://substrate-adapter2:8080/
```

6. Add the External Initiator to the Chainlink node

```bash
./scripts/ei-config.sh
```

Check if `fm-demo/external_initiator.env` is populated with the new credentials

(Manual steps in case the above step fails:

```bash
docker exec -it chainlink-node /bin/bash
chainlink admin login -f /run/secrets/apicredentials
chainlink initiators create substrate http://external-initiator:8080/jobs
```

Take note of the keys and secrets, and enter them in the `external_initiator.env` file:

```dotenv
EI_CI_ACCESSKEY=[OUTGOINGTOKEN]
EI_CI_SECRET=[OUTGOINGSECRET]
EI_IC_ACCESSKEY=[ACCESSKEY]
EI_IC_SECRET=[SECRET]
```

)

7. Start the external initiator

```bash
./scripts/run-ei.sh
```

8. Add the jobs to the Chainlink node (assuming feed ID "0"):

> These jobs will run with 1m heartbeat and 30s polling.

```json
{
  "initiators": [
    {
      "type": "external",
      "params": {
        "name": "substrate",
        "body": {
          "endpoint": "substrate",
          "feed_id": 0,
          "account_id": "0x7c522c8273973e7bcf4a5dbfcc745dba4a3ab08c1e410167d7b1bdf9cb924f6c",
          "fluxmonitor": {
            "requestData": {
              "data": { "from": "DOT", "to": "USD" }
            },
            "feeds": [{ "url": "http://coingecko-adapter:8080" }],
            "threshold": 0.5,
            "absoluteThreshold": 0,
            "precision": 8,
            "pollTimer": { "period": "30s" },
            "idleTimer": { "duration": "1m" }
          }
        }
      }
    }
  ],
  "tasks": [
    {
      "type": "substrate-adapter1",
      "params": { "multiply": 1e8 }
    }
  ]
}
```

```json
{
  "initiators": [
    {
      "type": "external",
      "params": {
        "name": "substrate",
        "body": {
          "endpoint": "substrate",
          "feed_id": 0,
          "account_id": "0x06f0d58c43477508c0e5d5901342acf93a0208088816ff303996564a1d8c1c54",
          "fluxmonitor": {
            "requestData": {
              "data": { "from": "DOT", "to": "USD" }
            },
            "feeds": [{ "url": "http://coingecko-adapter:8080" }],
            "threshold": 0.5,
            "absoluteThreshold": 0,
            "precision": 8,
            "pollTimer": { "period": "30s" },
            "idleTimer": { "duration": "1m" }
          }
        }
      }
    }
  ],
  "tasks": [
    {
      "type": "substrate-adapter2",
      "params": { "multiply": 1e8 }
    }
  ]
}
```
