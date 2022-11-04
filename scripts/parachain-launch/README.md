# parachain-launch



# Introduction

[Parachain-launch](https://github.com/open-web3-stack/parachain-launch) is a script that generates a docker compose file allowing you to launch a testnet of multiple blockchain nodes.

The output directory already contains a generated docker-compose file so you can spin a testnet up without needing to install `parachain-launch`

To get the testnet up and running run the below command

```
docker-compose -f output/docker-compose.yml up -d
```

To regenerate the compose file, first ensure `parachain-launch` is installed and run the following command

```
parachain-launch generate statemine.yml
```

# Requirements
The only requirement to spin a testnet up is to have [docker-compose](https://docs.docker.com/compose/install/) installed, however to generate your own compose file then please follow the instructions to install `parachain-launch`