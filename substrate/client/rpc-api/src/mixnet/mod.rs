// This file is part of a fork of Substrate which has had various changes.

// Copyright (C) Parity Technologies (UK) Ltd.
// Copyright (C) 2022-2023 Luke Parker
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

//! Substrate mixnet API.

pub mod error;

use error::Error;
use jsonrpsee::proc_macros::rpc;
use sp_core::Bytes;

#[rpc(client, server)]
pub trait MixnetApi {
	/// Submit encoded extrinsic over the mixnet for inclusion in block.
	#[method(name = "mixnet_submitExtrinsic")]
	async fn submit_extrinsic(&self, extrinsic: Bytes) -> Result<(), Error>;
}
