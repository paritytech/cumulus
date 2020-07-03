use sp_keyring::Sr25519Keyring;

fn main() {
	println!("Hello, world!");
}

fn create_transaction(runtime: PathBuf, genesis_state: &str, tokens: u64, parachain_id: ParaId,
sudo_private_key: Sr25519Keyring) -> UncheckedExtrinsic {
	let wasm = fs::read(target_dir().join(
		"wbuild/cumulus-test-parachain-runtime/cumulus_test_parachain_runtime.compact.wasm",
	))
	.unwrap();
	let call = pallet_sudo::Call::sudo(Box::new(
		registrar::Call::<Runtime>::register_para(
			100.into(),
			Info {
				scheduling: Scheduling::Always,
			},
			wasm.into(),
			genesis_state.into(),
		)
		.into(),
	));
	let nonce = 0;
	let period = BlockHashCount::get()
		.checked_next_power_of_two()
		.map(|c| c / 2)
		.unwrap_or(2) as u64;
	let tip = 0;
	let extra: SignedExtra = (
		frame_system::CheckSpecVersion::<Runtime>::new(),
		frame_system::CheckTxVersion::<Runtime>::new(),
		frame_system::CheckGenesis::<Runtime>::new(),
		frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
		frame_system::CheckNonce::<Runtime>::from(nonce),
		frame_system::CheckWeight::<Runtime>::new(),
		pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
		registrar::LimitParathreadCommits::<Runtime>::new(),
		parachains::ValidateDoubleVoteReports::<Runtime>::new(),
		pallet_grandpa::ValidateEquivocationReport::<Runtime>::new(),
		claims::PrevalidateAttests::<Runtime>::new(),
	);
	let raw_payload = SignedPayload::from_raw(
		call.clone().into(),
		extra.clone(),
		(
			(),
			runtime_version.spec_version,
			runtime_version.transaction_version,
			genesis_block,
			current_block_hash,
			(),
			(),
			(),
			(),
			(),
			(),
			(),
		),
	);
	let signature = raw_payload.using_encoded(|e| Alice.sign(e));
	polkadot_runtime::UncheckedExtrinsic::new_signed(
		call.into(),
		Alice.into(),
		sp_runtime::MultiSignature::Sr25519(signature),
		extra,
	)
}
