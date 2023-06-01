use subxt::{dynamic::{At, Value}, tx::{PairSigner}, utils::AccountId32, ext::sp_core::{Pair, sr25519::Pair as Sr25519Pair}, OnlineClient};
use crate::glutton_para::runtime_types::{sp_arithmetic::per_things::Perbill};

mod config;
use crate::config::GluttonConfig;

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "./artifacts/glutton_metadata.scale")]
pub mod glutton_para {}

type RuntimeCall = glutton_para::runtime_types::glutton_runtime::RuntimeCall;
type GluttonCall = glutton_para::runtime_types::pallet_glutton::pallet::Call;

async fn get_account_nonce(
	client: &OnlineClient<GluttonConfig>,
	account: &AccountId32
) -> Result<u32, Box<dyn std::error::Error>> {
	let mut nonce = 0;

	let storage_query =
			subxt::dynamic::storage("System", "Account", vec![Value::from_bytes(account)]);

	let value = client
		.storage()
		.at_latest()
		.await?
		.fetch(&storage_query)
		.await?;

	if let Some(decoded_value) = value {
		nonce = decoded_value
					.to_value()?
					.at("nonce")
					.unwrap()
					.as_u128()
					.unwrap() as u32
	}

	Ok(nonce)
}

async fn update_glutton(
	client: &OnlineClient<GluttonConfig>,
	call: RuntimeCall,
	account: &AccountId32,
	signer: &PairSigner<GluttonConfig, Sr25519Pair>
) -> Result<(), Box<dyn std::error::Error>> {
	// Get signer's `nonce`
	let nonce = get_account_nonce(client, account).await?;

	// Build the tx
	let tx = glutton_para::tx().sudo().sudo(call);

	// Build the signed tx
	let signed_tx = client
		.tx()
		.create_signed_with_nonce(
			&tx,
			signer,
			nonce,
			Default::default()
		)
		.unwrap();

	// Submit and watch the tx
	let in_block = signed_tx
		.submit_and_watch()
		.await
		.unwrap()
		.wait_for_in_block()
		.await
		.unwrap();

	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let para_ids = 1300..1301;
	let new_storage = 100_000_000;
	let new_compute = 200_000_000;
	let seed = "0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"; // Alice

	// Get account and signer from secre seed
    let seed_bytes = hex::decode(&seed[2..]).expect("Invalid seed hex value");
	let account_pair: Sr25519Pair = Pair::from_seed_slice(&seed_bytes).expect("Failed to create key pair");
	// let account_pair: Pair = Pair::from_string(seed, None).expect("Failed to create key pair");
	let sudo_signer = PairSigner::new(account_pair.clone());
	let sudo_account = AccountId32::from(account_pair.public());

	for id in para_ids {
		println!("Para id: {}", id);
		// Create a new API client, configured to talk to Glutton nodes.
		// let client = OnlineClient::<GluttonConfig>::from_url(format!("wss://versi-glutton-collator-{}-node-1.parity-versi.parity.io:443", id)).await?;
		let client = OnlineClient::<GluttonConfig>::from_url("ws://127.0.0.1:9810").await?;

		// Build `set_storage` call
		let set_storage_call = RuntimeCall::Glutton(GluttonCall::set_storage {
			storage: Perbill(new_storage)
		});

		// Sumbit `set_storage` call
		update_glutton(&client, set_storage_call, &sudo_account, &sudo_signer).await?;

		// Build `set_compute` call
		let set_compute_call = RuntimeCall::Glutton(GluttonCall::set_compute {
			compute: Perbill(new_compute)
		});

		// Sumbit `set_compute` call
		update_glutton(&client, set_compute_call, &sudo_account, &sudo_signer).await?;
	}

    Ok(())
}
