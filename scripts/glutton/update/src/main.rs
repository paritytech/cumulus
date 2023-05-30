use sp_keyring::AccountKeyring;
use subxt::{tx::PairSigner, OnlineClient, PolkadotConfig};
use crate::glutton_para::runtime_types::sp_arithmetic::per_things::Perbill;

// wss://versi-glutton-collator-1300-node-1.parity-versi.parity.io

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "./artifacts/glutton_metadata.scale")]
pub mod glutton_para {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new API client, configured to talk to Glutton nodes.
    let api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:9810").await?;

    // let dest = AccountKeyring::Bob.to_account_id().into();
	let storage = Perbill(100_000_000); // 10%
    let set_compute_tx = glutton_para::tx().glutton().set_compute(storage);

    // Submit the balance transfer extrinsic from Alice, and wait for it to be successful
    // and in a finalized block. We get back the extrinsic events if all is well.
    let from = PairSigner::new(AccountKeyring::Alice.pair());
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&set_compute_tx, &from)
        .await?
        .wait_for_finalized_success()
        .await?;

	// A query to obtain some contant:
	// let constant_query = glutton_para::constants().system().block_length();

	// // Obtain the value:
	// let value = api.constants().at(&constant_query)?;

	// println!("Block length: {value:?}");
	// Ok(())

    // Find a Transfer event and print it.
    // let set_compute_event = events.find_first::<glutton_para::glutton::events::ComputationLimitSet>()?;
    // if let Some(event) = set_compute_event {
    //     println!("Balance transfer success: {event:?}");
    // }

    Ok(())
}
