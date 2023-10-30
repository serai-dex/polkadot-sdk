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

//! Traits for dealing with a single non-fungible item.
//!
//! This assumes a single-level namespace identified by `Inspect::ItemId`, and could
//! reasonably be implemented by pallets that want to expose a single collection of NFT-like
//! objects.
//!
//! For an NFT API that has dual-level namespacing, the traits in `nonfungibles` are better to
//! use.

use crate::dispatch::{DispatchResult, Parameter};
use codec::{Decode, Encode};
use sp_runtime::TokenError;
use sp_std::prelude::*;

/// Trait for providing an interface to a read-only NFT-like item.
pub trait Inspect<AccountId> {
	/// Type for identifying an item.
	type ItemId: Parameter;

	/// Returns the owner of `item`, or `None` if the item doesn't exist or has no
	/// owner.
	fn owner(item: &Self::ItemId) -> Option<AccountId>;

	/// Returns the attribute value of `item` corresponding to `key`.
	///
	/// By default this is `None`; no attributes are defined.
	fn attribute(_item: &Self::ItemId, _key: &[u8]) -> Option<Vec<u8>> {
		None
	}

	/// Returns the custom attribute value of `item` corresponding to `key`.
	///
	/// By default this is `None`; no attributes are defined.
	fn custom_attribute(
		_account: &AccountId,
		_item: &Self::ItemId,
		_key: &[u8],
	) -> Option<Vec<u8>> {
		None
	}

	/// Returns the system attribute value of `item` corresponding to `key`.
	///
	/// By default this is `None`; no attributes are defined.
	fn system_attribute(_item: &Self::ItemId, _key: &[u8]) -> Option<Vec<u8>> {
		None
	}

	/// Returns the strongly-typed attribute value of `item` corresponding to `key`.
	///
	/// By default this just attempts to use `attribute`.
	fn typed_attribute<K: Encode, V: Decode>(item: &Self::ItemId, key: &K) -> Option<V> {
		key.using_encoded(|d| Self::attribute(item, d))
			.and_then(|v| V::decode(&mut &v[..]).ok())
	}

	/// Returns the strongly-typed custom attribute value of `item` corresponding to `key`.
	///
	/// By default this just attempts to use `custom_attribute`.
	fn typed_custom_attribute<K: Encode, V: Decode>(
		account: &AccountId,
		item: &Self::ItemId,
		key: &K,
	) -> Option<V> {
		key.using_encoded(|d| Self::custom_attribute(account, item, d))
			.and_then(|v| V::decode(&mut &v[..]).ok())
	}

	/// Returns the strongly-typed system attribute value of `item` corresponding to `key`.
	///
	/// By default this just attempts to use `system_attribute`.
	fn typed_system_attribute<K: Encode, V: Decode>(item: &Self::ItemId, key: &K) -> Option<V> {
		key.using_encoded(|d| Self::system_attribute(item, d))
			.and_then(|v| V::decode(&mut &v[..]).ok())
	}

	/// Returns `true` if the `item` may be transferred.
	///
	/// Default implementation is that all items are transferable.
	fn can_transfer(_item: &Self::ItemId) -> bool {
		true
	}
}

/// Interface for enumerating items in existence or owned by a given account over a collection
/// of NFTs.
pub trait InspectEnumerable<AccountId>: Inspect<AccountId> {
	/// The iterator type for [`Self::items`].
	type ItemsIterator: Iterator<Item = Self::ItemId>;
	/// The iterator type for [`Self::owned`].
	type OwnedIterator: Iterator<Item = Self::ItemId>;

	/// Returns an iterator of the items within a `collection` in existence.
	fn items() -> Self::ItemsIterator;

	/// Returns an iterator of the items of all collections owned by `who`.
	fn owned(who: &AccountId) -> Self::OwnedIterator;
}

/// Trait for providing an interface for NFT-like items which may be minted, burned and/or have
/// attributes set on them.
pub trait Mutate<AccountId, ItemConfig>: Inspect<AccountId> {
	/// Mint some `item` to be owned by `who`.
	///
	/// By default, this is not a supported operation.
	fn mint_into(
		_item: &Self::ItemId,
		_who: &AccountId,
		_config: &ItemConfig,
		_deposit_collection_owner: bool,
	) -> DispatchResult {
		Err(TokenError::Unsupported.into())
	}

	/// Burn some `item`.
	///
	/// By default, this is not a supported operation.
	fn burn(_item: &Self::ItemId, _maybe_check_owner: Option<&AccountId>) -> DispatchResult {
		Err(TokenError::Unsupported.into())
	}

	/// Set attribute `value` of `item`'s `key`.
	///
	/// By default, this is not a supported operation.
	fn set_attribute(_item: &Self::ItemId, _key: &[u8], _value: &[u8]) -> DispatchResult {
		Err(TokenError::Unsupported.into())
	}

	/// Attempt to set the strongly-typed attribute `value` of `item`'s `key`.
	///
	/// By default this just attempts to use `set_attribute`.
	fn set_typed_attribute<K: Encode, V: Encode>(
		item: &Self::ItemId,
		key: &K,
		value: &V,
	) -> DispatchResult {
		key.using_encoded(|k| value.using_encoded(|v| Self::set_attribute(item, k, v)))
	}

	/// Clear attribute of `item`'s `key`.
	///
	/// By default, this is not a supported operation.
	fn clear_attribute(_item: &Self::ItemId, _key: &[u8]) -> DispatchResult {
		Err(TokenError::Unsupported.into())
	}

	/// Attempt to clear the strongly-typed attribute of `item`'s `key`.
	///
	/// By default this just attempts to use `clear_attribute`.
	fn clear_typed_attribute<K: Encode>(item: &Self::ItemId, key: &K) -> DispatchResult {
		key.using_encoded(|k| Self::clear_attribute(item, k))
	}
}

/// Trait for transferring and controlling the transfer of non-fungible sets of items.
pub trait Transfer<AccountId>: Inspect<AccountId> {
	/// Transfer `item` into `destination` account.
	fn transfer(item: &Self::ItemId, destination: &AccountId) -> DispatchResult;
	/// Disable the `item` of `collection` transfer.
	///
	/// By default, this is not a supported operation.
	fn disable_transfer(item: &Self::ItemId) -> DispatchResult;
	/// Re-enable the `item` of `collection` transfer.
	///
	/// By default, this is not a supported operation.
	fn enable_transfer(item: &Self::ItemId) -> DispatchResult;
}
