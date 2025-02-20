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

//! Hashing Functions.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::hash::Hasher;

use byteorder::{ByteOrder, LittleEndian};
use digest::Digest;

#[inline(always)]
fn blake2<const N: usize>(data: &[u8]) -> [u8; N] {
	blake2b_simd::Params::new()
		.hash_length(N)
		.hash(data)
		.as_bytes()
		.try_into()
		.expect("slice is always the necessary length")
}

/// Do a Blake2 512-bit hash and place result in `dest`.
pub fn blake2_512_into(data: &[u8], dest: &mut [u8; 64]) {
	*dest = blake2(data);
}

/// Do a Blake2 512-bit hash and return result.
pub fn blake2_512(data: &[u8]) -> [u8; 64] {
	blake2(data)
}

/// Do a Blake2 256-bit hash and return result.
pub fn blake2_256(data: &[u8]) -> [u8; 32] {
	blake2(data)
}

/// Do a Blake2 128-bit hash and return result.
pub fn blake2_128(data: &[u8]) -> [u8; 16] {
	blake2(data)
}

/// Do a Blake2 64-bit hash and return result.
pub fn blake2_64(data: &[u8]) -> [u8; 8] {
	blake2(data)
}

/// Do a XX 64-bit hash and place result in `dest`.
pub fn twox_64_into(data: &[u8], dest: &mut [u8; 8]) {
	let r0 = twox_hash::XxHash::with_seed(0).chain_update(data).finish();
	LittleEndian::write_u64(&mut dest[0..8], r0);
}

/// Do a XX 64-bit hash and return result.
pub fn twox_64(data: &[u8]) -> [u8; 8] {
	let mut r: [u8; 8] = [0; 8];
	twox_64_into(data, &mut r);
	r
}

/// Do a XX 128-bit hash and place result in `dest`.
pub fn twox_128_into(data: &[u8], dest: &mut [u8; 16]) {
	let r0 = twox_hash::XxHash::with_seed(0).chain_update(data).finish();
	let r1 = twox_hash::XxHash::with_seed(1).chain_update(data).finish();
	LittleEndian::write_u64(&mut dest[0..8], r0);
	LittleEndian::write_u64(&mut dest[8..16], r1);
}

/// Do a XX 128-bit hash and return result.
pub fn twox_128(data: &[u8]) -> [u8; 16] {
	let mut r: [u8; 16] = [0; 16];
	twox_128_into(data, &mut r);
	r
}

/// Do a XX 256-bit hash and place result in `dest`.
pub fn twox_256_into(data: &[u8], dest: &mut [u8; 32]) {
	let r0 = twox_hash::XxHash::with_seed(0).chain_update(data).finish();
	let r1 = twox_hash::XxHash::with_seed(1).chain_update(data).finish();
	let r2 = twox_hash::XxHash::with_seed(2).chain_update(data).finish();
	let r3 = twox_hash::XxHash::with_seed(3).chain_update(data).finish();
	LittleEndian::write_u64(&mut dest[0..8], r0);
	LittleEndian::write_u64(&mut dest[8..16], r1);
	LittleEndian::write_u64(&mut dest[16..24], r2);
	LittleEndian::write_u64(&mut dest[24..32], r3);
}

/// Do a XX 256-bit hash and return result.
pub fn twox_256(data: &[u8]) -> [u8; 32] {
	let mut r: [u8; 32] = [0; 32];
	twox_256_into(data, &mut r);
	r
}

/// Do a keccak 256-bit hash and return result.
pub fn keccak_256(data: &[u8]) -> [u8; 32] {
	sha3::Keccak256::digest(data).into()
}

/// Do a keccak 512-bit hash and return result.
pub fn keccak_512(data: &[u8]) -> [u8; 64] {
	sha3::Keccak512::digest(data).into()
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn blake2b() {
		assert_eq!(sp_crypto_hashing_proc_macro::blake2b_64!(b""), blake2_64(b"")[..]);
		assert_eq!(sp_crypto_hashing_proc_macro::blake2b_256!(b"test"), blake2_256(b"test")[..]);
		assert_eq!(sp_crypto_hashing_proc_macro::blake2b_512!(b""), blake2_512(b"")[..]);
	}

	#[test]
	fn keccak() {
		assert_eq!(sp_crypto_hashing_proc_macro::keccak_256!(b"test"), keccak_256(b"test")[..]);
		assert_eq!(sp_crypto_hashing_proc_macro::keccak_512!(b"test"), keccak_512(b"test")[..]);
	}

	#[test]
	fn twox() {
		assert_eq!(sp_crypto_hashing_proc_macro::twox_128!(b"test"), twox_128(b"test")[..]);
		assert_eq!(sp_crypto_hashing_proc_macro::twox_64!(b""), twox_64(b"")[..]);
	}

	#[test]
	fn twox_concats() {
		assert_eq!(
			sp_crypto_hashing_proc_macro::twox_128!(b"test", b"123", b"45", b"", b"67890"),
			twox_128(&b"test1234567890"[..]),
		);
		assert_eq!(
			sp_crypto_hashing_proc_macro::twox_128!(b"test", test, b"45", b"", b"67890"),
			twox_128(&b"testtest4567890"[..]),
		);
	}
}
