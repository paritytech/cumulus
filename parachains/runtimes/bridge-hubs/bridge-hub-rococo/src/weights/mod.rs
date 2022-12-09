// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Expose the auto generated weight files.

pub mod cumulus_pallet_xcmp_queue;
pub mod pallet_balances;
pub mod paritydb_weights;
pub mod rocksdb_weights;
pub mod xcm;

pub use bp_bridge_hub_rococo::{
	block_weights::constants::BlockExecutionWeight,
	extrinsic_weights::constants::ExtrinsicBaseWeight,
};
pub use paritydb_weights::constants::ParityDbWeight;
pub use rocksdb_weights::constants::RocksDbWeight;
