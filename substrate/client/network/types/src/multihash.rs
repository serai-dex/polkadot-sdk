// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! [`Multihash`] implemenattion used by substrate. Currently it's a wrapper over
//! multihash used by litep2p, but it can be switched to other implementation if needed.

use multihash::{
	Code as CoreCode, Error as CoreError, Multihash as CoreMultihash, MultihashDigest as _,
};
use std::fmt::{self, Debug};

/// Default [`Multihash`] implementations. Only hashes used by substrate are defined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
	/// Identity hasher.
	Identity,
	/// SHA-256 (32-byte hash size).
	Sha2_256,
}

impl Code {
	/// Calculate digest using this [`Code`]'s hashing algorithm.
	pub fn digest(&self, input: &[u8]) -> Multihash {
		CoreCode::from(*self).digest(input).into()
	}
}

/// Error generated when converting to [`Code`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// Invalid multihash size.
	#[error("invalid multihash size '{0}'")]
	InvalidSize(u64),
	/// The multihash code is not supported.
	#[error("unsupported multihash code '{0:x}'")]
	UnsupportedCode(u64),
	/// Catch-all for other errors emitted when converting `u64` code to enum or parsing multihash
	/// from bytes. Never generated as of multihash-0.17.0.
	#[error("other error: {0}")]
	Other(Box<dyn std::error::Error + Send + Sync>),
}

impl From<CoreError> for Error {
	fn from(error: CoreError) -> Self {
		match error {
			CoreError::InvalidSize(s) => Self::InvalidSize(s),
			CoreError::UnsupportedCode(c) => Self::UnsupportedCode(c),
			e => Self::Other(Box::new(e)),
		}
	}
}

impl From<Code> for CoreCode {
	fn from(code: Code) -> Self {
		match code {
			Code::Identity => CoreCode::Identity,
			Code::Sha2_256 => CoreCode::Sha2_256,
		}
	}
}

impl TryFrom<CoreCode> for Code {
	type Error = Error;

	fn try_from(code: CoreCode) -> Result<Self, Self::Error> {
		match code {
			CoreCode::Identity => Ok(Code::Identity),
			CoreCode::Sha2_256 => Ok(Code::Sha2_256),
			_ => Err(Error::UnsupportedCode(code.into())),
		}
	}
}

impl TryFrom<u64> for Code {
	type Error = Error;

	fn try_from(code: u64) -> Result<Self, Self::Error> {
		match CoreCode::try_from(code) {
			Ok(code) => code.try_into(),
			Err(e) => Err(e.into()),
		}
	}
}

impl From<Code> for u64 {
	fn from(code: Code) -> Self {
		CoreCode::from(code).into()
	}
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Multihash {
	multihash: CoreMultihash,
}

impl Multihash {
	/// Multihash code.
	pub fn code(&self) -> u64 {
		self.multihash.code()
	}

	/// Multihash digest.
	pub fn digest(&self) -> &[u8] {
		self.multihash.digest()
	}

	/// Wraps the digest in a multihash.
	pub fn wrap(code: u64, input_digest: &[u8]) -> Result<Self, Error> {
		CoreMultihash::wrap(code, input_digest).map(Into::into).map_err(Into::into)
	}

	/// Parses a multihash from bytes.
	///
	/// You need to make sure the passed in bytes have the length of 64.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
		CoreMultihash::from_bytes(bytes).map(Into::into).map_err(Into::into)
	}

	/// Returns the bytes of a multihash.
	pub fn to_bytes(&self) -> Vec<u8> {
		self.multihash.to_bytes()
	}
}

/// Remove extra layer of nestedness by deferring to the wrapped value's [`Debug`].
impl Debug for Multihash {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.multihash, f)
	}
}

// From/Into the multihash 0.18 we wrap
impl From<CoreMultihash> for Multihash {
	fn from(multihash: CoreMultihash) -> Self {
		Multihash { multihash }
	}
}

impl From<Multihash> for CoreMultihash {
	fn from(multihash: Multihash) -> Self {
		multihash.multihash
	}
}

// From/Into libp2p's multihash
impl From<libp2p_core::multihash::Multihash<64>> for Multihash {
	fn from(generic: libp2p_core::multihash::Multihash<64>) -> Self {
		CoreMultihash::wrap(generic.code(), generic.digest())
			.expect("both have size 64; qed")
			.into()
	}
}

impl From<Multihash> for libp2p_core::multihash::Multihash<64> {
	fn from(multihash: Multihash) -> Self {
		libp2p_core::multihash::Multihash::wrap(multihash.code(), multihash.digest())
			.expect("both have size 64; qed")
	}
}

// From/Into litep2p's multihash
#[cfg(feature = "litep2p")]
impl From<litep2p::types::multihash::Multihash> for Multihash {
	fn from(generic: litep2p::types::multihash::Multihash) -> Self {
		CoreMultihash::wrap(generic.code(), generic.digest())
			.expect("both have size 64; qed")
			.into()
	}
}

#[cfg(feature = "litep2p")]
impl From<Multihash> for litep2p::types::multihash::Multihash {
	fn from(multihash: Multihash) -> Self {
		litep2p::types::multihash::Multihash::wrap(multihash.code(), multihash.digest())
			.expect("both have size 64; qed")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn code_from_u64() {
		assert_eq!(Code::try_from(0x00).unwrap(), Code::Identity);
		assert_eq!(Code::try_from(0x12).unwrap(), Code::Sha2_256);
		assert!(matches!(Code::try_from(0x01).unwrap_err(), Error::UnsupportedCode(0x01)));
	}

	#[test]
	fn code_into_u64() {
		assert_eq!(u64::from(Code::Identity), 0x00);
		assert_eq!(u64::from(Code::Sha2_256), 0x12);
	}
}
