// This file is part of a fork of Substrate which has had various changes.

// Copyright (C) Parity Technologies (UK) Ltd.
// Copyright (C) 2022-2023 Luke Parker
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

//! Utilities for offchain calls testing.
//!
//! Namely all ExecutionExtensions that allow mocking
//! the extra APIs.

use crate::{
	offchain::{
		self, storage::InMemOffchainStorage, OffchainOverlayedChange, OffchainStorage,
		OpaqueNetworkState, StorageKind, Timestamp, TransactionPool,
	},
	OpaquePeerId,
};
use std::sync::Arc;

use parking_lot::RwLock;

/// Sharable "persistent" offchain storage for test.
#[derive(Debug, Clone, Default)]
pub struct TestPersistentOffchainDB {
	persistent: Arc<RwLock<InMemOffchainStorage>>,
}

impl TestPersistentOffchainDB {
	const PREFIX: &'static [u8] = b"";

	/// Create a new and empty offchain storage db for persistent items
	pub fn new() -> Self {
		Self { persistent: Arc::new(RwLock::new(InMemOffchainStorage::default())) }
	}

	/// Apply a set of off-chain changes directly to the test backend
	pub fn apply_offchain_changes(
		&mut self,
		changes: impl Iterator<Item = ((Vec<u8>, Vec<u8>), OffchainOverlayedChange)>,
	) {
		let mut me = self.persistent.write();
		for ((_prefix, key), value_operation) in changes {
			match value_operation {
				OffchainOverlayedChange::SetValue(val) =>
					me.set(Self::PREFIX, key.as_slice(), val.as_slice()),
				OffchainOverlayedChange::Remove => me.remove(Self::PREFIX, key.as_slice()),
			}
		}
	}

	/// Retrieve a key from the test backend.
	pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		OffchainStorage::get(self, Self::PREFIX, key)
	}
}

impl OffchainStorage for TestPersistentOffchainDB {
	fn set(&mut self, prefix: &[u8], key: &[u8], value: &[u8]) {
		self.persistent.write().set(prefix, key, value);
	}

	fn remove(&mut self, prefix: &[u8], key: &[u8]) {
		self.persistent.write().remove(prefix, key);
	}

	fn get(&self, prefix: &[u8], key: &[u8]) -> Option<Vec<u8>> {
		self.persistent.read().get(prefix, key)
	}

	fn compare_and_set(
		&mut self,
		prefix: &[u8],
		key: &[u8],
		old_value: Option<&[u8]>,
		new_value: &[u8],
	) -> bool {
		self.persistent.write().compare_and_set(prefix, key, old_value, new_value)
	}
}

/// Internal state of the externalities.
///
/// This can be used in tests to respond or assert stuff about interactions.
#[derive(Debug, Default)]
pub struct OffchainState {
	/// Persistent local storage
	pub persistent_storage: TestPersistentOffchainDB,
	/// Local storage
	pub local_storage: InMemOffchainStorage,
	/// A supposedly random seed.
	pub seed: [u8; 32],
	/// A timestamp simulating the current time.
	pub timestamp: Timestamp,
}

/// Implementation of offchain externalities used for tests.
#[derive(Clone, Default, Debug)]
pub struct TestOffchainExt(pub Arc<RwLock<OffchainState>>);

impl TestOffchainExt {
	/// Create new `TestOffchainExt` and a reference to the internal state.
	pub fn new() -> (Self, Arc<RwLock<OffchainState>>) {
		let ext = Self::default();
		let state = ext.0.clone();
		(ext, state)
	}

	/// Create new `TestOffchainExt` and a reference to the internal state.
	pub fn with_offchain_db(
		offchain_db: TestPersistentOffchainDB,
	) -> (Self, Arc<RwLock<OffchainState>>) {
		let (ext, state) = Self::new();
		ext.0.write().persistent_storage = offchain_db;
		(ext, state)
	}
}

impl offchain::Externalities for TestOffchainExt {
	fn is_validator(&self) -> bool {
		true
	}

	fn network_state(&self) -> Result<OpaqueNetworkState, ()> {
		Ok(OpaqueNetworkState { peer_id: Default::default(), external_addresses: vec![] })
	}

	fn timestamp(&mut self) -> Timestamp {
		self.0.read().timestamp
	}

	fn sleep_until(&mut self, deadline: Timestamp) {
		self.0.write().timestamp = deadline;
	}

	fn random_seed(&mut self) -> [u8; 32] {
		self.0.read().seed
	}

	fn set_authorized_nodes(&mut self, _nodes: Vec<OpaquePeerId>, _authorized_only: bool) {
		unimplemented!()
	}
}

impl offchain::DbExternalities for TestOffchainExt {
	fn local_storage_set(&mut self, kind: StorageKind, key: &[u8], value: &[u8]) {
		let mut state = self.0.write();
		match kind {
			StorageKind::LOCAL => state.local_storage.set(b"", key, value),
			StorageKind::PERSISTENT => state.persistent_storage.set(b"", key, value),
		};
	}

	fn local_storage_clear(&mut self, kind: StorageKind, key: &[u8]) {
		let mut state = self.0.write();
		match kind {
			StorageKind::LOCAL => state.local_storage.remove(b"", key),
			StorageKind::PERSISTENT => state.persistent_storage.remove(b"", key),
		};
	}

	fn local_storage_compare_and_set(
		&mut self,
		kind: StorageKind,
		key: &[u8],
		old_value: Option<&[u8]>,
		new_value: &[u8],
	) -> bool {
		let mut state = self.0.write();
		match kind {
			StorageKind::LOCAL =>
				state.local_storage.compare_and_set(b"", key, old_value, new_value),
			StorageKind::PERSISTENT =>
				state.persistent_storage.compare_and_set(b"", key, old_value, new_value),
		}
	}

	fn local_storage_get(&mut self, kind: StorageKind, key: &[u8]) -> Option<Vec<u8>> {
		let state = self.0.read();
		match kind {
			StorageKind::LOCAL => state.local_storage.get(TestPersistentOffchainDB::PREFIX, key),
			StorageKind::PERSISTENT => state.persistent_storage.get(key),
		}
	}
}

/// The internal state of the fake transaction pool.
#[derive(Default)]
pub struct PoolState {
	/// A vector of transactions submitted from the runtime.
	pub transactions: Vec<Vec<u8>>,
}

/// Implementation of transaction pool used for test.
///
/// Note that this implementation does not verify correctness
/// of sent extrinsics. It's meant to be used in contexts
/// where an actual runtime is not known.
///
/// It's advised to write integration tests that include the
/// actual transaction pool to make sure the produced
/// transactions are valid.
#[derive(Default)]
pub struct TestTransactionPoolExt(Arc<RwLock<PoolState>>);

impl TestTransactionPoolExt {
	/// Create new `TestTransactionPoolExt` and a reference to the internal state.
	pub fn new() -> (Self, Arc<RwLock<PoolState>>) {
		let ext = Self::default();
		let state = ext.0.clone();
		(ext, state)
	}
}

impl TransactionPool for TestTransactionPoolExt {
	fn submit_transaction(&mut self, extrinsic: Vec<u8>) -> Result<(), ()> {
		self.0.write().transactions.push(extrinsic);
		Ok(())
	}
}
