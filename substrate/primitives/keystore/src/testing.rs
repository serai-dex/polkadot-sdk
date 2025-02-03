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

//! Types that should only be used for testing!

use crate::{Error, Keystore, KeystorePtr};

use sp_core::{
	crypto::{ByteArray, KeyTypeId, Pair, VrfSecret},
	sr25519,
};

use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

/// A keystore implementation usable in tests.
#[derive(Default, Clone)]
pub struct MemoryKeystore {
	/// `KeyTypeId` maps to public keys and public keys map to private keys.
	keys: Arc<RwLock<HashMap<KeyTypeId, HashMap<Vec<u8>, String>>>>,
}

impl MemoryKeystore {
	/// Creates a new instance of `Self`.
	pub fn new() -> Self {
		Self::default()
	}

	fn pair<T: Pair>(&self, key_type: KeyTypeId, public: &T::Public) -> Option<T> {
		self.keys.read().get(&key_type).and_then(|inner| {
			inner
				.get(public.as_slice())
				.map(|s| T::from_string(s, None).expect("seed slice is valid"))
		})
	}

	fn public_keys<T: Pair>(&self, key_type: KeyTypeId) -> Vec<T::Public> {
		self.keys
			.read()
			.get(&key_type)
			.map(|keys| {
				keys.values()
					.map(|s| T::from_string(s, None).expect("seed slice is valid"))
					.map(|p| p.public())
					.collect()
			})
			.unwrap_or_default()
	}

	fn generate_new<T: Pair>(
		&self,
		key_type: KeyTypeId,
		seed: Option<&str>,
	) -> Result<T::Public, Error> {
		match seed {
			Some(seed) => {
				let pair = T::from_string(seed, None)
					.map_err(|_| Error::ValidationError("Generates a pair.".to_owned()))?;
				self.keys
					.write()
					.entry(key_type)
					.or_default()
					.insert(pair.public().to_raw_vec(), seed.into());
				Ok(pair.public())
			},
			None => {
				let (pair, phrase, _) = T::generate_with_phrase(None);
				self.keys
					.write()
					.entry(key_type)
					.or_default()
					.insert(pair.public().to_raw_vec(), phrase);
				Ok(pair.public())
			},
		}
	}

	fn sign<T: Pair>(
		&self,
		key_type: KeyTypeId,
		public: &T::Public,
		msg: &[u8],
	) -> Result<Option<T::Signature>, Error> {
		let sig = self.pair::<T>(key_type, public).map(|pair| pair.sign(msg));
		Ok(sig)
	}

	fn vrf_sign<T: Pair + VrfSecret>(
		&self,
		key_type: KeyTypeId,
		public: &T::Public,
		data: &T::VrfSignData,
	) -> Result<Option<T::VrfSignature>, Error> {
		let sig = self.pair::<T>(key_type, public).map(|pair| pair.vrf_sign(data));
		Ok(sig)
	}

	fn vrf_pre_output<T: Pair + VrfSecret>(
		&self,
		key_type: KeyTypeId,
		public: &T::Public,
		input: &T::VrfInput,
	) -> Result<Option<T::VrfPreOutput>, Error> {
		let pre_output = self.pair::<T>(key_type, public).map(|pair| pair.vrf_pre_output(input));
		Ok(pre_output)
	}
}

impl Keystore for MemoryKeystore {
	fn sr25519_public_keys(&self, key_type: KeyTypeId) -> Vec<sr25519::Public> {
		self.public_keys::<sr25519::Pair>(key_type)
	}

	fn sr25519_generate_new(
		&self,
		key_type: KeyTypeId,
		seed: Option<&str>,
	) -> Result<sr25519::Public, Error> {
		self.generate_new::<sr25519::Pair>(key_type, seed)
	}

	fn sr25519_sign(
		&self,
		key_type: KeyTypeId,
		public: &sr25519::Public,
		msg: &[u8],
	) -> Result<Option<sr25519::Signature>, Error> {
		self.sign::<sr25519::Pair>(key_type, public, msg)
	}

	fn sr25519_vrf_sign(
		&self,
		key_type: KeyTypeId,
		public: &sr25519::Public,
		data: &sr25519::vrf::VrfSignData,
	) -> Result<Option<sr25519::vrf::VrfSignature>, Error> {
		self.vrf_sign::<sr25519::Pair>(key_type, public, data)
	}

	fn sr25519_vrf_pre_output(
		&self,
		key_type: KeyTypeId,
		public: &sr25519::Public,
		input: &sr25519::vrf::VrfInput,
	) -> Result<Option<sr25519::vrf::VrfPreOutput>, Error> {
		self.vrf_pre_output::<sr25519::Pair>(key_type, public, input)
	}

	fn insert(&self, key_type: KeyTypeId, suri: &str, public: &[u8]) -> Result<(), ()> {
		self.keys
			.write()
			.entry(key_type)
			.or_default()
			.insert(public.to_owned(), suri.to_string());
		Ok(())
	}

	fn keys(&self, key_type: KeyTypeId) -> Result<Vec<Vec<u8>>, Error> {
		let keys = self
			.keys
			.read()
			.get(&key_type)
			.map(|map| map.keys().cloned().collect())
			.unwrap_or_default();
		Ok(keys)
	}

	fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
		public_keys
			.iter()
			.all(|(k, t)| self.keys.read().get(t).and_then(|s| s.get(k)).is_some())
	}
}

impl Into<KeystorePtr> for MemoryKeystore {
	fn into(self) -> KeystorePtr {
		Arc::new(self)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_core::{sr25519, testing::SR25519};

	#[test]
	fn store_key_and_extract() {
		let store = MemoryKeystore::new();

		let public = store.sr25519_generate_new(SR25519, None).expect("Generates key");

		let public_keys = store.sr25519_public_keys(SR25519);

		assert!(public_keys.contains(&public.into()));
	}

	#[test]
	fn store_unknown_and_extract_it() {
		let store = MemoryKeystore::new();

		let secret_uri = "//Alice";
		let key_pair = sr25519::Pair::from_string(secret_uri, None).expect("Generates key pair");

		store
			.insert(SR25519, secret_uri, key_pair.public().as_ref())
			.expect("Inserts unknown key");

		let public_keys = store.sr25519_public_keys(SR25519);

		assert!(public_keys.contains(&key_pair.public().into()));
	}

	#[test]
	fn sr25519_vrf_sign() {
		let store = MemoryKeystore::new();

		let secret_uri = "//Alice";
		let key_pair = sr25519::Pair::from_string(secret_uri, None).expect("Generates key pair");

		let data = sr25519::vrf::VrfInput::new(
			b"Test",
			&[
				(b"one", &1_u64.to_le_bytes()),
				(b"two", &2_u64.to_le_bytes()),
				(b"three", "test".as_bytes()),
			],
		)
		.into_sign_data();

		let result = store.sr25519_vrf_sign(SR25519, &key_pair.public(), &data);
		assert!(result.unwrap().is_none());

		store
			.insert(SR25519, secret_uri, key_pair.public().as_ref())
			.expect("Inserts unknown key");

		let result = store.sr25519_vrf_sign(SR25519, &key_pair.public(), &data);

		assert!(result.unwrap().is_some());
	}

	#[test]
	fn sr25519_vrf_pre_output() {
		let store = MemoryKeystore::new();

		let secret_uri = "//Alice";
		let pair = sr25519::Pair::from_string(secret_uri, None).expect("Generates key pair");

		let input = sr25519::vrf::VrfInput::new(
			b"Test",
			&[
				(b"one", &1_u64.to_le_bytes()),
				(b"two", &2_u64.to_le_bytes()),
				(b"three", "test".as_bytes()),
			],
		);

		let result = store.sr25519_vrf_pre_output(SR25519, &pair.public(), &input);
		assert!(result.unwrap().is_none());

		store
			.insert(SR25519, secret_uri, pair.public().as_ref())
			.expect("Inserts unknown key");

		let pre_output =
			store.sr25519_vrf_pre_output(SR25519, &pair.public(), &input).unwrap().unwrap();

		let result = pre_output.make_bytes::<32>(b"rand", &input, &pair.public());
		assert!(result.is_ok());
	}
}
