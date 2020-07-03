use polkadot_parachain::primitives::HeadData;
use polkadot_primitives::{
	parachain::{Id as ParaId, Info, Scheduling},
	AccountId, Hash as PHash,
};
use polkadot_runtime::{Header, Runtime, SignedExtra, SignedPayload, UncheckedExtrinsic};
use polkadot_runtime_common::{claims, parachains, registrar, BlockHashCount};
use sp_arithmetic::traits::SaturatedConversion;
use sp_keyring::Sr25519Keyring;
use sp_runtime::{codec::Encode, generic};
use sp_version::RuntimeVersion;

jsonrpsee::rpc_api! {
	Author {
		#[rpc(method = "author_submitExtrinsic", positional_params)]
		fn submit_extrinsic(extrinsic: String) -> PHash;
	}

	Chain {
		#[rpc(method = "chain_getFinalizedHead")]
		fn current_block_hash() -> PHash;

		#[rpc(method = "chain_getHeader", positional_params)]
		fn header(hash: PHash) -> Option<Header>;

		#[rpc(method = "chain_getBlockHash", positional_params)]
		fn block_hash(hash: Option<u64>) -> Option<PHash>;
	}

	State {
		#[rpc(method = "state_getRuntimeVersion")]
		fn runtime_version() -> RuntimeVersion;
	}

	System {
		#[rpc(method = "system_networkState")]
		fn network_state() -> serde_json::Value;
	}
}

fn main() {
	println!("Hello, world!");
}

async fn create_transaction(
	runtime: impl AsRef<std::path::Path>,
	genesis_state: HeadData,
	tokens: u64,
	parachain_id: ParaId,
	sudo_private_key: Sr25519Keyring,
	account_id: AccountId,
) -> UncheckedExtrinsic {
	let transport_client_alice =
		jsonrpsee::transport::http::HttpTransportClient::new("http://127.0.0.1:27015");
	let mut client_alice = jsonrpsee::raw::RawClient::new(transport_client_alice);

	let runtime_version = State::runtime_version(&mut client_alice).await.unwrap();
	let current_block_hash = Chain::block_hash(&mut client_alice, None)
		.await
		.unwrap()
		.unwrap();
	let current_block = Chain::header(&mut client_alice, current_block_hash)
		.await
		.unwrap()
		.unwrap()
		.number
		.saturated_into::<u64>();

	let genesis_block = Chain::block_hash(&mut client_alice, 0)
		.await
		.unwrap()
		.unwrap();

	let wasm = std::fs::read(runtime).unwrap();
	let call = pallet_sudo::Call::sudo(Box::new(
		registrar::Call::<Runtime>::register_para(
			parachain_id,
			Info {
				scheduling: Scheduling::Always,
			},
			wasm.into(),
			genesis_state,
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
	let signature = raw_payload.using_encoded(|e| sudo_private_key.sign(e));
	UncheckedExtrinsic::new_signed(
		call.into(),
		account_id.into(),
		sp_runtime::MultiSignature::Sr25519(signature),
		extra,
	)
}
