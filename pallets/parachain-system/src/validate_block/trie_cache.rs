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

use core::{
	cell::RefCell,
	sync::atomic::{AtomicU32, Ordering},
};
use environmental::PhantomData;
use hash_db::Hasher;
use hashbrown::hash_map::HashMap;
use sp_state_machine::TrieCacheProvider;
use sp_std::boxed::Box;
use sp_trie::NodeCodec;
use trie_db::{node::NodeOwned, TrieCache, TrieError};

pub(crate) struct SimpleCache<'a, H: Hasher> {
	node_cache: spin::MutexGuard<'a, HashMap<H::Out, NodeOwned<H::Out>>>,
	value_cache: spin::MutexGuard<'a, HashMap<Box<[u8]>, trie_db::CachedValue<H::Out>>>,
}

impl<'a, H: Hasher> trie_db::TrieCache<NodeCodec<H>> for SimpleCache<'a, H> {
	fn lookup_value_for_key(&mut self, key: &[u8]) -> Option<&trie_db::CachedValue<H::Out>> {
		self.value_cache.get(key)
	}

	fn cache_value_for_key(&mut self, key: &[u8], value: trie_db::CachedValue<H::Out>) {
		self.value_cache.insert(key.into(), value);
	}

	fn get_or_insert_node(
		&mut self,
		hash: <NodeCodec<H> as trie_db::NodeCodec>::HashOut,
		fetch_node: &mut dyn FnMut() -> trie_db::Result<
			NodeOwned<H::Out>,
			H::Out,
			<NodeCodec<H> as trie_db::NodeCodec>::Error,
		>,
	) -> trie_db::Result<&NodeOwned<H::Out>, H::Out, <NodeCodec<H> as trie_db::NodeCodec>::Error> {
		if self.node_cache.contains_key(&hash) {
			if let Some(value) = self.node_cache.get(&hash) {
				return Ok(value)
			} else {
				panic!("This can not happen");
			}
		}

		let fetched = match fetch_node() {
			Ok(new_node) => new_node,
			Err(e) => return Err(e),
		};

		let (_key, value) = self.node_cache.insert_unique_unchecked(hash, fetched);
		Ok(value)
	}

	fn get_node(
		&mut self,
		hash: &H::Out,
	) -> Option<&NodeOwned<<NodeCodec<H> as trie_db::NodeCodec>::HashOut>> {
		self.node_cache.get(hash)
	}
}

pub(crate) struct CacheProvider<H: Hasher> {
	initialized: AtomicU32,
	node_cache: spin::Mutex<HashMap<H::Out, NodeOwned<H::Out>>>,
	value_cache: spin::Mutex<HashMap<Box<[u8]>, trie_db::CachedValue<H::Out>>>,
}

impl<H: Hasher> CacheProvider<H> {
	pub fn new() -> Self {
		CacheProvider {
			initialized: Default::default(),
			node_cache: spin::Mutex::new(HashMap::new()),
			value_cache: spin::Mutex::new(HashMap::new()),
		}
	}
}

impl<H: Hasher> TrieCacheProvider<H> for CacheProvider<H> {
	type Cache<'a> = SimpleCache<'a, H> where H: 'a;

	fn as_trie_db_cache(&self, _storage_root: <H as Hasher>::Out) -> Self::Cache<'_> {
		SimpleCache { value_cache: self.value_cache.lock(), node_cache: self.node_cache.lock() }
	}

	fn as_trie_db_mut_cache(&self) -> Self::Cache<'_> {
		SimpleCache { value_cache: self.value_cache.lock(), node_cache: self.node_cache.lock() }
	}

	fn merge<'a>(&'a self, _other: Self::Cache<'a>, _new_root: <H as Hasher>::Out) {}
}
