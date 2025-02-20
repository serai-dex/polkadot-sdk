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

//! Main entry point of the sc-network crate.
//!
//! There are two main structs in this module: [`NetworkWorker`] and [`NetworkService`].
//! The [`NetworkWorker`] *is* the network. Network is driven by [`NetworkWorker::run`] future that
//! terminates only when all instances of the control handles [`NetworkService`] were dropped.
//! The [`NetworkService`] is merely a shared version of the [`NetworkWorker`]. You can obtain an
//! `Arc<NetworkService>` by calling [`NetworkWorker::service`].
//!
//! The methods of the [`NetworkService`] are implemented by sending a message over a channel,
//! which is then processed by [`NetworkWorker::next_action`].

use crate::{
	behaviour::{self, Behaviour, BehaviourOut},
	config::{parse_addr, FullNetworkConfiguration, MultiaddrWithPeerId, Params, TransportConfig},
	discovery::DiscoveryConfig,
	error::Error,
	event::{DhtEvent, Event},
	network_state::{
		NetworkState, NotConnectedPeer as NetworkStateNotConnectedPeer, Peer as NetworkStatePeer,
	},
	peer_store::{PeerStoreHandle, PeerStoreProvider},
	protocol::{self, NotifsHandlerError, Protocol, Ready},
	protocol_controller::{self, ProtoSetConfig, ProtocolController, SetId},
	request_responses::{IfDisconnected, RequestFailure},
	service::{
		signature::{Signature, SigningError},
		traits::{
			NetworkDHTProvider, NetworkEventStream, NetworkNotification, NetworkPeers,
			NetworkRequest, NetworkSigner, NetworkStateInfo, NetworkStatus, NetworkStatusProvider,
			NotificationSender as NotificationSenderT, NotificationSenderError,
			NotificationSenderReady as NotificationSenderReadyT,
		},
	},
	transport,
	types::ProtocolName,
	ReputationChange,
};

use codec::DecodeAll;
use either::Either;
use futures::{channel::oneshot, prelude::*};
#[allow(deprecated)]
use libp2p::{
	connection_limits::Exceeded,
	core::{upgrade, ConnectedPoint, Endpoint},
	identify::Info as IdentifyInfo,
	kad::record::Key as KademliaKey,
	multiaddr,
	ping::Failure as PingFailure,
	swarm::{
		AddressScore, ConnectionError, ConnectionId, ConnectionLimits, DialError, Executor,
		ListenError, NetworkBehaviour, Swarm, SwarmBuilder, SwarmEvent, THandlerErr,
	},
	Multiaddr, PeerId,
};
use log::{debug, error, info, trace, warn};
use metrics::{Histogram, MetricSources, Metrics};
use parking_lot::Mutex;

use sc_network_common::{
	role::{ObservedRole, Roles},
	ExHashT,
};
use sc_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};
use sp_runtime::traits::Block as BlockT;

use std::{
	cmp,
	collections::{HashMap, HashSet},
	fs, iter,
	marker::PhantomData,
	num::NonZeroUsize,
	pin::Pin,
	str,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc,
	},
};

pub use behaviour::{InboundFailure, OutboundFailure, ResponseFailure};
pub use libp2p::identity::{DecodingError, Keypair, PublicKey};
pub use protocol::NotificationsSink;

mod metrics;
mod out_events;

pub mod signature;
pub mod traits;

/// Substrate network service. Handles network IO and manages connectivity.
pub struct NetworkService<B: BlockT + 'static, H: ExHashT> {
	/// Number of peers we're connected to.
	num_connected: Arc<AtomicUsize>,
	/// The local external addresses.
	external_addresses: Arc<Mutex<HashSet<Multiaddr>>>,
	/// Listen addresses. Do **NOT** include a trailing `/p2p/` with our `PeerId`.
	listen_addresses: Arc<Mutex<HashSet<Multiaddr>>>,
	/// Local copy of the `PeerId` of the local node.
	local_peer_id: PeerId,
	/// The `KeyPair` that defines the `PeerId` of the local node.
	local_identity: Keypair,
	/// Bandwidth logging system. Can be queried to know the average bandwidth consumed.
	bandwidth: Arc<transport::BandwidthSinks>,
	/// Used to query and report reputation changes.
	peer_store_handle: PeerStoreHandle,
	/// Channel that sends messages to the actual worker.
	to_worker: TracingUnboundedSender<ServiceToWorkerMsg>,
	/// Protocol name -> `SetId` mapping for notification protocols. The map never changes after
	/// initialization.
	notification_protocol_ids: HashMap<ProtocolName, SetId>,
	/// Handles to manage peer connections on notification protocols. The vector never changes
	/// after initialization.
	protocol_handles: Vec<protocol_controller::ProtocolHandle>,
	/// Shortcut to sync protocol handle (`protocol_handles[0]`).
	sync_protocol_handle: protocol_controller::ProtocolHandle,
	/// Marker to pin the `H` generic. Serves no purpose except to not break backwards
	/// compatibility.
	_marker: PhantomData<H>,
	/// Marker for block type
	_block: PhantomData<B>,
}

impl<B, H> NetworkWorker<B, H>
where
	B: BlockT + 'static,
	H: ExHashT,
{
	/// Creates the network service.
	///
	/// Returns a `NetworkWorker` that implements `Future` and must be regularly polled in order
	/// for the network processing to advance. From it, you can extract a `NetworkService` using
	/// `worker.service()`. The `NetworkService` can be shared through the codebase.
	pub fn new(params: Params<B>) -> Result<Self, Error> {
		let FullNetworkConfiguration {
			notification_protocols,
			request_response_protocols,
			mut network_config,
		} = params.network_config;

		// Private and public keys configuration.
		let local_identity = network_config.node_key.clone().into_keypair()?;
		let local_public = local_identity.public();
		let local_peer_id = local_public.to_peer_id();

		network_config.boot_nodes = network_config
			.boot_nodes
			.into_iter()
			.filter(|boot_node| boot_node.peer_id != local_peer_id)
			.collect();
		network_config.default_peers_set.reserved_nodes = network_config
			.default_peers_set
			.reserved_nodes
			.into_iter()
			.filter(|reserved_node| {
				if reserved_node.peer_id == local_peer_id {
					warn!(
						target: "sub-libp2p",
						"Local peer ID used in reserved node, ignoring: {}",
						reserved_node,
					);
					false
				} else {
					true
				}
			})
			.collect();

		// Ensure the listen addresses are consistent with the transport.
		ensure_addresses_consistent_with_transport(
			network_config.listen_addresses.iter(),
			&network_config.transport,
		)?;
		ensure_addresses_consistent_with_transport(
			network_config.boot_nodes.iter().map(|x| &x.multiaddr),
			&network_config.transport,
		)?;
		ensure_addresses_consistent_with_transport(
			network_config.default_peers_set.reserved_nodes.iter().map(|x| &x.multiaddr),
			&network_config.transport,
		)?;
		for notification_protocol in &notification_protocols {
			ensure_addresses_consistent_with_transport(
				notification_protocol.set_config().reserved_nodes.iter().map(|x| &x.multiaddr),
				&network_config.transport,
			)?;
		}
		ensure_addresses_consistent_with_transport(
			network_config.public_addresses.iter(),
			&network_config.transport,
		)?;

		let (to_worker, from_service) = tracing_unbounded("mpsc_network_worker", 100_000);

		if let Some(path) = &network_config.net_config_path {
			fs::create_dir_all(path)?;
		}

		info!(
			target: "sub-libp2p",
			"Local node identity is: {}",
			local_peer_id.to_base58(),
		);

		let (transport, bandwidth) = {
			let config_mem = match network_config.transport {
				TransportConfig::MemoryOnly => true,
				TransportConfig::Normal { .. } => false,
			};

			// The yamux buffer size limit is configured to be equal to the maximum frame size
			// of all protocols. 10 bytes are added to each limit for the length prefix that
			// is not included in the upper layer protocols limit but is still present in the
			// yamux buffer. These 10 bytes correspond to the maximum size required to encode
			// a variable-length-encoding 64bits number. In other words, we make the
			// assumption that no notification larger than 2^64 will ever be sent.
			let yamux_maximum_buffer_size = {
				let requests_max = request_response_protocols
					.iter()
					.map(|cfg| usize::try_from(cfg.max_request_size).unwrap_or(usize::MAX));
				let responses_max = request_response_protocols
					.iter()
					.map(|cfg| usize::try_from(cfg.max_response_size).unwrap_or(usize::MAX));
				let notifs_max = notification_protocols
					.iter()
					.map(|cfg| usize::try_from(cfg.max_notification_size()).unwrap_or(usize::MAX));

				// A "default" max is added to cover all the other protocols: ping, identify,
				// kademlia, block announces, and transactions.
				let default_max = cmp::max(
					1024 * 1024,
					usize::try_from(protocol::BLOCK_ANNOUNCES_TRANSACTIONS_SUBSTREAM_SIZE)
						.unwrap_or(usize::MAX),
				);

				iter::once(default_max)
					.chain(requests_max)
					.chain(responses_max)
					.chain(notifs_max)
					.max()
					.expect("iterator known to always yield at least one element")
					.saturating_add(10)
			};

			transport::build_transport(
				local_identity.clone(),
				config_mem,
				network_config.yamux_window_size,
				yamux_maximum_buffer_size,
			)
		};

		let (to_notifications, from_protocol_controllers) =
			tracing_unbounded("mpsc_protocol_controllers_to_notifications", 10_000);

		// We must prepend a hardcoded default peer set to notification protocols.
		let all_peer_sets_iter = iter::once(&network_config.default_peers_set)
			.chain(notification_protocols.iter().map(|protocol| protocol.set_config()));

		let (protocol_handles, protocol_controllers): (Vec<_>, Vec<_>) = all_peer_sets_iter
			.enumerate()
			.map(|(set_id, set_config)| {
				let proto_set_config = ProtoSetConfig {
					in_peers: set_config.in_peers,
					out_peers: set_config.out_peers,
					reserved_nodes: set_config
						.reserved_nodes
						.iter()
						.map(|node| node.peer_id)
						.collect(),
					reserved_only: set_config.non_reserved_mode.is_reserved_only(),
				};

				ProtocolController::new(
					SetId::from(set_id),
					proto_set_config,
					to_notifications.clone(),
					Box::new(params.peer_store.clone()),
				)
			})
			.unzip();

		// Shortcut to default (sync) peer set protocol handle.
		let sync_protocol_handle = protocol_handles[0].clone();

		// Spawn `ProtocolController` runners.
		protocol_controllers
			.into_iter()
			.for_each(|controller| (params.executor)(controller.run().boxed()));

		// Protocol name to protocol id mapping. The first protocol is always block announce (sync)
		// protocol, aka default (hardcoded) peer set.
		let notification_protocol_ids: HashMap<ProtocolName, SetId> =
			iter::once(&params.block_announce_config)
				.chain(notification_protocols.iter())
				.enumerate()
				.map(|(index, protocol)| (protocol.protocol_name().clone(), SetId::from(index)))
				.collect();

		let known_addresses = {
			// Collect all reserved nodes and bootnodes addresses.
			let mut addresses: Vec<_> = network_config
				.default_peers_set
				.reserved_nodes
				.iter()
				.map(|reserved| (reserved.peer_id, reserved.multiaddr.clone()))
				.chain(notification_protocols.iter().flat_map(|protocol| {
					protocol
						.set_config()
						.reserved_nodes
						.iter()
						.map(|reserved| (reserved.peer_id, reserved.multiaddr.clone()))
				}))
				.chain(
					network_config
						.boot_nodes
						.iter()
						.map(|bootnode| (bootnode.peer_id, bootnode.multiaddr.clone())),
				)
				.collect();

			// Remove possible duplicates.
			addresses.sort();
			addresses.dedup();

			addresses
		};

		// Check for duplicate bootnodes.
		network_config.boot_nodes.iter().try_for_each(|bootnode| {
			if let Some(other) = network_config
				.boot_nodes
				.iter()
				.filter(|o| o.multiaddr == bootnode.multiaddr)
				.find(|o| o.peer_id != bootnode.peer_id)
			{
				Err(Error::DuplicateBootnode {
					address: bootnode.multiaddr.clone(),
					first_id: bootnode.peer_id,
					second_id: other.peer_id,
				})
			} else {
				Ok(())
			}
		})?;

		// List of bootnode multiaddresses.
		let mut boot_node_ids = HashMap::<PeerId, Vec<Multiaddr>>::new();

		for bootnode in network_config.boot_nodes.iter() {
			boot_node_ids
				.entry(bootnode.peer_id)
				.or_default()
				.push(bootnode.multiaddr.clone());
		}

		let boot_node_ids = Arc::new(boot_node_ids);

		let num_connected = Arc::new(AtomicUsize::new(0));
		let external_addresses = Arc::new(Mutex::new(HashSet::new()));

		let (protocol, notif_protocol_handles) = Protocol::new(
			From::from(&params.role),
			&params.metrics_registry,
			notification_protocols,
			params.block_announce_config,
			params.peer_store.clone(),
			protocol_handles.clone(),
			from_protocol_controllers,
		)?;

		// Build the swarm.
		let (mut swarm, bandwidth): (Swarm<Behaviour<B>>, _) = {
			let user_agent =
				format!("{} ({})", network_config.client_version, network_config.node_name);

			let discovery_config = {
				let mut config = DiscoveryConfig::new(local_public.to_peer_id());
				config.with_permanent_addresses(known_addresses);
				config.discovery_limit(u64::from(network_config.default_peers_set.out_peers) + 15);
				config.with_kademlia(
					params.genesis_hash,
					params.fork_id.as_deref(),
					&params.protocol_id,
				);
				config.with_dht_random_walk(network_config.enable_dht_random_walk);
				config.allow_non_globals_in_dht(network_config.allow_non_globals_in_dht);
				config.use_kademlia_disjoint_query_paths(
					network_config.kademlia_disjoint_query_paths,
				);
				config.with_kademlia_replication_factor(network_config.kademlia_replication_factor);

				match network_config.transport {
					TransportConfig::MemoryOnly => {
						config.with_mdns(false);
						config.allow_private_ip(false);
					},
					TransportConfig::Normal {
						enable_mdns,
						allow_private_ip: allow_private_ipv4,
						..
					} => {
						config.with_mdns(enable_mdns);
						config.allow_private_ip(allow_private_ipv4);
					},
				}

				config
			};

			let behaviour = {
				let result = Behaviour::new(
					protocol,
					user_agent,
					local_public,
					discovery_config,
					request_response_protocols,
					params.peer_store.clone(),
					external_addresses.clone(),
				);

				match result {
					Ok(b) => b,
					Err(crate::request_responses::RegisterError::DuplicateProtocol(proto)) =>
						return Err(Error::DuplicateRequestResponseProtocol { protocol: proto }),
				}
			};

			let builder = {
				struct SpawnImpl<F>(F);
				impl<F: Fn(Pin<Box<dyn Future<Output = ()> + Send>>)> Executor for SpawnImpl<F> {
					fn exec(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) {
						(self.0)(f)
					}
				}
				SwarmBuilder::with_executor(
					transport,
					behaviour,
					local_peer_id,
					SpawnImpl(params.executor),
				)
			};
			#[allow(deprecated)]
			let builder = builder
				.connection_limits(
					ConnectionLimits::default()
						.with_max_established_per_peer(Some(crate::MAX_CONNECTIONS_PER_PEER as u32))
						.with_max_established_incoming(Some(
							crate::MAX_CONNECTIONS_ESTABLISHED_INCOMING,
						)),
				)
				.substream_upgrade_protocol_override(upgrade::Version::V1Lazy)
				.notify_handler_buffer_size(NonZeroUsize::new(32).expect("32 != 0"))
				// NOTE: 24 is somewhat arbitrary and should be tuned in the future if necessary.
				// See <https://github.com/paritytech/substrate/pull/6080>
				.per_connection_event_buffer_size(24)
				.max_negotiating_inbound_streams(2048);

			(builder.build(), bandwidth)
		};

		// Initialize the metrics.
		let metrics = match &params.metrics_registry {
			Some(registry) => Some(metrics::register(
				registry,
				MetricSources {
					bandwidth: bandwidth.clone(),
					connected_peers: num_connected.clone(),
				},
			)?),
			None => None,
		};

		// Listen on multiaddresses.
		for addr in &network_config.listen_addresses {
			if let Err(err) = Swarm::<Behaviour<B>>::listen_on(&mut swarm, addr.clone()) {
				warn!(target: "sub-libp2p", "Can't listen on {} because: {:?}", addr, err)
			}
		}

		// Add external addresses.
		for addr in &network_config.public_addresses {
			Swarm::<Behaviour<B>>::add_external_address(
				&mut swarm,
				addr.clone(),
				AddressScore::Infinite,
			);
		}

		let listen_addresses = Arc::new(Mutex::new(HashSet::new()));

		let service = Arc::new(NetworkService {
			bandwidth,
			external_addresses,
			listen_addresses: listen_addresses.clone(),
			num_connected: num_connected.clone(),
			local_peer_id,
			local_identity,
			to_worker,
			notification_protocol_ids,
			protocol_handles,
			sync_protocol_handle,
			peer_store_handle: params.peer_store.clone(),
			_marker: PhantomData,
			_block: Default::default(),
		});

		Ok(NetworkWorker {
			listen_addresses,
			num_connected,
			network_service: swarm,
			service,
			from_service,
			event_streams: out_events::OutChannels::new(params.metrics_registry.as_ref())?,
			metrics,
			boot_node_ids,
			reported_invalid_boot_nodes: Default::default(),
			peer_store_handle: params.peer_store,
			notif_protocol_handles,
			_marker: Default::default(),
			_block: Default::default(),
		})
	}

	/// High-level network status information.
	pub fn status(&self) -> NetworkStatus {
		NetworkStatus {
			num_connected_peers: self.num_connected_peers(),
			total_bytes_inbound: self.total_bytes_inbound(),
			total_bytes_outbound: self.total_bytes_outbound(),
		}
	}

	/// Returns the total number of bytes received so far.
	pub fn total_bytes_inbound(&self) -> u64 {
		self.service.bandwidth.total_inbound()
	}

	/// Returns the total number of bytes sent so far.
	pub fn total_bytes_outbound(&self) -> u64 {
		self.service.bandwidth.total_outbound()
	}

	/// Returns the number of peers we're connected to.
	pub fn num_connected_peers(&self) -> usize {
		self.network_service.behaviour().user_protocol().num_sync_peers()
	}

	/// Adds an address for a node.
	pub fn add_known_address(&mut self, peer_id: PeerId, addr: Multiaddr) {
		self.network_service.behaviour_mut().add_known_address(peer_id, addr);
	}

	/// Return a `NetworkService` that can be shared through the code base and can be used to
	/// manipulate the worker.
	pub fn service(&self) -> &Arc<NetworkService<B, H>> {
		&self.service
	}

	/// Returns the local `PeerId`.
	pub fn local_peer_id(&self) -> &PeerId {
		Swarm::<Behaviour<B>>::local_peer_id(&self.network_service)
	}

	/// Returns the list of addresses we are listening on.
	///
	/// Does **NOT** include a trailing `/p2p/` with our `PeerId`.
	pub fn listen_addresses(&self) -> impl Iterator<Item = &Multiaddr> {
		Swarm::<Behaviour<B>>::listeners(&self.network_service)
	}

	/// Get network state.
	///
	/// **Note**: Use this only for debugging. This API is unstable. There are warnings literally
	/// everywhere about this. Please don't use this function to retrieve actual information.
	pub fn network_state(&mut self) -> NetworkState {
		let swarm = &mut self.network_service;
		let open = swarm.behaviour_mut().user_protocol().open_peers().cloned().collect::<Vec<_>>();
		let connected_peers = {
			let swarm = &mut *swarm;
			open.iter()
				.filter_map(move |peer_id| {
					let known_addresses = if let Ok(addrs) =
						NetworkBehaviour::handle_pending_outbound_connection(
							swarm.behaviour_mut(),
							ConnectionId::new_unchecked(0), // dummy value
							Some(*peer_id),
							&vec![],
							Endpoint::Listener,
						) {
						addrs.into_iter().collect()
					} else {
						error!(target: "sub-libp2p", "Was not able to get known addresses for {:?}", peer_id);
						return None
					};

					let endpoint = if let Some(e) =
						swarm.behaviour_mut().node(peer_id).and_then(|i| i.endpoint())
					{
						e.clone().into()
					} else {
						error!(target: "sub-libp2p", "Found state inconsistency between custom protocol \
						and debug information about {:?}", peer_id);
						return None
					};

					Some((
						peer_id.to_base58(),
						NetworkStatePeer {
							endpoint,
							version_string: swarm
								.behaviour_mut()
								.node(peer_id)
								.and_then(|i| i.client_version().map(|s| s.to_owned())),
							latest_ping_time: swarm
								.behaviour_mut()
								.node(peer_id)
								.and_then(|i| i.latest_ping()),
							known_addresses,
						},
					))
				})
				.collect()
		};

		let not_connected_peers = {
			let swarm = &mut *swarm;
			swarm
				.behaviour_mut()
				.known_peers()
				.into_iter()
				.filter(|p| open.iter().all(|n| n != p))
				.map(move |peer_id| {
					let known_addresses = if let Ok(addrs) =
						NetworkBehaviour::handle_pending_outbound_connection(
							swarm.behaviour_mut(),
							ConnectionId::new_unchecked(0), // dummy value
							Some(peer_id),
							&vec![],
							Endpoint::Listener,
						) {
						addrs.into_iter().collect()
					} else {
						error!(target: "sub-libp2p", "Was not able to get known addresses for {:?}", peer_id);
						Default::default()
					};

					(
						peer_id.to_base58(),
						NetworkStateNotConnectedPeer {
							version_string: swarm
								.behaviour_mut()
								.node(&peer_id)
								.and_then(|i| i.client_version().map(|s| s.to_owned())),
							latest_ping_time: swarm
								.behaviour_mut()
								.node(&peer_id)
								.and_then(|i| i.latest_ping()),
							known_addresses,
						},
					)
				})
				.collect()
		};

		let peer_id = Swarm::<Behaviour<B>>::local_peer_id(swarm).to_base58();
		let listened_addresses = swarm.listeners().cloned().collect();
		let external_addresses = swarm.external_addresses().map(|r| &r.addr).cloned().collect();

		NetworkState {
			peer_id,
			listened_addresses,
			external_addresses,
			connected_peers,
			not_connected_peers,
			// TODO: Check what info we can include here.
			//       Issue reference: https://github.com/paritytech/substrate/issues/14160.
			peerset: serde_json::json!(
				"Unimplemented. See https://github.com/paritytech/substrate/issues/14160."
			),
		}
	}

	/// Removes a `PeerId` from the list of reserved peers.
	pub fn remove_reserved_peer(&self, peer: PeerId) {
		self.service.remove_reserved_peer(peer);
	}

	/// Adds a `PeerId` and its `Multiaddr` as reserved.
	pub fn add_reserved_peer(&self, peer: MultiaddrWithPeerId) -> Result<(), String> {
		self.service.add_reserved_peer(peer)
	}
}

impl<B: BlockT + 'static, H: ExHashT> NetworkService<B, H> {
	/// Get network state.
	///
	/// **Note**: Use this only for debugging. This API is unstable. There are warnings literally
	/// everywhere about this. Please don't use this function to retrieve actual information.
	///
	/// Returns an error if the `NetworkWorker` is no longer running.
	pub async fn network_state(&self) -> Result<NetworkState, ()> {
		let (tx, rx) = oneshot::channel();

		let _ = self
			.to_worker
			.unbounded_send(ServiceToWorkerMsg::NetworkState { pending_response: tx });

		match rx.await {
			Ok(v) => v.map_err(|_| ()),
			// The channel can only be closed if the network worker no longer exists.
			Err(_) => Err(()),
		}
	}

	/// Get the list of reserved peers.
	///
	/// Returns an error if the `NetworkWorker` is no longer running.
	pub async fn reserved_peers(&self) -> Result<Vec<PeerId>, ()> {
		let (tx, rx) = oneshot::channel();

		self.sync_protocol_handle.reserved_peers(tx);

		// The channel can only be closed if `ProtocolController` no longer exists.
		rx.await.map_err(|_| ())
	}

	/// Utility function to extract `PeerId` from each `Multiaddr` for peer set updates.
	///
	/// Returns an `Err` if one of the given addresses is invalid or contains an
	/// invalid peer ID (which includes the local peer ID).
	fn split_multiaddr_and_peer_id(
		&self,
		peers: HashSet<Multiaddr>,
	) -> Result<Vec<(PeerId, Multiaddr)>, String> {
		peers
			.into_iter()
			.map(|mut addr| {
				let peer = match addr.pop() {
					Some(multiaddr::Protocol::P2p(key)) => PeerId::from_multihash(key)
						.map_err(|_| "Invalid PeerId format".to_string())?,
					_ => return Err("Missing PeerId from address".to_string()),
				};

				// Make sure the local peer ID is never added to the PSM
				// or added as a "known address", even if given.
				if peer == self.local_peer_id {
					Err("Local peer ID in peer set.".to_string())
				} else {
					Ok((peer, addr))
				}
			})
			.collect::<Result<Vec<(PeerId, Multiaddr)>, String>>()
	}
}

impl<B, H> NetworkStateInfo for NetworkService<B, H>
where
	B: sp_runtime::traits::Block,
	H: ExHashT,
{
	/// Returns the local external addresses.
	fn external_addresses(&self) -> Vec<Multiaddr> {
		self.external_addresses.lock().iter().cloned().collect()
	}

	/// Returns the listener addresses (without trailing `/p2p/` with our `PeerId`).
	fn listen_addresses(&self) -> Vec<Multiaddr> {
		self.listen_addresses.lock().iter().cloned().collect()
	}

	/// Returns the local Peer ID.
	fn local_peer_id(&self) -> PeerId {
		self.local_peer_id
	}
}

impl<B, H> NetworkSigner for NetworkService<B, H>
where
	B: sp_runtime::traits::Block,
	H: ExHashT,
{
	fn sign_with_local_identity(&self, msg: impl AsRef<[u8]>) -> Result<Signature, SigningError> {
		Signature::sign_message(msg.as_ref(), &self.local_identity)
	}
}

impl<B, H> NetworkDHTProvider for NetworkService<B, H>
where
	B: BlockT + 'static,
	H: ExHashT,
{
	/// Start getting a value from the DHT.
	///
	/// This will generate either a `ValueFound` or a `ValueNotFound` event and pass it as an
	/// item on the [`NetworkWorker`] stream.
	fn get_value(&self, key: &KademliaKey) {
		let _ = self.to_worker.unbounded_send(ServiceToWorkerMsg::GetValue(key.clone()));
	}

	/// Start putting a value in the DHT.
	///
	/// This will generate either a `ValuePut` or a `ValuePutFailed` event and pass it as an
	/// item on the [`NetworkWorker`] stream.
	fn put_value(&self, key: KademliaKey, value: Vec<u8>) {
		let _ = self.to_worker.unbounded_send(ServiceToWorkerMsg::PutValue(key, value));
	}
}

#[async_trait::async_trait]
impl<B, H> NetworkStatusProvider for NetworkService<B, H>
where
	B: BlockT + 'static,
	H: ExHashT,
{
	async fn status(&self) -> Result<NetworkStatus, ()> {
		let (tx, rx) = oneshot::channel();

		let _ = self
			.to_worker
			.unbounded_send(ServiceToWorkerMsg::NetworkStatus { pending_response: tx });

		match rx.await {
			Ok(v) => v.map_err(|_| ()),
			// The channel can only be closed if the network worker no longer exists.
			Err(_) => Err(()),
		}
	}
}

impl<B, H> NetworkPeers for NetworkService<B, H>
where
	B: BlockT + 'static,
	H: ExHashT,
{
	fn set_authorized_peers(&self, peers: HashSet<PeerId>) {
		self.sync_protocol_handle.set_reserved_peers(peers);
	}

	fn set_authorized_only(&self, reserved_only: bool) {
		self.sync_protocol_handle.set_reserved_only(reserved_only);
	}

	fn add_known_address(&self, peer_id: PeerId, addr: Multiaddr) {
		let _ = self
			.to_worker
			.unbounded_send(ServiceToWorkerMsg::AddKnownAddress(peer_id, addr));
	}

	fn report_peer(&self, peer_id: PeerId, cost_benefit: ReputationChange) {
		self.peer_store_handle.clone().report_peer(peer_id, cost_benefit);
	}

	fn peer_reputation(&self, peer_id: &PeerId) -> i32 {
		self.peer_store_handle.peer_reputation(peer_id)
	}

	fn disconnect_peer(&self, peer_id: PeerId, protocol: ProtocolName) {
		let _ = self
			.to_worker
			.unbounded_send(ServiceToWorkerMsg::DisconnectPeer(peer_id, protocol));
	}

	fn accept_unreserved_peers(&self) {
		self.sync_protocol_handle.set_reserved_only(false);
	}

	fn deny_unreserved_peers(&self) {
		self.sync_protocol_handle.set_reserved_only(true);
	}

	fn add_reserved_peer(&self, peer: MultiaddrWithPeerId) -> Result<(), String> {
		// Make sure the local peer ID is never added as a reserved peer.
		if peer.peer_id == self.local_peer_id {
			return Err("Local peer ID cannot be added as a reserved peer.".to_string())
		}

		let _ = self
			.to_worker
			.unbounded_send(ServiceToWorkerMsg::AddKnownAddress(peer.peer_id, peer.multiaddr));
		self.sync_protocol_handle.add_reserved_peer(peer.peer_id);
		Ok(())
	}

	fn remove_reserved_peer(&self, peer_id: PeerId) {
		self.sync_protocol_handle.remove_reserved_peer(peer_id);
	}

	fn set_reserved_peers(
		&self,
		protocol: ProtocolName,
		peers: HashSet<Multiaddr>,
	) -> Result<(), String> {
		let Some(set_id) = self.notification_protocol_ids.get(&protocol) else {
			return Err(format!("Cannot set reserved peers for unknown protocol: {}", protocol))
		};

		let peers_addrs = self.split_multiaddr_and_peer_id(peers)?;

		let mut peers: HashSet<PeerId> = HashSet::with_capacity(peers_addrs.len());

		for (peer_id, addr) in peers_addrs.into_iter() {
			// Make sure the local peer ID is never added to the PSM.
			if peer_id == self.local_peer_id {
				return Err("Local peer ID cannot be added as a reserved peer.".to_string())
			}

			peers.insert(peer_id);

			if !addr.is_empty() {
				let _ = self
					.to_worker
					.unbounded_send(ServiceToWorkerMsg::AddKnownAddress(peer_id, addr));
			}
		}

		self.protocol_handles[usize::from(*set_id)].set_reserved_peers(peers);

		Ok(())
	}

	fn add_peers_to_reserved_set(
		&self,
		protocol: ProtocolName,
		peers: HashSet<Multiaddr>,
	) -> Result<(), String> {
		let Some(set_id) = self.notification_protocol_ids.get(&protocol) else {
			return Err(format!(
				"Cannot add peers to reserved set of unknown protocol: {}",
				protocol
			))
		};

		let peers = self.split_multiaddr_and_peer_id(peers)?;

		for (peer_id, addr) in peers.into_iter() {
			// Make sure the local peer ID is never added to the PSM.
			if peer_id == self.local_peer_id {
				return Err("Local peer ID cannot be added as a reserved peer.".to_string())
			}

			if !addr.is_empty() {
				let _ = self
					.to_worker
					.unbounded_send(ServiceToWorkerMsg::AddKnownAddress(peer_id, addr));
			}

			self.protocol_handles[usize::from(*set_id)].add_reserved_peer(peer_id);
		}

		Ok(())
	}

	fn remove_peers_from_reserved_set(
		&self,
		protocol: ProtocolName,
		peers: Vec<PeerId>,
	) -> Result<(), String> {
		let Some(set_id) = self.notification_protocol_ids.get(&protocol) else {
			return Err(format!(
				"Cannot remove peers from reserved set of unknown protocol: {}",
				protocol
			))
		};

		for peer_id in peers.into_iter() {
			self.protocol_handles[usize::from(*set_id)].remove_reserved_peer(peer_id);
		}

		Ok(())
	}

	fn sync_num_connected(&self) -> usize {
		self.num_connected.load(Ordering::Relaxed)
	}

	fn peer_role(&self, peer_id: PeerId, handshake: Vec<u8>) -> Option<ObservedRole> {
		match Roles::decode_all(&mut &handshake[..]) {
			Ok(role) => Some(role.into()),
			Err(_) => {
				log::debug!(target: "sub-libp2p", "handshake doesn't contain peer role: {handshake:?}");
				self.peer_store_handle.peer_role(&peer_id)
			},
		}
	}
}

impl<B, H> NetworkEventStream for NetworkService<B, H>
where
	B: BlockT + 'static,
	H: ExHashT,
{
	fn event_stream(&self, name: &'static str) -> Pin<Box<dyn Stream<Item = Event> + Send>> {
		let (tx, rx) = out_events::channel(name, 100_000);
		let _ = self.to_worker.unbounded_send(ServiceToWorkerMsg::EventStream(tx));
		Box::pin(rx)
	}
}

impl<B, H> NetworkNotification for NetworkService<B, H>
where
	B: BlockT + 'static,
	H: ExHashT,
{
	fn write_notification(&self, _target: PeerId, _protocol: ProtocolName, _message: Vec<u8>) {
		unimplemented!();
	}

	fn notification_sender(
		&self,
		_target: PeerId,
		_protocol: ProtocolName,
	) -> Result<Box<dyn NotificationSenderT>, NotificationSenderError> {
		unimplemented!();
	}

	fn set_notification_handshake(&self, _protocol: ProtocolName, _handshake: Vec<u8>) {
		unimplemented!();
	}
}

#[async_trait::async_trait]
impl<B, H> NetworkRequest for NetworkService<B, H>
where
	B: BlockT + 'static,
	H: ExHashT,
{
	async fn request(
		&self,
		target: PeerId,
		protocol: ProtocolName,
		request: Vec<u8>,
		fallback_request: Option<(Vec<u8>, ProtocolName)>,
		connect: IfDisconnected,
	) -> Result<(Vec<u8>, ProtocolName), RequestFailure> {
		let (tx, rx) = oneshot::channel();

		self.start_request(target, protocol, request, fallback_request, tx, connect);

		match rx.await {
			Ok(v) => v,
			// The channel can only be closed if the network worker no longer exists. If the
			// network worker no longer exists, then all connections to `target` are necessarily
			// closed, and we legitimately report this situation as a "ConnectionClosed".
			Err(_) => Err(RequestFailure::Network(OutboundFailure::ConnectionClosed)),
		}
	}

	fn start_request(
		&self,
		target: PeerId,
		protocol: ProtocolName,
		request: Vec<u8>,
		fallback_request: Option<(Vec<u8>, ProtocolName)>,
		tx: oneshot::Sender<Result<(Vec<u8>, ProtocolName), RequestFailure>>,
		connect: IfDisconnected,
	) {
		let _ = self.to_worker.unbounded_send(ServiceToWorkerMsg::Request {
			target,
			protocol: protocol.into(),
			request,
			fallback_request,
			pending_response: tx,
			connect,
		});
	}
}

/// A `NotificationSender` allows for sending notifications to a peer with a chosen protocol.
#[must_use]
pub struct NotificationSender {
	sink: NotificationsSink,

	/// Name of the protocol on the wire.
	protocol_name: ProtocolName,

	/// Field extracted from the [`Metrics`] struct and necessary to report the
	/// notifications-related metrics.
	notification_size_metric: Option<Histogram>,
}

#[async_trait::async_trait]
impl NotificationSenderT for NotificationSender {
	async fn ready(
		&self,
	) -> Result<Box<dyn NotificationSenderReadyT + '_>, NotificationSenderError> {
		Ok(Box::new(NotificationSenderReady {
			ready: match self.sink.reserve_notification().await {
				Ok(r) => Some(r),
				Err(()) => return Err(NotificationSenderError::Closed),
			},
			peer_id: self.sink.peer_id(),
			protocol_name: &self.protocol_name,
			notification_size_metric: self.notification_size_metric.clone(),
		}))
	}
}

/// Reserved slot in the notifications buffer, ready to accept data.
#[must_use]
pub struct NotificationSenderReady<'a> {
	ready: Option<Ready<'a>>,

	/// Target of the notification.
	peer_id: &'a PeerId,

	/// Name of the protocol on the wire.
	protocol_name: &'a ProtocolName,

	/// Field extracted from the [`Metrics`] struct and necessary to report the
	/// notifications-related metrics.
	notification_size_metric: Option<Histogram>,
}

impl<'a> NotificationSenderReadyT for NotificationSenderReady<'a> {
	fn send(&mut self, notification: Vec<u8>) -> Result<(), NotificationSenderError> {
		if let Some(notification_size_metric) = &self.notification_size_metric {
			notification_size_metric.observe(notification.len() as f64);
		}

		trace!(
			target: "sub-libp2p",
			"External API => Notification({:?}, {}, {} bytes)",
			self.peer_id, self.protocol_name, notification.len(),
		);
		trace!(target: "sub-libp2p", "Handler({:?}) <= Async notification", self.peer_id);

		self.ready
			.take()
			.ok_or(NotificationSenderError::Closed)?
			.send(notification)
			.map_err(|()| NotificationSenderError::Closed)
	}
}

/// Messages sent from the `NetworkService` to the `NetworkWorker`.
///
/// Each entry corresponds to a method of `NetworkService`.
enum ServiceToWorkerMsg {
	GetValue(KademliaKey),
	PutValue(KademliaKey, Vec<u8>),
	AddKnownAddress(PeerId, Multiaddr),
	EventStream(out_events::Sender),
	Request {
		target: PeerId,
		protocol: ProtocolName,
		request: Vec<u8>,
		fallback_request: Option<(Vec<u8>, ProtocolName)>,
		pending_response: oneshot::Sender<Result<(Vec<u8>, ProtocolName), RequestFailure>>,
		connect: IfDisconnected,
	},
	NetworkStatus {
		pending_response: oneshot::Sender<Result<NetworkStatus, RequestFailure>>,
	},
	NetworkState {
		pending_response: oneshot::Sender<Result<NetworkState, RequestFailure>>,
	},
	DisconnectPeer(PeerId, ProtocolName),
}

/// Main network worker. Must be polled in order for the network to advance.
///
/// You are encouraged to poll this in a separate background thread or task.
#[must_use = "The NetworkWorker must be polled in order for the network to advance"]
pub struct NetworkWorker<B, H>
where
	B: BlockT + 'static,
	H: ExHashT,
{
	/// Updated by the `NetworkWorker` and loaded by the `NetworkService`.
	listen_addresses: Arc<Mutex<HashSet<Multiaddr>>>,
	/// Updated by the `NetworkWorker` and loaded by the `NetworkService`.
	num_connected: Arc<AtomicUsize>,
	/// The network service that can be extracted and shared through the codebase.
	service: Arc<NetworkService<B, H>>,
	/// The *actual* network.
	network_service: Swarm<Behaviour<B>>,
	/// Messages from the [`NetworkService`] that must be processed.
	from_service: TracingUnboundedReceiver<ServiceToWorkerMsg>,
	/// Senders for events that happen on the network.
	event_streams: out_events::OutChannels,
	/// Prometheus network metrics.
	metrics: Option<Metrics>,
	/// The `PeerId`'s of all boot nodes mapped to the registered addresses.
	boot_node_ids: Arc<HashMap<PeerId, Vec<Multiaddr>>>,
	/// Boot nodes that we already have reported as invalid.
	reported_invalid_boot_nodes: HashSet<PeerId>,
	/// Peer reputation store handle.
	peer_store_handle: PeerStoreHandle,
	/// Notification protocol handles.
	notif_protocol_handles: Vec<protocol::ProtocolHandle>,
	/// Marker to pin the `H` generic. Serves no purpose except to not break backwards
	/// compatibility.
	_marker: PhantomData<H>,
	/// Marker for block type
	_block: PhantomData<B>,
}

impl<B, H> NetworkWorker<B, H>
where
	B: BlockT + 'static,
	H: ExHashT,
{
	/// Run the network.
	pub async fn run(mut self) {
		while self.next_action().await {}
	}

	/// Perform one action on the network.
	///
	/// Returns `false` when the worker should be shutdown.
	/// Use in tests only.
	pub async fn next_action(&mut self) -> bool {
		futures::select! {
			// Next message from the service.
			msg = self.from_service.next() => {
				if let Some(msg) = msg {
					self.handle_worker_message(msg);
				} else {
					return false
				}
			},
			// Next event from `Swarm` (the stream guaranteed to never terminate).
			event = self.network_service.select_next_some() => {
				self.handle_swarm_event(event);
			},
		};

		// Update the `num_connected` count shared with the `NetworkService`.
		let num_connected_peers = self.network_service.behaviour().user_protocol().num_sync_peers();
		self.num_connected.store(num_connected_peers, Ordering::Relaxed);

		if let Some(metrics) = self.metrics.as_ref() {
			if let Some(buckets) = self.network_service.behaviour_mut().num_entries_per_kbucket() {
				for (lower_ilog2_bucket_bound, num_entries) in buckets {
					metrics
						.kbuckets_num_nodes
						.with_label_values(&[&lower_ilog2_bucket_bound.to_string()])
						.set(num_entries as u64);
				}
			}
			if let Some(num_entries) = self.network_service.behaviour_mut().num_kademlia_records() {
				metrics.kademlia_records_count.set(num_entries as u64);
			}
			if let Some(num_entries) =
				self.network_service.behaviour_mut().kademlia_records_total_size()
			{
				metrics.kademlia_records_sizes_total.set(num_entries as u64);
			}
			metrics
				.peerset_num_discovered
				.set(self.peer_store_handle.num_known_peers() as u64);
			metrics.pending_connections.set(
				Swarm::network_info(&self.network_service).connection_counters().num_pending()
					as u64,
			);
		}

		true
	}

	/// Process the next message coming from the `NetworkService`.
	fn handle_worker_message(&mut self, msg: ServiceToWorkerMsg) {
		match msg {
			ServiceToWorkerMsg::GetValue(key) =>
				self.network_service.behaviour_mut().get_value(key),
			ServiceToWorkerMsg::PutValue(key, value) =>
				self.network_service.behaviour_mut().put_value(key, value),
			ServiceToWorkerMsg::AddKnownAddress(peer_id, addr) =>
				self.network_service.behaviour_mut().add_known_address(peer_id, addr),
			ServiceToWorkerMsg::EventStream(sender) => self.event_streams.push(sender),
			ServiceToWorkerMsg::Request {
				target,
				protocol,
				request,
				fallback_request,
				pending_response,
				connect,
			} => {
				self.network_service.behaviour_mut().send_request(
					&target,
					protocol,
					request,
					fallback_request,
					pending_response,
					connect,
				);
			},
			ServiceToWorkerMsg::NetworkStatus { pending_response } => {
				let _ = pending_response.send(Ok(self.status()));
			},
			ServiceToWorkerMsg::NetworkState { pending_response } => {
				let _ = pending_response.send(Ok(self.network_state()));
			},
			ServiceToWorkerMsg::DisconnectPeer(who, protocol_name) => self
				.network_service
				.behaviour_mut()
				.user_protocol_mut()
				.disconnect_peer(&who, protocol_name),
		}
	}

	/// Process the next event coming from `Swarm`.
	fn handle_swarm_event(&mut self, event: SwarmEvent<BehaviourOut, THandlerErr<Behaviour<B>>>) {
		match event {
			SwarmEvent::Behaviour(BehaviourOut::InboundRequest { protocol, result, .. }) => {
				if let Some(metrics) = self.metrics.as_ref() {
					match result {
						Ok(serve_time) => {
							metrics
								.requests_in_success_total
								.with_label_values(&[&protocol])
								.observe(serve_time.as_secs_f64());
						},
						Err(err) => {
							let reason = match err {
								ResponseFailure::Network(InboundFailure::Timeout) =>
									Some("timeout"),
								ResponseFailure::Network(InboundFailure::UnsupportedProtocols) =>
								// `UnsupportedProtocols` is reported for every single
								// inbound request whenever a request with an unsupported
								// protocol is received. This is not reported in order to
								// avoid confusions.
									None,
								ResponseFailure::Network(InboundFailure::ResponseOmission) =>
									Some("busy-omitted"),
								ResponseFailure::Network(InboundFailure::ConnectionClosed) =>
									Some("connection-closed"),
							};

							if let Some(reason) = reason {
								metrics
									.requests_in_failure_total
									.with_label_values(&[&protocol, reason])
									.inc();
							}
						},
					}
				}
			},
			SwarmEvent::Behaviour(BehaviourOut::RequestFinished {
				protocol,
				duration,
				result,
				..
			}) =>
				if let Some(metrics) = self.metrics.as_ref() {
					match result {
						Ok(_) => {
							metrics
								.requests_out_success_total
								.with_label_values(&[&protocol])
								.observe(duration.as_secs_f64());
						},
						Err(err) => {
							let reason = match err {
								RequestFailure::NotConnected => "not-connected",
								RequestFailure::UnknownProtocol => "unknown-protocol",
								RequestFailure::Refused => "refused",
								RequestFailure::Obsolete => "obsolete",
								RequestFailure::Network(OutboundFailure::DialFailure) =>
									"dial-failure",
								RequestFailure::Network(OutboundFailure::Timeout) => "timeout",
								RequestFailure::Network(OutboundFailure::ConnectionClosed) =>
									"connection-closed",
								RequestFailure::Network(OutboundFailure::UnsupportedProtocols) =>
									"unsupported",
							};

							metrics
								.requests_out_failure_total
								.with_label_values(&[&protocol, reason])
								.inc();
						},
					}
				},
			SwarmEvent::Behaviour(BehaviourOut::ReputationChanges { peer, changes }) => {
				for change in changes {
					self.peer_store_handle.report_peer(peer, change);
				}
			},
			SwarmEvent::Behaviour(BehaviourOut::PeerIdentify {
				peer_id,
				info:
					IdentifyInfo {
						protocol_version, agent_version, mut listen_addrs, protocols, ..
					},
			}) => {
				if listen_addrs.len() > 30 {
					debug!(
						target: "sub-libp2p",
						"Node {:?} has reported more than 30 addresses; it is identified by {:?} and {:?}",
						peer_id, protocol_version, agent_version
					);
					listen_addrs.truncate(30);
				}
				for addr in listen_addrs {
					self.network_service
						.behaviour_mut()
						.add_self_reported_address_to_dht(&peer_id, &protocols, addr);
				}
				self.peer_store_handle.add_known_peer(peer_id);
			},
			SwarmEvent::Behaviour(BehaviourOut::Discovered(peer_id)) => {
				self.peer_store_handle.add_known_peer(peer_id);
			},
			SwarmEvent::Behaviour(BehaviourOut::RandomKademliaStarted) => {
				if let Some(metrics) = self.metrics.as_ref() {
					metrics.kademlia_random_queries_total.inc();
				}
			},
			SwarmEvent::Behaviour(BehaviourOut::NotificationStreamOpened {
				remote,
				set_id,
				direction,
				negotiated_fallback,
				notifications_sink,
				received_handshake,
			}) => {
				let _ = self.notif_protocol_handles[usize::from(set_id)].report_substream_opened(
					remote,
					direction,
					received_handshake,
					negotiated_fallback,
					notifications_sink,
				);
			},
			SwarmEvent::Behaviour(BehaviourOut::NotificationStreamReplaced {
				remote,
				set_id,
				notifications_sink,
			}) => {
				let _ = self.notif_protocol_handles[usize::from(set_id)]
					.report_notification_sink_replaced(remote, notifications_sink);

				// TODO: Notifications might have been lost as a result of the previous
				// connection being dropped, and as a result it would be preferable to notify
				// the users of this fact by simulating the substream being closed then
				// reopened.
				// The code below doesn't compile because `role` is unknown. Propagating the
				// handshake of the secondary connections is quite an invasive change and
				// would conflict with https://github.com/paritytech/substrate/issues/6403.
				// Considering that dropping notifications is generally regarded as
				// acceptable, this bug is at the moment intentionally left there and is
				// intended to be fixed at the same time as
				// https://github.com/paritytech/substrate/issues/6403.
				// self.event_streams.send(Event::NotificationStreamClosed {
				// remote,
				// protocol,
				// });
				// self.event_streams.send(Event::NotificationStreamOpened {
				// remote,
				// protocol,
				// role,
				// });
			},
			SwarmEvent::Behaviour(BehaviourOut::NotificationStreamClosed { remote, set_id }) => {
				let _ = self.notif_protocol_handles[usize::from(set_id)]
					.report_substream_closed(remote);
			},
			SwarmEvent::Behaviour(BehaviourOut::NotificationsReceived {
				remote,
				set_id,
				notification,
			}) => {
				let _ = self.notif_protocol_handles[usize::from(set_id)]
					.report_notification_received(remote, notification);
			},
			SwarmEvent::Behaviour(BehaviourOut::Dht(event, duration)) => {
				if let Some(metrics) = self.metrics.as_ref() {
					let query_type = match event {
						DhtEvent::ValueFound(_) => "value-found",
						DhtEvent::ValueNotFound(_) => "value-not-found",
						DhtEvent::ValuePut(_) => "value-put",
						DhtEvent::ValuePutFailed(_) => "value-put-failed",
					};
					metrics
						.kademlia_query_duration
						.with_label_values(&[query_type])
						.observe(duration.as_secs_f64());
				}

				self.event_streams.send(Event::Dht(event));
			},
			SwarmEvent::Behaviour(BehaviourOut::None) => {
				// Ignored event from lower layers.
			},
			SwarmEvent::ConnectionEstablished {
				peer_id,
				endpoint,
				num_established,
				concurrent_dial_errors,
				..
			} => {
				if let Some(errors) = concurrent_dial_errors {
					debug!(target: "sub-libp2p", "Libp2p => Connected({:?}) with errors: {:?}", peer_id, errors);
				} else {
					debug!(target: "sub-libp2p", "Libp2p => Connected({:?})", peer_id);
				}

				if let Some(metrics) = self.metrics.as_ref() {
					let direction = match endpoint {
						ConnectedPoint::Dialer { .. } => "out",
						ConnectedPoint::Listener { .. } => "in",
					};
					metrics.connections_opened_total.with_label_values(&[direction]).inc();

					if num_established.get() == 1 {
						metrics.distinct_peers_connections_opened_total.inc();
					}
				}
			},
			SwarmEvent::ConnectionClosed { peer_id, cause, endpoint, num_established } => {
				debug!(target: "sub-libp2p", "Libp2p => Disconnected({:?}, {:?})", peer_id, cause);
				if let Some(metrics) = self.metrics.as_ref() {
					let direction = match endpoint {
						ConnectedPoint::Dialer { .. } => "out",
						ConnectedPoint::Listener { .. } => "in",
					};
					let reason = match cause {
						Some(ConnectionError::IO(_)) => "transport-error",
						Some(ConnectionError::Handler(Either::Left(Either::Left(
							Either::Right(Either::Left(PingFailure::Timeout)),
						)))) => "ping-timeout",
						Some(ConnectionError::Handler(Either::Left(Either::Left(
							Either::Left(NotifsHandlerError::SyncNotificationsClogged),
						)))) => "sync-notifications-clogged",
						Some(ConnectionError::Handler(_)) => "protocol-error",
						Some(ConnectionError::KeepAliveTimeout) => "keep-alive-timeout",
						None => "actively-closed",
					};
					metrics.connections_closed_total.with_label_values(&[direction, reason]).inc();

					// `num_established` represents the number of *remaining* connections.
					if num_established == 0 {
						metrics.distinct_peers_connections_closed_total.inc();
					}
				}
			},
			SwarmEvent::NewListenAddr { address, .. } => {
				trace!(target: "sub-libp2p", "Libp2p => NewListenAddr({})", address);
				if let Some(metrics) = self.metrics.as_ref() {
					metrics.listeners_local_addresses.inc();
				}
				self.listen_addresses.lock().insert(address.clone());
			},
			SwarmEvent::ExpiredListenAddr { address, .. } => {
				info!(target: "sub-libp2p", "No longer listening on {}", address);
				if let Some(metrics) = self.metrics.as_ref() {
					metrics.listeners_local_addresses.dec();
				}
				self.listen_addresses.lock().remove(&address);
			},
			SwarmEvent::OutgoingConnectionError { peer_id, error } => {
				if let Some(peer_id) = peer_id {
					trace!(
						target: "sub-libp2p",
						"Libp2p => Failed to reach {:?}: {}",
						peer_id, error,
					);

					let not_reported = !self.reported_invalid_boot_nodes.contains(&peer_id);

					if let Some(addresses) =
						not_reported.then(|| self.boot_node_ids.get(&peer_id)).flatten()
					{
						if let DialError::WrongPeerId { obtained, endpoint } = &error {
							if let ConnectedPoint::Dialer { address, role_override: _ } = endpoint {
								let address_without_peer_id = parse_addr(address.clone())
									.map_or_else(|_| address.clone(), |r| r.1);

								// Only report for address of boot node that was added at startup of
								// the node and not for any address that the node learned of the
								// boot node.
								if addresses.iter().any(|a| address_without_peer_id == *a) {
									warn!(
										"The bootnode you want to connect to at `{address}` provided a \
										 different peer ID `{obtained}` than the one you expect `{peer_id}`.",
									);

									self.reported_invalid_boot_nodes.insert(peer_id);
								}
							}
						}
					}
				}

				if let Some(metrics) = self.metrics.as_ref() {
					#[allow(deprecated)]
					let reason = match error {
						DialError::Denied { cause } =>
							if cause.downcast::<Exceeded>().is_ok() {
								Some("limit-reached")
							} else {
								None
							},
						DialError::ConnectionLimit(_) => Some("limit-reached"),
						DialError::InvalidPeerId(_) |
						DialError::WrongPeerId { .. } |
						DialError::LocalPeerId { .. } => Some("invalid-peer-id"),
						DialError::Transport(_) => Some("transport-error"),
						DialError::Banned |
						DialError::NoAddresses |
						DialError::DialPeerConditionFalse(_) |
						DialError::Aborted => None, // ignore them
					};
					if let Some(reason) = reason {
						metrics.pending_connections_errors_total.with_label_values(&[reason]).inc();
					}
				}
			},
			SwarmEvent::Dialing(peer_id) => {
				trace!(target: "sub-libp2p", "Libp2p => Dialing({:?})", peer_id)
			},
			SwarmEvent::IncomingConnection { local_addr, send_back_addr } => {
				trace!(target: "sub-libp2p", "Libp2p => IncomingConnection({},{}))",
					local_addr, send_back_addr);
				if let Some(metrics) = self.metrics.as_ref() {
					metrics.incoming_connections_total.inc();
				}
			},
			SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error } => {
				debug!(
					target: "sub-libp2p",
					"Libp2p => IncomingConnectionError({},{}): {}",
					local_addr, send_back_addr, error,
				);
				if let Some(metrics) = self.metrics.as_ref() {
					#[allow(deprecated)]
					let reason = match error {
						ListenError::Denied { cause } =>
							if cause.downcast::<Exceeded>().is_ok() {
								Some("limit-reached")
							} else {
								None
							},
						ListenError::ConnectionLimit(_) => Some("limit-reached"),
						ListenError::WrongPeerId { .. } | ListenError::LocalPeerId { .. } =>
							Some("invalid-peer-id"),
						ListenError::Transport(_) => Some("transport-error"),
						ListenError::Aborted => None, // ignore it
					};

					if let Some(reason) = reason {
						metrics
							.incoming_connections_errors_total
							.with_label_values(&[reason])
							.inc();
					}
				}
			},
			#[allow(deprecated)]
			SwarmEvent::BannedPeer { peer_id, endpoint } => {
				debug!(
					target: "sub-libp2p",
					"Libp2p => BannedPeer({}). Connected via {:?}.",
					peer_id, endpoint,
				);
				if let Some(metrics) = self.metrics.as_ref() {
					metrics.incoming_connections_errors_total.with_label_values(&["banned"]).inc();
				}
			},
			SwarmEvent::ListenerClosed { reason, addresses, .. } => {
				if let Some(metrics) = self.metrics.as_ref() {
					metrics.listeners_local_addresses.sub(addresses.len() as u64);
				}
				let mut listen_addresses = self.listen_addresses.lock();
				for addr in &addresses {
					listen_addresses.remove(addr);
				}
				drop(listen_addresses);

				let addrs =
					addresses.into_iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
				match reason {
					Ok(()) => error!(
						target: "sub-libp2p",
						"Libp2p listener ({}) closed gracefully",
						addrs
					),
					Err(e) => error!(
						target: "sub-libp2p",
						"Libp2p listener ({}) closed: {}",
						addrs, e
					),
				}
			},
			SwarmEvent::ListenerError { error, .. } => {
				debug!(target: "sub-libp2p", "Libp2p => ListenerError: {}", error);
				if let Some(metrics) = self.metrics.as_ref() {
					metrics.listeners_errors_total.inc();
				}
			},
		}
	}
}

impl<B, H> Unpin for NetworkWorker<B, H>
where
	B: BlockT + 'static,
	H: ExHashT,
{
}

fn ensure_addresses_consistent_with_transport<'a>(
	addresses: impl Iterator<Item = &'a Multiaddr>,
	transport: &TransportConfig,
) -> Result<(), Error> {
	if matches!(transport, TransportConfig::MemoryOnly) {
		let addresses: Vec<_> = addresses
			.filter(|x| {
				x.iter().any(|y| !matches!(y, libp2p::core::multiaddr::Protocol::Memory(_)))
			})
			.cloned()
			.collect();

		if !addresses.is_empty() {
			return Err(Error::AddressesForAnotherTransport {
				transport: transport.clone(),
				addresses,
			})
		}
	} else {
		let addresses: Vec<_> = addresses
			.filter(|x| x.iter().any(|y| matches!(y, libp2p::core::multiaddr::Protocol::Memory(_))))
			.cloned()
			.collect();

		if !addresses.is_empty() {
			return Err(Error::AddressesForAnotherTransport {
				transport: transport.clone(),
				addresses,
			})
		}
	}

	Ok(())
}
