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

//! Mocked components for tests.

use crate::{peer_store::PeerStoreProvider, protocol_controller::ProtocolHandle, ReputationChange};
use libp2p::PeerId;
use sc_network_common::role::ObservedRole;
use std::collections::HashSet;

/// No-op `PeerStore`.
#[derive(Debug)]
pub struct MockPeerStore {}

impl PeerStoreProvider for MockPeerStore {
	fn is_banned(&self, _peer_id: &PeerId) -> bool {
		// Make sure that the peer is not banned.
		false
	}

	fn register_protocol(&self, _protocol_handle: ProtocolHandle) {
		// Make sure not to fail.
	}

	fn report_disconnect(&mut self, _peer_id: PeerId) {
		// Make sure not to fail.
	}

	fn report_peer(&mut self, _peer_id: PeerId, _change: ReputationChange) {
		// Make sure not to fail.
	}

	fn peer_reputation(&self, _peer_id: &PeerId) -> i32 {
		// Make sure that the peer is not banned.
		0
	}

	fn peer_role(&self, _peer_id: &PeerId) -> Option<ObservedRole> {
		None
	}

	fn set_peer_role(&mut self, _peer_id: &PeerId, _role: ObservedRole) {
		unimplemented!();
	}

	fn outgoing_candidates(&self, _count: usize, _ignored: HashSet<&PeerId>) -> Vec<PeerId> {
		unimplemented!()
	}
}
