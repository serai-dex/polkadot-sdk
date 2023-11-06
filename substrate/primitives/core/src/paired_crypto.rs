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

//! API for using a pair of crypto schemes together.

#[cfg(feature = "serde")]
use crate::crypto::Ss58Codec;
use crate::crypto::{ByteArray, CryptoType, Derive, Public as PublicT, UncheckedFrom};
#[cfg(feature = "full_crypto")]
use crate::crypto::{DeriveError, DeriveJunction, Pair as PairT, SecretStringError};

#[cfg(feature = "full_crypto")]
use sp_std::vec::Vec;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
#[cfg(all(not(feature = "std"), feature = "serde"))]
use sp_std::alloc::{format, string::String};

use sp_runtime_interface::pass_by::{self, PassBy, PassByInner};
use sp_std::convert::TryFrom;

/// Secure seed length.
///
/// Currently only supporting sub-schemes whose seed is a 32-bytes array.
#[cfg(feature = "full_crypto")]
const SECURE_SEED_LEN: usize = 32;

/// A secret seed.
///
/// It's not called a "secret key" because ring doesn't expose the secret keys
/// of the key pair (yeah, dumb); as such we're forced to remember the seed manually if we
/// will need it later (such as for HDKD).
#[cfg(feature = "full_crypto")]
type Seed = [u8; SECURE_SEED_LEN];

/// A public key.
#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, PartialOrd, Ord)]
pub struct Public<const LEFT_PLUS_RIGHT_LEN: usize>([u8; LEFT_PLUS_RIGHT_LEN]);

#[cfg(feature = "full_crypto")]
impl<const LEFT_PLUS_RIGHT_LEN: usize> sp_std::hash::Hash for Public<LEFT_PLUS_RIGHT_LEN> {
	fn hash<H: sp_std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state);
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> ByteArray for Public<LEFT_PLUS_RIGHT_LEN> {
	const LEN: usize = LEFT_PLUS_RIGHT_LEN;
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> TryFrom<&[u8]> for Public<LEFT_PLUS_RIGHT_LEN> {
	type Error = ();

	fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
		if data.len() != LEFT_PLUS_RIGHT_LEN {
			return Err(())
		}
		let mut inner = [0u8; LEFT_PLUS_RIGHT_LEN];
		inner.copy_from_slice(data);
		Ok(Public(inner))
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> AsRef<[u8; LEFT_PLUS_RIGHT_LEN]>
	for Public<LEFT_PLUS_RIGHT_LEN>
{
	fn as_ref(&self) -> &[u8; LEFT_PLUS_RIGHT_LEN] {
		&self.0
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> AsRef<[u8]> for Public<LEFT_PLUS_RIGHT_LEN> {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> AsMut<[u8]> for Public<LEFT_PLUS_RIGHT_LEN> {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.0[..]
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> PassByInner for Public<LEFT_PLUS_RIGHT_LEN> {
	type Inner = [u8; LEFT_PLUS_RIGHT_LEN];

	fn into_inner(self) -> Self::Inner {
		self.0
	}

	fn inner(&self) -> &Self::Inner {
		&self.0
	}

	fn from_inner(inner: Self::Inner) -> Self {
		Self(inner)
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> PassBy for Public<LEFT_PLUS_RIGHT_LEN> {
	type PassBy = pass_by::Inner<Self, [u8; LEFT_PLUS_RIGHT_LEN]>;
}

#[cfg(feature = "full_crypto")]
impl<
		LeftPair: PairT,
		RightPair: PairT,
		const LEFT_PLUS_RIGHT_PUBLIC_LEN: usize,
		const SIGNATURE_LEN: usize,
	> From<Pair<LeftPair, RightPair, LEFT_PLUS_RIGHT_PUBLIC_LEN, SIGNATURE_LEN>>
	for Public<LEFT_PLUS_RIGHT_PUBLIC_LEN>
where
	Pair<LeftPair, RightPair, LEFT_PLUS_RIGHT_PUBLIC_LEN, SIGNATURE_LEN>:
		PairT<Public = Public<LEFT_PLUS_RIGHT_PUBLIC_LEN>>,
{
	fn from(x: Pair<LeftPair, RightPair, LEFT_PLUS_RIGHT_PUBLIC_LEN, SIGNATURE_LEN>) -> Self {
		x.public()
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> UncheckedFrom<[u8; LEFT_PLUS_RIGHT_LEN]>
	for Public<LEFT_PLUS_RIGHT_LEN>
{
	fn unchecked_from(data: [u8; LEFT_PLUS_RIGHT_LEN]) -> Self {
		Public(data)
	}
}

#[cfg(feature = "std")]
impl<const LEFT_PLUS_RIGHT_LEN: usize> std::fmt::Display for Public<LEFT_PLUS_RIGHT_LEN>
where
	Public<LEFT_PLUS_RIGHT_LEN>: CryptoType,
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.to_ss58check())
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> sp_std::fmt::Debug for Public<LEFT_PLUS_RIGHT_LEN>
where
	Public<LEFT_PLUS_RIGHT_LEN>: CryptoType,
	[u8; LEFT_PLUS_RIGHT_LEN]: crate::hexdisplay::AsBytesRef,
{
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		let s = self.to_ss58check();
		write!(f, "{} ({}...)", crate::hexdisplay::HexDisplay::from(&self.0), &s[0..8])
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

#[cfg(feature = "serde")]
impl<const LEFT_PLUS_RIGHT_LEN: usize> Serialize for Public<LEFT_PLUS_RIGHT_LEN>
where
	Public<LEFT_PLUS_RIGHT_LEN>: CryptoType,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_ss58check())
	}
}

#[cfg(feature = "serde")]
impl<'de, const LEFT_PLUS_RIGHT_LEN: usize> Deserialize<'de> for Public<LEFT_PLUS_RIGHT_LEN>
where
	Public<LEFT_PLUS_RIGHT_LEN>: CryptoType,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Public::from_ss58check(&String::deserialize(deserializer)?)
			.map_err(|e| de::Error::custom(format!("{:?}", e)))
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> PublicT for Public<LEFT_PLUS_RIGHT_LEN> where
	Public<LEFT_PLUS_RIGHT_LEN>: CryptoType
{
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> Derive for Public<LEFT_PLUS_RIGHT_LEN> {}

/// Trait characterizing a signature which could be used as individual component of an
/// `paired_crypto:Signature` pair.
pub trait SignatureBound: ByteArray {}

impl<T: ByteArray> SignatureBound for T {}

/// A pair of signatures of different types
#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq)]
pub struct Signature<const LEFT_PLUS_RIGHT_LEN: usize>([u8; LEFT_PLUS_RIGHT_LEN]);

#[cfg(feature = "full_crypto")]
impl<const LEFT_PLUS_RIGHT_LEN: usize> sp_std::hash::Hash for Signature<LEFT_PLUS_RIGHT_LEN> {
	fn hash<H: sp_std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state);
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> ByteArray for Signature<LEFT_PLUS_RIGHT_LEN> {
	const LEN: usize = LEFT_PLUS_RIGHT_LEN;
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> TryFrom<&[u8]> for Signature<LEFT_PLUS_RIGHT_LEN> {
	type Error = ();

	fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
		if data.len() != LEFT_PLUS_RIGHT_LEN {
			return Err(())
		}
		let mut inner = [0u8; LEFT_PLUS_RIGHT_LEN];
		inner.copy_from_slice(data);
		Ok(Signature(inner))
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> AsMut<[u8]> for Signature<LEFT_PLUS_RIGHT_LEN> {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.0[..]
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> AsRef<[u8; LEFT_PLUS_RIGHT_LEN]>
	for Signature<LEFT_PLUS_RIGHT_LEN>
{
	fn as_ref(&self) -> &[u8; LEFT_PLUS_RIGHT_LEN] {
		&self.0
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> AsRef<[u8]> for Signature<LEFT_PLUS_RIGHT_LEN> {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

#[cfg(feature = "serde")]
impl<const LEFT_PLUS_RIGHT_LEN: usize> Serialize for Signature<LEFT_PLUS_RIGHT_LEN> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&array_bytes::bytes2hex("", self))
	}
}

#[cfg(feature = "serde")]
impl<'de, const LEFT_PLUS_RIGHT_LEN: usize> Deserialize<'de> for Signature<LEFT_PLUS_RIGHT_LEN> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let bytes = array_bytes::hex2bytes(&String::deserialize(deserializer)?)
			.map_err(|e| de::Error::custom(format!("{:?}", e)))?;
		Signature::<LEFT_PLUS_RIGHT_LEN>::try_from(bytes.as_ref()).map_err(|e| {
			de::Error::custom(format!("Error converting deserialized data into signature: {:?}", e))
		})
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> From<Signature<LEFT_PLUS_RIGHT_LEN>>
	for [u8; LEFT_PLUS_RIGHT_LEN]
{
	fn from(signature: Signature<LEFT_PLUS_RIGHT_LEN>) -> [u8; LEFT_PLUS_RIGHT_LEN] {
		signature.0
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> sp_std::fmt::Debug for Signature<LEFT_PLUS_RIGHT_LEN>
where
	[u8; LEFT_PLUS_RIGHT_LEN]: crate::hexdisplay::AsBytesRef,
{
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "{}", crate::hexdisplay::HexDisplay::from(&self.0))
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<const LEFT_PLUS_RIGHT_LEN: usize> UncheckedFrom<[u8; LEFT_PLUS_RIGHT_LEN]>
	for Signature<LEFT_PLUS_RIGHT_LEN>
{
	fn unchecked_from(data: [u8; LEFT_PLUS_RIGHT_LEN]) -> Self {
		Signature(data)
	}
}

/// A key pair.
#[cfg(feature = "full_crypto")]
#[derive(Clone)]
pub struct Pair<
	LeftPair: PairT,
	RightPair: PairT,
	const PUBLIC_KEY_LEN: usize,
	const SIGNATURE_LEN: usize,
> {
	left: LeftPair,
	right: RightPair,
}

#[cfg(feature = "full_crypto")]
impl<
		LeftPair: PairT,
		RightPair: PairT,
		const PUBLIC_KEY_LEN: usize,
		const SIGNATURE_LEN: usize,
	> PairT for Pair<LeftPair, RightPair, PUBLIC_KEY_LEN, SIGNATURE_LEN>
where
	Pair<LeftPair, RightPair, PUBLIC_KEY_LEN, SIGNATURE_LEN>: CryptoType,
	LeftPair::Signature: SignatureBound,
	RightPair::Signature: SignatureBound,
	Public<PUBLIC_KEY_LEN>: CryptoType,
	LeftPair::Seed: From<Seed> + Into<Seed>,
	RightPair::Seed: From<Seed> + Into<Seed>,
{
	type Seed = Seed;
	type Public = Public<PUBLIC_KEY_LEN>;
	type Signature = Signature<SIGNATURE_LEN>;

	fn from_seed_slice(seed_slice: &[u8]) -> Result<Self, SecretStringError> {
		if seed_slice.len() != SECURE_SEED_LEN {
			return Err(SecretStringError::InvalidSeedLength)
		}
		let left = LeftPair::from_seed_slice(&seed_slice)?;
		let right = RightPair::from_seed_slice(&seed_slice)?;
		Ok(Pair { left, right })
	}

	/// Derive a child key from a series of given junctions.
	///
	/// Note: if the `LeftPair` and `RightPair` crypto schemes differ in
	/// seed derivation, `derive` will drop the seed in the return.
	fn derive<Iter: Iterator<Item = DeriveJunction>>(
		&self,
		path: Iter,
		seed: Option<Self::Seed>,
	) -> Result<(Self, Option<Self::Seed>), DeriveError> {
		let path: Vec<_> = path.collect();

		let left = self.left.derive(path.iter().cloned(), seed.map(|s| s.into()))?;
		let right = self.right.derive(path.into_iter(), seed.map(|s| s.into()))?;

		let seed = match (left.1, right.1) {
			(Some(l), Some(r)) if l.as_ref() == r.as_ref() => Some(l.into()),
			_ => None,
		};

		Ok((Self { left: left.0, right: right.0 }, seed))
	}

	fn public(&self) -> Self::Public {
		let mut raw = [0u8; PUBLIC_KEY_LEN];
		let left_pub = self.left.public();
		let right_pub = self.right.public();
		raw[..LeftPair::Public::LEN].copy_from_slice(left_pub.as_ref());
		raw[LeftPair::Public::LEN..].copy_from_slice(right_pub.as_ref());
		Self::Public::unchecked_from(raw)
	}

	fn sign(&self, message: &[u8]) -> Self::Signature {
		let mut raw: [u8; SIGNATURE_LEN] = [0u8; SIGNATURE_LEN];
		raw[..LeftPair::Signature::LEN].copy_from_slice(self.left.sign(message).as_ref());
		raw[LeftPair::Signature::LEN..].copy_from_slice(self.right.sign(message).as_ref());
		Self::Signature::unchecked_from(raw)
	}

	fn verify<M: AsRef<[u8]>>(sig: &Self::Signature, message: M, public: &Self::Public) -> bool {
		let Ok(left_pub) = public.0[..LeftPair::Public::LEN].try_into() else { return false };
		let Ok(left_sig) = sig.0[0..LeftPair::Signature::LEN].try_into() else { return false };
		if !LeftPair::verify(&left_sig, message.as_ref(), &left_pub) {
			return false
		}

		let Ok(right_pub) = public.0[LeftPair::Public::LEN..PUBLIC_KEY_LEN].try_into() else {
			return false
		};
		let Ok(right_sig) = sig.0[LeftPair::Signature::LEN..].try_into() else { return false };
		RightPair::verify(&right_sig, message.as_ref(), &right_pub)
	}

	/// Get the seed/secret key for each key and then concatenate them.
	fn to_raw_vec(&self) -> Vec<u8> {
		let mut raw = self.left.to_raw_vec();
		raw.extend(self.right.to_raw_vec());
		raw
	}
}
