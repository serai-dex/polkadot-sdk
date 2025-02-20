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

//! Offchain worker related configuration parameters.
//!
//! A subset of configuration parameters which are relevant to
//! the inner working of offchain workers. The usage is solely
//! targeted at handling input parameter parsing providing
//! a reasonable abstraction.

use clap::{ArgAction, Args};
use sc_network::config::Role;
use sc_service::config::OffchainWorkerConfig;

use crate::{error, OffchainWorkerEnabled};

/// Offchain worker related parameters.
#[derive(Debug, Clone, Args)]
pub struct OffchainWorkerParams {
	/// Execute offchain workers on every block.
	#[arg(
		long = "offchain-worker",
		value_name = "ENABLED",
		value_enum,
		ignore_case = true,
		default_value_t = OffchainWorkerEnabled::WhenAuthority
	)]
	pub enabled: OffchainWorkerEnabled,

	/// Enable offchain indexing API.
	///
	/// Allows the runtime to write directly to offchain workers DB during block import.
	#[arg(long = "enable-offchain-indexing", value_name = "ENABLE_OFFCHAIN_INDEXING", default_value_t = false, action = ArgAction::Set)]
	pub indexing_enabled: bool,
}

impl OffchainWorkerParams {
	/// Load spec to `Configuration` from `OffchainWorkerParams` and spec factory.
	pub fn offchain_worker(&self, role: &Role) -> error::Result<OffchainWorkerConfig> {
		let enabled = match (&self.enabled, role) {
			(OffchainWorkerEnabled::WhenAuthority, Role::Authority { .. }) => true,
			(OffchainWorkerEnabled::Always, _) => true,
			(OffchainWorkerEnabled::Never, _) => false,
			(OffchainWorkerEnabled::WhenAuthority, _) => false,
		};

		let indexing_enabled = self.indexing_enabled;
		Ok(OffchainWorkerConfig { enabled, indexing_enabled })
	}
}
