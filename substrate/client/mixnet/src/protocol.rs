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

use super::config::Config;
use mixnet::core::PACKET_SIZE;
use sc_network::{
	config::{NonDefaultSetConfig, NonReservedPeerMode, SetConfig},
	NotificationService, ProtocolName,
};

/// Returns the protocol name to use for the mixnet controlled by the given chain.
pub fn protocol_name(genesis_hash: &[u8], fork_id: Option<&str>) -> ProtocolName {
	let name = if let Some(fork_id) = fork_id {
		format!("/{}/{}/mixnet/1", array_bytes::bytes2hex("", genesis_hash), fork_id)
	} else {
		format!("/{}/mixnet/1", array_bytes::bytes2hex("", genesis_hash))
	};
	name.into()
}

/// Returns the peers set configuration for the mixnet protocol.
pub fn peers_set_config(
	name: ProtocolName,
	config: &Config,
) -> (NonDefaultSetConfig, Box<dyn NotificationService>) {
	let (mut set_config, service) = NonDefaultSetConfig::new(
		name,
		Vec::new(),
		PACKET_SIZE as u64,
		None,
		SetConfig {
			in_peers: 0,
			out_peers: 0,
			reserved_nodes: Vec::new(),
			non_reserved_mode: NonReservedPeerMode::Deny,
		},
	);
	if config.substrate.num_gateway_slots != 0 {
		// out_peers is always 0; we are only interested in connecting to mixnodes, which we do by
		// setting them as reserved nodes
		set_config.allow_non_reserved(config.substrate.num_gateway_slots, 0);
	}
	(set_config, service)
}
