// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use environmental::PhantomData;
use hash_db::Hasher;
use sp_trie::NodeCodec;
use trie_db::TrieCache;

struct SimpleCache<H: Hasher> {
	_phantom: PhantomData<H>,
}

impl<H: Hasher> trie_db::TrieCache<NodeCodec<H>> for SimpleCache<H> {
	fn lookup_value_for_key(&mut self, key: &[u8]) -> Option<&trie_db::CachedValue<H::Out>> {
		None
	}

	fn cache_value_for_key(&mut self, key: &[u8], value: trie_db::CachedValue<H::Out>) {}

	fn get_or_insert_node(
		&mut self,
		hash: <NodeCodec<H> as trie_db::NodeCodec>::HashOut,
		fetch_node: &mut dyn FnMut() -> trie_db::Result<
			trie_db::node::NodeOwned<H::Out>,
			H::Out,
			<NodeCodec<H> as trie_db::NodeCodec>::Error,
		>,
	) -> trie_db::Result<
		&trie_db::node::NodeOwned<H::Out>,
		H::Out,
		<NodeCodec<H> as trie_db::NodeCodec>::Error,
	> {
		todo!()
	}

	fn get_node(
		&mut self,
		hash: &H::Out,
	) -> Option<&trie_db::node::NodeOwned<<NodeCodec<H> as trie_db::NodeCodec>::HashOut>> {
		None
	}
}
