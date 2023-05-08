pub use integration_tests_common::{
	AccountId,
	PolkadotMockNet, KusamaMockNet, PolkadotRelay, KusamaRelay, Statemint, Statemine, PenpalPolkadot, PenpalKusama,
	PolkadotSender, PolkadotReceiver, KusamaSender, KusamaReceiver,
	StatemintSender, StatemintReceiver, StatemineSender, StatemineReceiver,
	PenpalPolkadotSender, PenpalPolkadotReceiver, PenpalKusamaSender, PenpalKusamaReceiver,
	constants::accounts::{ALICE, BOB},
};
#[cfg(test)]
mod tests {}
