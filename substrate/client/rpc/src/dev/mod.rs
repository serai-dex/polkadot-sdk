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

//! Implementation of the [`DevApiServer`] trait providing debug utilities for Substrate based
//! blockchains.

#[cfg(test)]
mod tests;

use jsonrpsee::Extensions;
use sc_client_api::{BlockBackend, HeaderBackend};
use sc_rpc_api::{check_if_safe, dev::error::Error};
use sp_api::{ApiExt, Core, ProvideRuntimeApi};
use sp_core::Encode;
use sp_runtime::{
	generic::DigestItem,
	traits::{Block as BlockT, Header},
};
use std::{
	marker::{PhantomData, Send, Sync},
	sync::Arc,
};

pub use sc_rpc_api::dev::{BlockStats, DevApiServer};

type HasherOf<Block> = <<Block as BlockT>::Header as Header>::Hashing;

/// The Dev API. All methods are unsafe.
pub struct Dev<Block: BlockT, Client> {
	client: Arc<Client>,
	_phantom: PhantomData<Block>,
}

impl<Block: BlockT, Client> Dev<Block, Client> {
	/// Create a new Dev API.
	pub fn new(client: Arc<Client>) -> Self {
		Self { client, _phantom: PhantomData::default() }
	}
}

impl<Block, Client> DevApiServer<Block::Hash> for Dev<Block, Client>
where
	Block: BlockT + 'static,
	Client: BlockBackend<Block>
		+ HeaderBackend<Block>
		+ ProvideRuntimeApi<Block>
		+ Send
		+ Sync
		+ 'static,
	Client::Api: Core<Block>,
{
	fn block_stats(
		&self,
		ext: &Extensions,
		hash: Block::Hash,
	) -> Result<Option<BlockStats>, Error> {
		check_if_safe(ext)?;

		let block = {
			let block = self.client.block(hash).map_err(|e| Error::BlockQueryError(Box::new(e)))?;
			if let Some(block) = block {
				let (mut header, body) = block.block.deconstruct();
				// Remove the `Seal` to ensure we have the number of digests as expected by the
				// runtime.
				header.digest_mut().logs.retain(|item| !matches!(item, DigestItem::Seal(_, _)));
				Block::new(header, body)
			} else {
				return Ok(None)
			}
		};
		let block_len = block.encoded_size() as u64;
		let num_extrinsics = block.extrinsics().len() as u64;
		Ok(Some(BlockStats { block_len, num_extrinsics }))
	}
}
