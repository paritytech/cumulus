use sp_keyring::AccountKeyring;
use subxt::{dynamic::{At, Value}, tx::{PairSigner}, OnlineClient};
use crate::glutton_para::runtime_types::{sp_arithmetic::per_things::Perbill};

mod config;
use crate::config::GluttonConfig;

// wss://versi-glutton-collator-1300-node-1.parity-versi.parity.io

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "./artifacts/glutton_metadata.scale")]
pub mod glutton_para {}

type RuntimeCall = glutton_para::runtime_types::glutton_runtime::RuntimeCall;
type GluttonCall = glutton_para::runtime_types::pallet_glutton::pallet::Call;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new API client, configured to talk to Glutton nodes.
    let client = OnlineClient::<GluttonConfig>::from_url("ws://127.0.0.1:9810").await?;

	// Get the nonce of sudo account
	let sudo_account = AccountKeyring::Alice.to_account_id();

	let storage_query =
		subxt::dynamic::storage("System", "Account", vec![Value::from_bytes(sudo_account)]);

	let value = client
		.storage()
		.at_latest()
		.await?
		.fetch(&storage_query)
		.await?;

	let nonce = if let Some(decoded_value) = value {
		decoded_value
			.to_value()?
			.at("nonce")
			.unwrap()
			.as_u128()
			.unwrap() as u32
	} else {
		0
	};

	// Build de pallet_glutton call
	let new_storage = Perbill(55555);

	let set_storage_call = RuntimeCall::Glutton(GluttonCall::set_storage {
		storage: new_storage
    });

	// Build the tx to sign
    let sudo_set_storage_tx = glutton_para::tx().sudo().sudo(set_storage_call);

	// Get the sudo signer
	let signer = PairSigner::new(AccountKeyring::Alice.pair());

	// Build the signed tx
	let signed_extrinsic = client
		.tx()
		.create_signed_with_nonce(
			&sudo_set_storage_tx,
			&signer,
			nonce,
			Default::default()
		)
		.unwrap();

	// Submit and Watch the extrinsic
    let in_block = signed_extrinsic
        .submit_and_watch()
        .await
        .unwrap()
        .wait_for_in_block()
        .await
        .unwrap();

    Ok(())
}
