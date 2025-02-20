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

use crate::{
	discovery::{DiscoveryBehaviour, DiscoveryConfig, DiscoveryOut},
	event::DhtEvent,
	peer_info,
	peer_store::PeerStoreHandle,
	protocol::{CustomMessageOutcome, NotificationsSink, Protocol},
	protocol_controller::SetId,
	request_responses::{self, IfDisconnected, ProtocolConfig, RequestFailure},
	service::traits::Direction,
	types::ProtocolName,
	ReputationChange,
};

use futures::channel::oneshot;
use libp2p::{
	core::Multiaddr, identify::Info as IdentifyInfo, identity::PublicKey, kad::RecordKey,
	swarm::NetworkBehaviour, PeerId,
};

use parking_lot::Mutex;
use sp_runtime::traits::Block as BlockT;
use std::{collections::HashSet, sync::Arc, time::Duration};

pub use crate::request_responses::{InboundFailure, OutboundFailure, ResponseFailure};

/// General behaviour of the network. Combines all protocols together.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BehaviourOut")]
pub struct Behaviour<B: BlockT> {
	/// All the substrate-specific protocols.
	substrate: Protocol<B>,
	/// Periodically pings and identifies the nodes we are connected to, and store information in a
	/// cache.
	peer_info: peer_info::PeerInfoBehaviour,
	/// Discovers nodes of the network.
	discovery: DiscoveryBehaviour,
	/// Generic request-response protocols.
	request_responses: request_responses::RequestResponsesBehaviour,
}

/// Event generated by `Behaviour`.
pub enum BehaviourOut {
	/// Started a random iterative Kademlia discovery query.
	RandomKademliaStarted,

	/// We have received a request from a peer and answered it.
	///
	/// This event is generated for statistics purposes.
	InboundRequest {
		/// Peer which sent us a request.
		peer: PeerId,
		/// Protocol name of the request.
		protocol: ProtocolName,
		/// If `Ok`, contains the time elapsed between when we received the request and when we
		/// sent back the response. If `Err`, the error that happened.
		result: Result<Duration, ResponseFailure>,
	},

	/// A request has succeeded or failed.
	///
	/// This event is generated for statistics purposes.
	RequestFinished {
		/// Peer that we send a request to.
		peer: PeerId,
		/// Name of the protocol in question.
		protocol: ProtocolName,
		/// Duration the request took.
		duration: Duration,
		/// Result of the request.
		result: Result<(), RequestFailure>,
	},

	/// A request protocol handler issued reputation changes for the given peer.
	ReputationChanges { peer: PeerId, changes: Vec<ReputationChange> },

	/// Opened a substream with the given node with the given notifications protocol.
	///
	/// The protocol is always one of the notification protocols that have been registered.
	NotificationStreamOpened {
		/// Node we opened the substream with.
		remote: PeerId,
		/// Set ID.
		set_id: SetId,
		/// Direction of the stream.
		direction: Direction,
		/// If the negotiation didn't use the main name of the protocol (the one in
		/// `notifications_protocol`), then this field contains which name has actually been
		/// used.
		/// See also [`crate::Event::NotificationStreamOpened`].
		negotiated_fallback: Option<ProtocolName>,
		/// Object that permits sending notifications to the peer.
		notifications_sink: NotificationsSink,
		/// Received handshake.
		received_handshake: Vec<u8>,
	},

	/// The [`NotificationsSink`] object used to send notifications with the given peer must be
	/// replaced with a new one.
	///
	/// This event is typically emitted when a transport-level connection is closed and we fall
	/// back to a secondary connection.
	NotificationStreamReplaced {
		/// Id of the peer we are connected to.
		remote: PeerId,
		/// Set ID.
		set_id: SetId,
		/// Replacement for the previous [`NotificationsSink`].
		notifications_sink: NotificationsSink,
	},

	/// Closed a substream with the given node. Always matches a corresponding previous
	/// `NotificationStreamOpened` message.
	NotificationStreamClosed {
		/// Node we closed the substream with.
		remote: PeerId,
		/// Set ID.
		set_id: SetId,
	},

	/// Received one or more messages from the given node using the given protocol.
	NotificationsReceived {
		/// Node we received the message from.
		remote: PeerId,
		/// Set ID.
		set_id: SetId,
		/// Concerned protocol and associated message.
		notification: Vec<u8>,
	},

	/// We have obtained identity information from a peer, including the addresses it is listening
	/// on.
	PeerIdentify {
		/// Id of the peer that has been identified.
		peer_id: PeerId,
		/// Information about the peer.
		info: IdentifyInfo,
	},

	/// We have learned about the existence of a node on the default set.
	Discovered(PeerId),

	/// Events generated by a DHT as a response to get_value or put_value requests as well as the
	/// request duration.
	Dht(DhtEvent, Duration),

	/// Ignored event generated by lower layers.
	None,
}

impl<B: BlockT> Behaviour<B> {
	/// Builds a new `Behaviour`.
	pub fn new(
		substrate: Protocol<B>,
		user_agent: String,
		local_public_key: PublicKey,
		disco_config: DiscoveryConfig,
		request_response_protocols: Vec<ProtocolConfig>,
		peer_store_handle: PeerStoreHandle,
		external_addresses: Arc<Mutex<HashSet<Multiaddr>>>,
	) -> Result<Self, request_responses::RegisterError> {
		Ok(Self {
			substrate,
			peer_info: peer_info::PeerInfoBehaviour::new(
				user_agent,
				local_public_key,
				external_addresses,
			),
			discovery: disco_config.finish(),
			request_responses: request_responses::RequestResponsesBehaviour::new(
				request_response_protocols.into_iter(),
				Box::new(peer_store_handle),
			)?,
		})
	}

	/// Returns the list of nodes that we know exist in the network.
	pub fn known_peers(&mut self) -> HashSet<PeerId> {
		self.discovery.known_peers()
	}

	/// Adds a hard-coded address for the given peer, that never expires.
	pub fn add_known_address(&mut self, peer_id: PeerId, addr: Multiaddr) {
		self.discovery.add_known_address(peer_id, addr)
	}

	/// Returns the number of nodes in each Kademlia kbucket.
	///
	/// Identifies kbuckets by the base 2 logarithm of their lower bound.
	pub fn num_entries_per_kbucket(&mut self) -> Option<Vec<(u32, usize)>> {
		self.discovery.num_entries_per_kbucket()
	}

	/// Returns the number of records in the Kademlia record stores.
	pub fn num_kademlia_records(&mut self) -> Option<usize> {
		self.discovery.num_kademlia_records()
	}

	/// Returns the total size in bytes of all the records in the Kademlia record stores.
	pub fn kademlia_records_total_size(&mut self) -> Option<usize> {
		self.discovery.kademlia_records_total_size()
	}

	/// Borrows `self` and returns a struct giving access to the information about a node.
	///
	/// Returns `None` if we don't know anything about this node. Always returns `Some` for nodes
	/// we're connected to, meaning that if `None` is returned then we're not connected to that
	/// node.
	pub fn node(&self, peer_id: &PeerId) -> Option<peer_info::Node> {
		self.peer_info.node(peer_id)
	}

	/// Initiates sending a request.
	pub fn send_request(
		&mut self,
		target: &PeerId,
		protocol: ProtocolName,
		request: Vec<u8>,
		fallback_request: Option<(Vec<u8>, ProtocolName)>,
		pending_response: oneshot::Sender<Result<(Vec<u8>, ProtocolName), RequestFailure>>,
		connect: IfDisconnected,
	) {
		self.request_responses.send_request(
			target,
			protocol,
			request,
			fallback_request,
			pending_response,
			connect,
		)
	}

	/// Returns a shared reference to the user protocol.
	pub fn user_protocol(&self) -> &Protocol<B> {
		&self.substrate
	}

	/// Returns a mutable reference to the user protocol.
	pub fn user_protocol_mut(&mut self) -> &mut Protocol<B> {
		&mut self.substrate
	}

	/// Add a self-reported address of a remote peer to the k-buckets of the supported
	/// DHTs (`supported_protocols`).
	pub fn add_self_reported_address_to_dht(
		&mut self,
		peer_id: &PeerId,
		supported_protocols: &[impl AsRef<[u8]>],
		addr: Multiaddr,
	) {
		self.discovery.add_self_reported_address(peer_id, supported_protocols, addr);
	}

	/// Start querying a record from the DHT. Will later produce either a `ValueFound` or a
	/// `ValueNotFound` event.
	pub fn get_value(&mut self, key: RecordKey) {
		self.discovery.get_value(key);
	}

	/// Starts putting a record into DHT. Will later produce either a `ValuePut` or a
	/// `ValuePutFailed` event.
	pub fn put_value(&mut self, key: RecordKey, value: Vec<u8>) {
		self.discovery.put_value(key, value);
	}
}

impl From<CustomMessageOutcome> for BehaviourOut {
	fn from(event: CustomMessageOutcome) -> Self {
		match event {
			CustomMessageOutcome::NotificationStreamOpened {
				remote,
				set_id,
				direction,
				negotiated_fallback,
				received_handshake,
				notifications_sink,
			} => BehaviourOut::NotificationStreamOpened {
				remote,
				set_id,
				direction,
				negotiated_fallback,
				received_handshake,
				notifications_sink,
			},
			CustomMessageOutcome::NotificationStreamReplaced {
				remote,
				set_id,
				notifications_sink,
			} => BehaviourOut::NotificationStreamReplaced { remote, set_id, notifications_sink },
			CustomMessageOutcome::NotificationStreamClosed { remote, set_id } =>
				BehaviourOut::NotificationStreamClosed { remote, set_id },
			CustomMessageOutcome::NotificationsReceived { remote, set_id, notification } =>
				BehaviourOut::NotificationsReceived { remote, set_id, notification },
		}
	}
}

impl From<request_responses::Event> for BehaviourOut {
	fn from(event: request_responses::Event) -> Self {
		match event {
			request_responses::Event::InboundRequest { peer, protocol, result } =>
				BehaviourOut::InboundRequest { peer, protocol, result },
			request_responses::Event::RequestFinished { peer, protocol, duration, result } =>
				BehaviourOut::RequestFinished { peer, protocol, duration, result },
			request_responses::Event::ReputationChanges { peer, changes } =>
				BehaviourOut::ReputationChanges { peer, changes },
		}
	}
}

impl From<peer_info::PeerInfoEvent> for BehaviourOut {
	fn from(event: peer_info::PeerInfoEvent) -> Self {
		let peer_info::PeerInfoEvent::Identified { peer_id, info } = event;
		BehaviourOut::PeerIdentify { peer_id, info }
	}
}

impl From<DiscoveryOut> for BehaviourOut {
	fn from(event: DiscoveryOut) -> Self {
		match event {
			DiscoveryOut::UnroutablePeer(_peer_id) => {
				// Obtaining and reporting listen addresses for unroutable peers back
				// to Kademlia is handled by the `Identify` protocol, part of the
				// `PeerInfoBehaviour`. See the `From<peer_info::PeerInfoEvent>`
				// implementation.
				BehaviourOut::None
			},
			DiscoveryOut::Discovered(peer_id) => BehaviourOut::Discovered(peer_id),
			DiscoveryOut::ValueFound(results, duration) =>
				BehaviourOut::Dht(DhtEvent::ValueFound(results), duration),
			DiscoveryOut::ValueNotFound(key, duration) =>
				BehaviourOut::Dht(DhtEvent::ValueNotFound(key), duration),
			DiscoveryOut::ValuePut(key, duration) =>
				BehaviourOut::Dht(DhtEvent::ValuePut(key), duration),
			DiscoveryOut::ValuePutFailed(key, duration) =>
				BehaviourOut::Dht(DhtEvent::ValuePutFailed(key), duration),
			DiscoveryOut::RandomKademliaStarted => BehaviourOut::RandomKademliaStarted,
		}
	}
}
