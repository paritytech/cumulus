const fs = require('fs');

async function addGenesisParachain(specPath, paraId, headPath, wasmPath, outPath) {
    let head = fs.readFileSync(headPath);
    let wasm = fs.readFileSync(wasmPath);
    let rawdata = fs.readFileSync(specPath);

    let chainSpec = JSON.parse(rawdata);
    // Check runtime_genesis_config key for rococo compatibility.
    const runtimeConfig =
        chainSpec.genesis.runtime.runtime_genesis_config ||
        chainSpec.genesis.runtime;
    if (runtimeConfig.parachainsParas) {
        let paras = runtimeConfig.parachainsParas.paras;

        let new_para = [
            parseInt(paraId),
            {
                genesis_head: head.toString(),
                validation_code: wasm.toString(),
                parachain: true,
            },
        ];

        paras.push(new_para);

        let data = JSON.stringify(chainSpec, null, 2);
        fs.writeFileSync(outPath, data);
        console.log(`  ✓ Added Genesis Parachain ${paraId}`);
    }
}

addGenesisParachain(
    process.env.SPEC_PATH,
    process.env.PARA_ID,
    process.env.HEAD_PATH,
    process.env.WASM_PATH,
    process.env.TMP_PATH,
)