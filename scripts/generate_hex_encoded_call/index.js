const fs = require("fs");
const { exit } = require("process");
const { WsProvider, ApiPromise } = require("@polkadot/api");
const util = require("@polkadot/util");

// connect to a substrate chain and return the api object
async function connect(endpoint, types = {}) {
	const provider = new WsProvider(endpoint);
	const api = await ApiPromise.create({
		provider,
		types,
		throwOnConnect: false,
	});
	return api;
}

function writeHexEncodedBytesToOutput(method, outputFile) {
	console.log("Payload (hex): ", method.toHex());
	console.log("Payload (bytes): ", Array.from(method.toU8a()));
	fs.writeFileSync(outputFile, JSON.stringify(Array.from(method.toU8a())));
}

function remarkWithEvent(endpoint, outputFile) {
	console.log(`Generating remarkWithEvent from RPC endpoint: ${endpoint} to outputFile: ${outputFile}`);
	connect(endpoint)
		.then((api) => {
			const call = api.tx.system.remarkWithEvent("Hello");
			writeHexEncodedBytesToOutput(call.method, outputFile);
			exit(0);
		})
		.catch((e) => {
			console.error(e);
			exit(1);
		});
}

function addBridgeConfig(endpoint, outputFile, bridgedNetwork, bridgeConfig) {
	console.log(`Generating addBridgeConfig from RPC endpoint: ${endpoint} to outputFile: ${outputFile} based on bridgedNetwork: ${bridgedNetwork}, bridgeConfig: ${bridgeConfig}`);
	connect(endpoint)
		.then((api) => {
			const call = api.tx.bridgeAssetsTransfer.addBridgeConfig(bridgedNetwork, JSON.parse(bridgeConfig));
			writeHexEncodedBytesToOutput(call.method, outputFile);
			exit(0);
		})
		.catch((e) => {
			console.error(e);
			exit(1);
		});
}

function removeBridgeConfig(endpoint, outputFile, bridgedNetwork) {
	console.log(`Generating removeBridgeConfig from RPC endpoint: ${endpoint} to outputFile: ${outputFile} based on bridgedNetwork: ${bridgedNetwork}`);
	connect(endpoint)
		.then((api) => {
			const call = api.tx.bridgeAssetsTransfer.removeBridgeConfig(bridgedNetwork);
			writeHexEncodedBytesToOutput(call.method, outputFile);
			exit(0);
		})
		.catch((e) => {
			console.error(e);
			exit(1);
		});
}

function transferAssetViaBridge(endpoint, outputFile, assets, destination) {
	console.log(`Generating transferAssetViaBridge from RPC endpoint: ${endpoint} to outputFile: ${outputFile} based on assets: ${assets}, destination: ${destination}`);
	connect(endpoint)
		.then((api) => {
			const call = api.tx.bridgeAssetsTransfer.transferAssetViaBridge(JSON.parse(assets), JSON.parse(destination));
			writeHexEncodedBytesToOutput(call.method, outputFile);
			exit(0);
		})
		.catch((e) => {
			console.error(e);
			exit(1);
		});
}

if (!process.argv[2] || !process.argv[3]) {
	console.log("usage: node ./script/generate_hex_encoded_call <type> <endpoint> <output hex-encoded data file> <input message>");
	exit(1);
}

const type = process.argv[2];
const rpcEnpoint = process.argv[3];
const output = process.argv[4];
const inputArgs = process.argv.slice(5, process.argv.length);
console.log(`Generating hex-encoded call data for:`);
console.log(`	type: ${type}`);
console.log(`	rpcEnpoint: ${rpcEnpoint}`);
console.log(`	output: ${output}`);
console.log(`	inputArgs: ${inputArgs}`);

switch (type) {
	case 'remark-with-event':
		remarkWithEvent(rpcEnpoint, output);
		break;
	case 'add-bridge-config':
		addBridgeConfig(rpcEnpoint, output, inputArgs[0], inputArgs[1]);
		break;
	case 'remove-bridge-config':
		removeBridgeConfig(rpcEnpoint, output, inputArgs[0], inputArgs[1]);
		break;
	case 'transfer-asset-via-bridge':
		transferAssetViaBridge(rpcEnpoint, output, inputArgs[0], inputArgs[1]);
		break;
	case 'check':
		console.log(`Checking nodejs installation, if you see this everything is ready!`);
		break;
	default:
		console.log(`Sorry, we are out of ${type} - not yet supported!`);
}
