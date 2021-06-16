# Chainlink components for Substrate

This tool automates the setup and running of Chainlink components to read/write from a Substrate chain.

## Running

### Initial setup

_Note: Make sure you have cd-ed into this directory_

```bash
./setup
```

This will create and start 3 Chainlink nodes, with an adapter and EI connected to each.

Also it will spin up a substrate node with the RunLog pallet included. (This node is built locally and might take some time initially. You will see a message of "API-WS: disconnected from ws://localhost:9944: 1006:: connection failed" while the node is being built. You can check the status of it by running "docker attach chain-runlog").

### Start/stop

To stop the nodes, run:

```bash
docker-compose down
```

And to start them again, run:

```bash
docker-compose up
```

## Troubleshooting

### Stuck at "waiting for localhost:669X"

Check the logs of your docker container running the chainlink node:
`docker logs -f runlog-demo_chainlink-node1_1`

You need to make sure you followed the setup section

### cat jobids.txt is null

The external initiator needs to be up and running before you can create jobs.

It might be the case that it wasn't operational yet, in this case simply re-execute the
part of job creation from setup:

```bash
source ./internal-scripts/add-jobspec.sh

add_jobspec ...
```
