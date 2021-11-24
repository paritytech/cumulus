const fs = require("fs");
const { exit } = require("process");
const { WsProvider, ApiPromise } = require("@polkadot/api");
const util = require("@polkadot/util");

// Utility script constructing a SCALE-encoded setStorage call from a key-value json array of
// genesis values by connecting to a running instance of the chain. (It is not required to be
// functional or synced.)

// connect to a local substrate chain and return the api object
async function connect(port, types = {}) {
	const provider = new WsProvider("ws://localhost:" + port);
	const api = await ApiPromise.create({
		provider,
		types,
		throwOnConnect: false,
	});
	return api;
}

if (!process.argv[2] || !process.argv[3]) {
	console.log("usage: node generate_keys <input json> <scale output file>");
	exit();
}

const input = process.argv[2];
const output = process.argv[3];

console.log("Processing", input, output);
fs.readFile(input, "utf8", (err, data) => {
	if (err) {
		console.log(`Error reading file from disk: ${err}`);
		exit(1);
	}

	const genesis = JSON.parse(data);

	console.log("loaded genesis, length =  ", genesis.length);
	console.log('Connecting via WebSocket to :9944');
	connect(9944)
		.then((api) => {
			console.log('Connected');
			const setStorage = api.tx.system.setStorage(genesis);
			const raw = setStorage.method.toU8a();
			const hex = util.u8aToHex(raw);
			fs.writeFileSync(output, hex);
			exit(0);
		})
		.catch((e) => {
			console.error(e);
			exit(1);
		});
});
