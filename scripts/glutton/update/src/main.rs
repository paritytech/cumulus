use crate::glutton_para::runtime_types::sp_arithmetic::per_things::Perbill;
use clap::{Parser, ValueEnum};
use subxt::{
	dynamic::Value,
	ext::sp_core::{sr25519::Pair as Sr25519Pair, Pair},
	tx::PairSigner,
	utils::AccountId32,
	OnlineClient,
};

mod config;
use crate::config::GluttonConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum RelayChain {
	Local,
	Kusama,
	Versi,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	/// Relay chain context selection: kusama, versi or local
	#[clap(value_enum)]
	#[arg(short, long)]
	relay: RelayChain,

	/// Update Glutton Parachains from 'para_id = from' (inclusive)
	#[arg(short, long)]
	from: u32,

	/// Update Glutton Parachains to 'para_id = to' (inclusive)
	#[arg(short, long)]
	to: u32,

	/// Optional new 'storage' value to update
	#[arg(short, long)]
	storage: Option<u32>,

	/// Optional new 'compute' value to update
	#[arg(short, long)]
	compute: Option<u32>,

	/// Sudo account secret seed in hex format
	#[arg(short, long)]
	private_key: String,
}

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "./artifacts/glutton_metadata.scale")]
pub mod glutton_para {}

type RuntimeCall = glutton_para::runtime_types::glutton_runtime::RuntimeCall;
type GluttonCall = glutton_para::runtime_types::pallet_glutton::pallet::Call;

async fn get_account_nonce(
	client: &OnlineClient<GluttonConfig>,
	account: &AccountId32,
) -> Result<u32, Box<dyn std::error::Error>> {
	let runtime_api_call = subxt::dynamic::runtime_api_call(
		"AccountNonceApi",
		"account_nonce",
		vec![Value::from_bytes(account)],
	);

	let nonce = client
		.runtime_api()
		.at_latest()
		.await?
		.call(runtime_api_call)
		.await?
		.to_value()?
		.as_u128()
		.unwrap() as u32;

	Ok(nonce)
}

async fn update_glutton(
	client: &OnlineClient<GluttonConfig>,
	call: RuntimeCall,
	account: &AccountId32,
	signer: &PairSigner<GluttonConfig, Sr25519Pair>,
) -> Result<(), Box<dyn std::error::Error>> {
	// Get signer's `nonce`
	let nonce = get_account_nonce(client, account).await?;

	// Build the tx
	let tx = glutton_para::tx().sudo().sudo(call);

	// Build the signed tx
	let signed_tx = client
		.tx()
		.create_signed_with_nonce(&tx, signer, nonce, Default::default())
		.unwrap();

	// Submit and watch the tx
	let in_block = signed_tx.submit_and_watch().await.unwrap().wait_for_in_block().await.unwrap();

	let block_hash = in_block.block_hash();

	println!("- Tx included in block {:?}", block_hash);

	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Cli::parse();
	let from = args.from;
	let to = args.to;
	let para_ids = from..to + 1;
	let seed = args.private_key;

	// Get account and signer from secre seed
	let account_pair: Sr25519Pair =
		Pair::from_string(seed.as_str(), None).expect("Failed to create key pair");
	let sudo_signer = PairSigner::new(account_pair.clone());
	let sudo_account = AccountId32::from(account_pair.public());

	for id in para_ids {
		println!("\nUpdating {:?} Glutton {}", args.relay, id);

		// Create a new API client, configured to talk to Glutton nodes.
		println!("- Connecting to client...");
		let client: OnlineClient<GluttonConfig>;

		match args.relay {
			RelayChain::Local => {
				client = OnlineClient::<GluttonConfig>::from_url("ws://127.0.0.1:9810").await?;
			},
			RelayChain::Versi => {
				client = OnlineClient::<GluttonConfig>::from_url(format!(
					"wss://versi-glutton-collator-{}-node-1.parity-versi.parity.io:443",
					id
				))
				.await?;
			},
			RelayChain::Kusama => {
				client = OnlineClient::<GluttonConfig>::from_url(format!(
					"wss://versi-glutton-collator-{}-node-1.parity-versi.parity.io:443", // TODO: change when we know Kusama endpoints
					id
				))
				.await?;
			},
		}

		// Set storage
		if let Some(new_storage) = args.storage {
			// Build `set_storage` call
			let set_storage_call =
				RuntimeCall::Glutton(GluttonCall::set_storage { storage: Perbill(new_storage) });

			// Sumbit `set_storage` call
			println!("- Submitting 'set_storage {{ storage: {} }}'...", new_storage);
			update_glutton(&client, set_storage_call, &sudo_account, &sudo_signer).await?;
		}

		// Set compute
		if let Some(new_compute) = args.compute {
			// Build `set_compute` call
			let set_compute_call =
				RuntimeCall::Glutton(GluttonCall::set_compute { compute: Perbill(new_compute) });

			// Sumbit `set_compute` call
			println!("- Submitting 'set_compute {{ compute: {} }}'...", new_compute);
			update_glutton(&client, set_compute_call, &sudo_account, &sudo_signer).await?;
		}
	}

	Ok(())
}
