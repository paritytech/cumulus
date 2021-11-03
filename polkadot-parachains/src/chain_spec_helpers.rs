// Copyright (c) 2019 Alain Brenzikofer
// This file is part of Encointer
//
// Encointer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Encointer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Encointer.  If not, see <http://www.gnu.org/licenses/>.

//! Some helpers to create chains-specs
//!
//! Only moved stuff here that is not in the `chain_spec.rs` upstream to prevent upstream merge
//! confusion.

use parachain_runtime::{AccountId, AuraId};
use sc_chain_spec::Properties;
use sc_service::ChainType;
use sp_core::{crypto::Ss58Codec, sr25519, Public};
use sp_keyring::AccountKeyring::{Alice, Bob};
use std::str::FromStr;

pub fn public_from_ss58<TPublic: Public + FromStr>(ss58: &str) -> TPublic
where
	<TPublic as FromStr>::Err: std::fmt::Debug,
{
	TPublic::from_ss58check(ss58).expect("supply valid ss58!")
}

/// Defines the key set to use for root, endowed accounts, or authorities.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GenesisKeys {
	/// Use Encointer keys. Root is not in endowed keys.
	Encointer,
	/// Use Encointer keys. Root is in endowed keys.
	EncointerWithRootEndowed,
	/// Use Keys from the keyring for a test setup
	WellKnown,
}

pub struct WellKnownKeys;

impl WellKnownKeys {
	pub fn root() -> AccountId {
		Alice.to_account_id()
	}

	pub fn endowed() -> Vec<AccountId> {
		vec![Alice.to_account_id(), Bob.to_account_id()]
	}

	pub fn authorities() -> Vec<AuraId> {
		vec![Alice.public().into()]
	}
}

pub struct EncointerKeys;

impl EncointerKeys {
	pub fn root() -> AccountId {
		public_from_ss58::<sr25519::Public>("5CSLXnYZQeVDvNmanYEJn4YXXhgFLKYwp2f216NsDehR8mVU")
			.into()
	}
	pub fn authorities() -> Vec<AuraId> {
		vec![
			public_from_ss58::<sr25519::Public>("5ECixNNkkfjHYqzwEkbuoVdzRqBpW2eTp8rp2SYR8fuNfQ4G")
				.into(),
			public_from_ss58::<sr25519::Public>("5CMekcxVqQ1ziRHoibG2w5Co7wXu7LWXtX7yTK67NWrJ61a9")
				.into(),
			public_from_ss58::<sr25519::Public>("5Gdh3vLvFKPMwMf2h4sngMgxSnaYGZUJTPGkGwoVmZFM2Ss5")
				.into(),
			public_from_ss58::<sr25519::Public>("5DhVfSunCNHy1R1ozJx1V59YbjDAEzEsaAghmzE77opGVUNf")
				.into(),
			public_from_ss58::<sr25519::Public>("5EWpnnj53PL9KbJAMnsrezQYZhwQ6UwnqSknnXd1ptVvRfVJ")
				.into(),
		]
	}
}

pub enum RelayChain {
	Kusama,
	KusamaLocal,
	Rococo,
	RococoLocal,
	Westend,
	WestendLocal,
}

impl ToString for RelayChain {
	fn to_string(&self) -> String {
		match self {
			RelayChain::Rococo => "rococo".into(),
			RelayChain::RococoLocal => "rococo-local".into(),
			RelayChain::Kusama => "kusama".into(),
			RelayChain::KusamaLocal => "kusama-local".into(),
			RelayChain::Westend => "westend".into(),
			RelayChain::WestendLocal => "westend-local".into(),
		}
	}
}

impl RelayChain {
	pub fn chain_type(&self) -> ChainType {
		match self {
			RelayChain::Rococo => ChainType::Live,
			RelayChain::RococoLocal => ChainType::Local,
			RelayChain::Kusama => ChainType::Live,
			RelayChain::KusamaLocal => ChainType::Local,
			RelayChain::Westend => ChainType::Local,
			RelayChain::WestendLocal => ChainType::Live,
		}
	}

	pub fn properties(&self) -> Properties {
		match self {
			RelayChain::Kusama | RelayChain::KusamaLocal => kusama_properties(),
			RelayChain::Rococo | RelayChain::RococoLocal => rococo_properties(),
			RelayChain::Westend | RelayChain::WestendLocal => westend_properties(),
		}
	}

	pub fn protocol_id(&self) -> &'static str {
		match self {
			RelayChain::Kusama | RelayChain::KusamaLocal => "ksmcc3",
			RelayChain::Rococo | RelayChain::RococoLocal => "rococo",
			RelayChain::Westend | RelayChain::WestendLocal => "wnd2",
		}
	}
}

pub fn rococo_properties() -> Properties {
	serde_json::from_str(
		r#"{
				"ss58Format": 42,
				"tokenDecimals": 12,
				"tokenSymbol": "ROC"
				}"#,
	)
	.unwrap()
}

pub fn kusama_properties() -> Properties {
	serde_json::from_str(
		r#"{
				"ss58Format": 2,
				"tokenDecimals": 12,
				"tokenSymbol": "KSM"
				}"#,
	)
	.unwrap()
}

pub fn westend_properties() -> Properties {
	serde_json::from_str(
		r#"{
				"ss58Format": 42,
				"tokenDecimals": 12,
				"tokenSymbol": "WND"
				}"#,
	)
	.unwrap()
}
