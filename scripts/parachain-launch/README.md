# parachain-launch



# Introduction

[Parachain-launch](https://github.com/open-web3-stack/parachain-launch) is a script that generates a docker compose file allowing you to launch a testnet of multiple blockchain nodes.

To generate the compose file, first ensure `parachain-launch` is installed and run the following command

```
parachain-launch generate statemine.yml
```

This will produce an output directory that contains all the necessary files to run the testnet.

Once the run is successful, you can spin up the infrastructure using the below command

```
docker-compose -f output/docker-compose.yml up -d
```

# Requirements
[docker-compose](https://docs.docker.com/compose/install/) and [Parachain-launch](https://github.com/open-web3-stack/parachain-launch) 