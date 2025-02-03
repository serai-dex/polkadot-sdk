// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

/// A no-std/Substrate compatible library to construct binary merkle tree.
#[cfg(feature = "binary-merkle-tree")]
pub use binary_merkle_tree;

/// Utility library for managing tree-like ordered data with logic for pruning the tree while
/// finalizing nodes.
#[cfg(feature = "fork-tree")]
pub use fork_tree;

/// Macro for benchmarking a FRAME runtime.
#[cfg(feature = "frame-benchmarking")]
pub use frame_benchmarking;

/// CLI for benchmarking FRAME.
#[cfg(feature = "frame-benchmarking-cli")]
pub use frame_benchmarking_cli;

/// Pallet for testing FRAME PoV benchmarking.
#[cfg(feature = "frame-benchmarking-pallet-pov")]
pub use frame_benchmarking_pallet_pov;

/// NPoS Solution Type.
#[cfg(feature = "frame-election-provider-solution-type")]
pub use frame_election_provider_solution_type;

/// election provider supporting traits.
#[cfg(feature = "frame-election-provider-support")]
pub use frame_election_provider_support;

/// FRAME executives engine.
#[cfg(feature = "frame-executive")]
pub use frame_executive;

/// FRAME signed extension for verifying the metadata hash.
#[cfg(feature = "frame-metadata-hash-extension")]
pub use frame_metadata_hash_extension;

/// An externalities provided environment that can load itself from remote nodes or cached
/// files.
#[cfg(feature = "frame-remote-externalities")]
pub use frame_remote_externalities;

/// Support code for the runtime.
#[cfg(feature = "frame-support")]
pub use frame_support;

/// Proc macro of Support code for the runtime.
#[cfg(feature = "frame-support-procedural")]
pub use frame_support_procedural;

/// Proc macro helpers for procedural macros.
#[cfg(feature = "frame-support-procedural-tools")]
pub use frame_support_procedural_tools;

/// Use to derive parsing for parsing struct.
#[cfg(feature = "frame-support-procedural-tools-derive")]
pub use frame_support_procedural_tools_derive;

/// FRAME system module.
#[cfg(feature = "frame-system")]
pub use frame_system;

/// FRAME System benchmarking.
#[cfg(feature = "frame-system-benchmarking")]
pub use frame_system_benchmarking;

/// Runtime API definition required by System RPC extensions.
#[cfg(feature = "frame-system-rpc-runtime-api")]
pub use frame_system_rpc_runtime_api;

/// Supporting types for try-runtime, testing and dry-running commands.
#[cfg(feature = "frame-try-runtime")]
pub use frame_try_runtime;

/// Bag threshold generation script for pallet-bag-list.
#[cfg(feature = "generate-bags")]
pub use generate_bags;

/// FRAME asset conversion pallet.
#[cfg(feature = "pallet-asset-conversion")]
pub use pallet_asset_conversion;

/// FRAME asset conversion pallet's operations suite.
#[cfg(feature = "pallet-asset-conversion-ops")]
pub use pallet_asset_conversion_ops;

/// Pallet to manage transaction payments in assets by converting them to native assets.
#[cfg(feature = "pallet-asset-conversion-tx-payment")]
pub use pallet_asset_conversion_tx_payment;

/// Whitelist non-native assets for treasury spending and provide conversion to native balance.
#[cfg(feature = "pallet-asset-rate")]
pub use pallet_asset_rate;

/// pallet to manage transaction payments in assets.
#[cfg(feature = "pallet-asset-tx-payment")]
pub use pallet_asset_tx_payment;

/// FRAME asset management pallet.
#[cfg(feature = "pallet-assets")]
pub use pallet_assets;

/// Provides freezing features to `pallet-assets`.
#[cfg(feature = "pallet-assets-freezer")]
pub use pallet_assets_freezer;

/// FRAME AURA consensus pallet.
#[cfg(feature = "pallet-aura")]
pub use pallet_aura;

/// FRAME pallet for authority discovery.
#[cfg(feature = "pallet-authority-discovery")]
pub use pallet_authority_discovery;

/// Block and Uncle Author tracking for the FRAME.
#[cfg(feature = "pallet-authorship")]
pub use pallet_authorship;

/// Consensus extension module for BABE consensus. Collects on-chain randomness from VRF
/// outputs and manages epoch transitions.
#[cfg(feature = "pallet-babe")]
pub use pallet_babe;

/// FRAME pallet bags list.
#[cfg(feature = "pallet-bags-list")]
pub use pallet_bags_list;

/// FRAME pallet to manage balances.
#[cfg(feature = "pallet-balances")]
pub use pallet_balances;

/// PALLET two phase election providers.
#[cfg(feature = "pallet-election-provider-multi-phase")]
pub use pallet_election_provider_multi_phase;

/// Benchmarking for election provider support onchain config trait.
#[cfg(feature = "pallet-election-provider-support-benchmarking")]
pub use pallet_election_provider_support_benchmarking;

/// FRAME pallet for pushing a chain to its weight limits.
#[cfg(feature = "pallet-glutton")]
pub use pallet_glutton;

/// FRAME pallet for GRANDPA finality gadget.
#[cfg(feature = "pallet-grandpa")]
pub use pallet_grandpa;

/// FRAME identity management pallet.
#[cfg(feature = "pallet-identity")]
pub use pallet_identity;

/// FRAME's I'm online pallet.
#[cfg(feature = "pallet-im-online")]
pub use pallet_im_online;

/// FRAME indices management pallet.
#[cfg(feature = "pallet-indices")]
pub use pallet_indices;

/// Insecure do not use in production: FRAME randomness collective flip pallet.
#[cfg(feature = "pallet-insecure-randomness-collective-flip")]
pub use pallet_insecure_randomness_collective_flip;

/// FRAME pallet to queue and process messages.
#[cfg(feature = "pallet-message-queue")]
pub use pallet_message_queue;

/// FRAME pallet to execute multi-block migrations.
#[cfg(feature = "pallet-migrations")]
pub use pallet_migrations;

/// FRAME's mixnet pallet.
#[cfg(feature = "pallet-mixnet")]
pub use pallet_mixnet;

/// FRAME multi-signature dispatch pallet.
#[cfg(feature = "pallet-multisig")]
pub use pallet_multisig;

/// FRAME pallet to convert non-fungible to fungible tokens.
#[cfg(feature = "pallet-nft-fractionalization")]
pub use pallet_nft_fractionalization;

/// FRAME NFTs pallet.
#[cfg(feature = "pallet-nfts")]
pub use pallet_nfts;

/// Runtime API for the FRAME NFTs pallet.
#[cfg(feature = "pallet-nfts-runtime-api")]
pub use pallet_nfts_runtime_api;

/// FRAME pallet for rewarding account freezing.
#[cfg(feature = "pallet-nis")]
pub use pallet_nis;

/// FRAME offences pallet.
#[cfg(feature = "pallet-offences")]
pub use pallet_offences;

/// FRAME offences pallet benchmarking.
#[cfg(feature = "pallet-offences-benchmarking")]
pub use pallet_offences_benchmarking;

/// FRAME pallet that provides a paged list data structure.
#[cfg(feature = "pallet-paged-list")]
pub use pallet_paged_list;

/// FRAME pallet for storing preimages of hashes.
#[cfg(feature = "pallet-preimage")]
pub use pallet_preimage;

/// FRAME proxying pallet.
#[cfg(feature = "pallet-proxy")]
pub use pallet_proxy;

/// Remark storage pallet.
#[cfg(feature = "pallet-remark")]
pub use pallet_remark;

/// FRAME root offences pallet.
#[cfg(feature = "pallet-root-offences")]
pub use pallet_root_offences;

/// FRAME root testing pallet.
#[cfg(feature = "pallet-root-testing")]
pub use pallet_root_testing;

/// FRAME safe-mode pallet.
#[cfg(feature = "pallet-safe-mode")]
pub use pallet_safe_mode;

/// FRAME Scheduler pallet.
#[cfg(feature = "pallet-scheduler")]
pub use pallet_scheduler;

/// FRAME sessions pallet.
#[cfg(feature = "pallet-session")]
pub use pallet_session;

/// FRAME sessions pallet benchmarking.
#[cfg(feature = "pallet-session-benchmarking")]
pub use pallet_session_benchmarking;

/// Pallet to skip payments for calls annotated with `feeless_if` if the respective conditions
/// are satisfied.
#[cfg(feature = "pallet-skip-feeless-payment")]
pub use pallet_skip_feeless_payment;

/// FRAME pallet staking.
#[cfg(feature = "pallet-staking")]
pub use pallet_staking;

/// Reward Curve for FRAME staking pallet.
#[cfg(feature = "pallet-staking-reward-curve")]
pub use pallet_staking_reward_curve;

/// Reward function for FRAME staking pallet.
#[cfg(feature = "pallet-staking-reward-fn")]
pub use pallet_staking_reward_fn;

/// RPC runtime API for transaction payment FRAME pallet.
#[cfg(feature = "pallet-staking-runtime-api")]
pub use pallet_staking_runtime_api;

/// FRAME pallet migration of trie.
#[cfg(feature = "pallet-state-trie-migration")]
pub use pallet_state_trie_migration;

/// FRAME pallet for sudo.
#[cfg(feature = "pallet-sudo")]
pub use pallet_sudo;

/// FRAME Timestamp Module.
#[cfg(feature = "pallet-timestamp")]
pub use pallet_timestamp;

/// FRAME pallet to manage transaction payments.
#[cfg(feature = "pallet-transaction-payment")]
pub use pallet_transaction_payment;

/// RPC interface for the transaction payment pallet.
#[cfg(feature = "pallet-transaction-payment-rpc")]
pub use pallet_transaction_payment_rpc;

/// RPC runtime API for transaction payment FRAME pallet.
#[cfg(feature = "pallet-transaction-payment-rpc-runtime-api")]
pub use pallet_transaction_payment_rpc_runtime_api;

/// Storage chain pallet.
#[cfg(feature = "pallet-transaction-storage")]
pub use pallet_transaction_storage;

/// FRAME transaction pause pallet.
#[cfg(feature = "pallet-tx-pause")]
pub use pallet_tx_pause;

/// FRAME NFT asset management pallet.
#[cfg(feature = "pallet-uniques")]
pub use pallet_uniques;

/// FRAME utilities pallet.
#[cfg(feature = "pallet-utility")]
pub use pallet_utility;

/// FRAME verify signature pallet.
#[cfg(feature = "pallet-verify-signature")]
pub use pallet_verify_signature;

/// FRAME pallet for whitelisting calls, and dispatching from a specific origin.
#[cfg(feature = "pallet-whitelist")]
pub use pallet_whitelist;

/// Collection of allocator implementations.
#[cfg(feature = "sc-allocator")]
pub use sc_allocator;

/// Substrate authority discovery.
#[cfg(feature = "sc-authority-discovery")]
pub use sc_authority_discovery;

/// Basic implementation of block-authoring logic.
#[cfg(feature = "sc-basic-authorship")]
pub use sc_basic_authorship;

/// Substrate block builder.
#[cfg(feature = "sc-block-builder")]
pub use sc_block_builder;

/// Substrate chain configurations.
#[cfg(feature = "sc-chain-spec")]
pub use sc_chain_spec;

/// Macros to derive chain spec extension traits implementation.
#[cfg(feature = "sc-chain-spec-derive")]
pub use sc_chain_spec_derive;

/// Substrate CLI interface.
#[cfg(feature = "sc-cli")]
pub use sc_cli;

/// Substrate client interfaces.
#[cfg(feature = "sc-client-api")]
pub use sc_client_api;

/// Client backend that uses RocksDB database as storage.
#[cfg(feature = "sc-client-db")]
pub use sc_client_db;

/// Collection of common consensus specific implementations for Substrate (client).
#[cfg(feature = "sc-consensus")]
pub use sc_consensus;

/// Aura consensus algorithm for substrate.
#[cfg(feature = "sc-consensus-aura")]
pub use sc_consensus_aura;

/// BABE consensus algorithm for substrate.
#[cfg(feature = "sc-consensus-babe")]
pub use sc_consensus_babe;

/// RPC extensions for the BABE consensus algorithm.
#[cfg(feature = "sc-consensus-babe-rpc")]
pub use sc_consensus_babe_rpc;

/// Generic epochs-based utilities for consensus.
#[cfg(feature = "sc-consensus-epochs")]
pub use sc_consensus_epochs;

/// Integration of the GRANDPA finality gadget into substrate.
#[cfg(feature = "sc-consensus-grandpa")]
pub use sc_consensus_grandpa;

/// RPC extensions for the GRANDPA finality gadget.
#[cfg(feature = "sc-consensus-grandpa-rpc")]
pub use sc_consensus_grandpa_rpc;

/// Manual sealing engine for Substrate.
#[cfg(feature = "sc-consensus-manual-seal")]
pub use sc_consensus_manual_seal;

/// PoW consensus algorithm for substrate.
#[cfg(feature = "sc-consensus-pow")]
pub use sc_consensus_pow;

/// Generic slots-based utilities for consensus.
#[cfg(feature = "sc-consensus-slots")]
pub use sc_consensus_slots;

/// A crate that provides means of executing/dispatching calls into the runtime.
#[cfg(feature = "sc-executor")]
pub use sc_executor;

/// A set of common definitions that are needed for defining execution engines.
#[cfg(feature = "sc-executor-common")]
pub use sc_executor_common;

/// PolkaVM executor for Substrate.
#[cfg(feature = "sc-executor-polkavm")]
pub use sc_executor_polkavm;

/// Defines a `WasmRuntime` that uses the Wasmtime JIT to execute.
#[cfg(feature = "sc-executor-wasmtime")]
pub use sc_executor_wasmtime;

/// Substrate informant.
#[cfg(feature = "sc-informant")]
pub use sc_informant;

/// Keystore (and session key management) for ed25519 based chains like Polkadot.
#[cfg(feature = "sc-keystore")]
pub use sc_keystore;

/// Substrate mixnet service.
#[cfg(feature = "sc-mixnet")]
pub use sc_mixnet;

/// Substrate network protocol.
#[cfg(feature = "sc-network")]
pub use sc_network;

/// Substrate network common.
#[cfg(feature = "sc-network-common")]
pub use sc_network_common;

/// Gossiping for the Substrate network protocol.
#[cfg(feature = "sc-network-gossip")]
pub use sc_network_gossip;

/// Substrate light network protocol.
#[cfg(feature = "sc-network-light")]
pub use sc_network_light;

/// Substrate sync network protocol.
#[cfg(feature = "sc-network-sync")]
pub use sc_network_sync;

/// Substrate transaction protocol.
#[cfg(feature = "sc-network-transactions")]
pub use sc_network_transactions;

/// Substrate network types.
#[cfg(feature = "sc-network-types")]
pub use sc_network_types;

/// Substrate offchain workers.
#[cfg(feature = "sc-offchain")]
pub use sc_offchain;

/// Basic metrics for block production.
#[cfg(feature = "sc-proposer-metrics")]
pub use sc_proposer_metrics;

/// Substrate Client RPC.
#[cfg(feature = "sc-rpc")]
pub use sc_rpc;

/// Substrate RPC interfaces.
#[cfg(feature = "sc-rpc-api")]
pub use sc_rpc_api;

/// Substrate RPC servers.
#[cfg(feature = "sc-rpc-server")]
pub use sc_rpc_server;

/// Substrate RPC interface v2.
#[cfg(feature = "sc-rpc-spec-v2")]
pub use sc_rpc_spec_v2;

/// Substrate service. Starts a thread that spins up the network, client, and extrinsic pool.
/// Manages communication between them.
#[cfg(feature = "sc-service")]
pub use sc_service;

/// State database maintenance. Handles canonicalization and pruning in the database.
#[cfg(feature = "sc-state-db")]
pub use sc_state_db;

/// Storage monitor service for substrate.
#[cfg(feature = "sc-storage-monitor")]
pub use sc_storage_monitor;

/// A RPC handler to create sync states for light clients.
#[cfg(feature = "sc-sync-state-rpc")]
pub use sc_sync_state_rpc;

/// A crate that provides basic hardware and software telemetry information.
#[cfg(feature = "sc-sysinfo")]
pub use sc_sysinfo;

/// Telemetry utils.
#[cfg(feature = "sc-telemetry")]
pub use sc_telemetry;

/// Instrumentation implementation for substrate.
#[cfg(feature = "sc-tracing")]
pub use sc_tracing;

/// Helper macros for Substrate's client CLI.
#[cfg(feature = "sc-tracing-proc-macro")]
pub use sc_tracing_proc_macro;

/// Substrate transaction pool implementation.
#[cfg(feature = "sc-transaction-pool")]
pub use sc_transaction_pool;

/// Transaction pool client facing API.
#[cfg(feature = "sc-transaction-pool-api")]
pub use sc_transaction_pool_api;

/// I/O for Substrate runtimes.
#[cfg(feature = "sc-utils")]
pub use sc_utils;

/// Substrate runtime api primitives.
#[cfg(feature = "sp-api")]
pub use sp_api;

/// Macros for declaring and implementing runtime apis.
#[cfg(feature = "sp-api-proc-macro")]
pub use sp_api_proc_macro;

/// Provides facilities for generating application specific crypto wrapper types.
#[cfg(feature = "sp-application-crypto")]
pub use sp_application_crypto;

/// Minimal fixed point arithmetic primitives and types for runtime.
#[cfg(feature = "sp-arithmetic")]
pub use sp_arithmetic;

/// Authority discovery primitives.
#[cfg(feature = "sp-authority-discovery")]
pub use sp_authority_discovery;

/// The block builder runtime api.
#[cfg(feature = "sp-block-builder")]
pub use sp_block_builder;

/// Substrate blockchain traits and primitives.
#[cfg(feature = "sp-blockchain")]
pub use sp_blockchain;

/// Common utilities for building and using consensus engines in substrate.
#[cfg(feature = "sp-consensus")]
pub use sp_consensus;

/// Primitives for Aura consensus.
#[cfg(feature = "sp-consensus-aura")]
pub use sp_consensus_aura;

/// Primitives for BABE consensus.
#[cfg(feature = "sp-consensus-babe")]
pub use sp_consensus_babe;

/// Primitives for GRANDPA integration, suitable for WASM compilation.
#[cfg(feature = "sp-consensus-grandpa")]
pub use sp_consensus_grandpa;

/// Primitives for Aura consensus.
#[cfg(feature = "sp-consensus-pow")]
pub use sp_consensus_pow;

/// Primitives for slots-based consensus.
#[cfg(feature = "sp-consensus-slots")]
pub use sp_consensus_slots;

/// Shareable Substrate types.
#[cfg(feature = "sp-core")]
pub use sp_core;

/// Hashing primitives (deprecated: use sp-crypto-hashing for new applications).
#[cfg(feature = "sp-core-hashing")]
pub use sp_core_hashing;

/// Procedural macros for calculating static hashes (deprecated in favor of
/// `sp-crypto-hashing-proc-macro`).
#[cfg(feature = "sp-core-hashing-proc-macro")]
pub use sp_core_hashing_proc_macro;

/// Host functions for common Arkworks elliptic curve operations.
#[cfg(feature = "sp-crypto-ec-utils")]
pub use sp_crypto_ec_utils;

/// Hashing primitives.
#[cfg(feature = "sp-crypto-hashing")]
pub use sp_crypto_hashing;

/// Procedural macros for calculating static hashes.
#[cfg(feature = "sp-crypto-hashing-proc-macro")]
pub use sp_crypto_hashing_proc_macro;

/// Substrate database trait.
#[cfg(feature = "sp-database")]
pub use sp_database;

/// Macros to derive runtime debug implementation.
#[cfg(feature = "sp-debug-derive")]
pub use sp_debug_derive;

/// Substrate externalities abstraction.
#[cfg(feature = "sp-externalities")]
pub use sp_externalities;

/// Substrate RuntimeGenesisConfig builder API.
#[cfg(feature = "sp-genesis-builder")]
pub use sp_genesis_builder;

/// Provides types and traits for creating and checking inherents.
#[cfg(feature = "sp-inherents")]
pub use sp_inherents;

/// I/O for Substrate runtimes.
#[cfg(feature = "sp-io")]
pub use sp_io;

/// Keyring support code for the runtime. A set of test accounts.
#[cfg(feature = "sp-keyring")]
pub use sp_keyring;

/// Keystore primitives.
#[cfg(feature = "sp-keystore")]
pub use sp_keystore;

/// Handling of blobs, usually Wasm code, which may be compressed.
#[cfg(feature = "sp-maybe-compressed-blob")]
pub use sp_maybe_compressed_blob;

/// Intermediate representation of the runtime metadata.
#[cfg(feature = "sp-metadata-ir")]
pub use sp_metadata_ir;

/// Substrate mixnet types and runtime interface.
#[cfg(feature = "sp-mixnet")]
pub use sp_mixnet;

/// NPoS election algorithm primitives.
#[cfg(feature = "sp-npos-elections")]
pub use sp_npos_elections;

/// Substrate offchain workers primitives.
#[cfg(feature = "sp-offchain")]
pub use sp_offchain;

/// Custom panic hook with bug report link.
#[cfg(feature = "sp-panic-handler")]
pub use sp_panic_handler;

/// Substrate RPC primitives and utilities.
#[cfg(feature = "sp-rpc")]
pub use sp_rpc;

/// Runtime Modules shared primitive types.
#[cfg(feature = "sp-runtime")]
pub use sp_runtime;

/// Substrate runtime interface.
#[cfg(feature = "sp-runtime-interface")]
pub use sp_runtime_interface;

/// This crate provides procedural macros for usage within the context of the Substrate runtime
/// interface.
#[cfg(feature = "sp-runtime-interface-proc-macro")]
pub use sp_runtime_interface_proc_macro;

/// Primitives for sessions.
#[cfg(feature = "sp-session")]
pub use sp_session;

/// A crate which contains primitives that are useful for implementation that uses staking
/// approaches in general. Definitions related to sessions, slashing, etc go here.
#[cfg(feature = "sp-staking")]
pub use sp_staking;

/// Substrate State Machine.
#[cfg(feature = "sp-state-machine")]
pub use sp_state_machine;

/// Lowest-abstraction level for the Substrate runtime: just exports useful primitives from std
/// or client/alloc to be used with any code that depends on the runtime.
#[cfg(feature = "sp-std")]
pub use sp_std;

/// Storage related primitives.
#[cfg(feature = "sp-storage")]
pub use sp_storage;

/// Substrate core types and inherents for timestamps.
#[cfg(feature = "sp-timestamp")]
pub use sp_timestamp;

/// Instrumentation primitives and macros for Substrate.
#[cfg(feature = "sp-tracing")]
pub use sp_tracing;

/// Transaction pool runtime facing API.
#[cfg(feature = "sp-transaction-pool")]
pub use sp_transaction_pool;

/// Transaction storage proof primitives.
#[cfg(feature = "sp-transaction-storage-proof")]
pub use sp_transaction_storage_proof;

/// Patricia trie stuff using a parity-scale-codec node format.
#[cfg(feature = "sp-trie")]
pub use sp_trie;

/// Version module for the Substrate runtime; Provides a function that returns the runtime
/// version.
#[cfg(feature = "sp-version")]
pub use sp_version;

/// Macro for defining a runtime version.
#[cfg(feature = "sp-version-proc-macro")]
pub use sp_version_proc_macro;

/// Types and traits for interfacing between the host and the wasm runtime.
#[cfg(feature = "sp-wasm-interface")]
pub use sp_wasm_interface;

/// Types and traits for interfacing between the host and the wasm runtime.
#[cfg(feature = "sp-weights")]
pub use sp_weights;

/// Utility for building chain-specification files for Substrate-based runtimes based on
/// `sp-genesis-builder`.
#[cfg(feature = "staging-chain-spec-builder")]
pub use staging_chain_spec_builder;

/// Substrate node block inspection tool.
#[cfg(feature = "staging-node-inspect")]
pub use staging_node_inspect;

/// Generate and restore keys for Substrate based chains such as Polkadot, Kusama and a growing
/// number of parachains and Substrate based projects.
#[cfg(feature = "subkey")]
pub use subkey;

/// Converting BIP39 entropy to valid Substrate (sr25519) SecretKeys.
#[cfg(feature = "substrate-bip39")]
pub use substrate_bip39;

/// Crate with utility functions for `build.rs` scripts.
#[cfg(feature = "substrate-build-script-utils")]
pub use substrate_build_script_utils;

/// Substrate RPC for FRAME's support.
#[cfg(feature = "substrate-frame-rpc-support")]
pub use substrate_frame_rpc_support;

/// FRAME's system exposed over Substrate RPC.
#[cfg(feature = "substrate-frame-rpc-system")]
pub use substrate_frame_rpc_system;

/// Endpoint to expose Prometheus metrics.
#[cfg(feature = "substrate-prometheus-endpoint")]
pub use substrate_prometheus_endpoint;

/// Shared JSON-RPC client.
#[cfg(feature = "substrate-rpc-client")]
pub use substrate_rpc_client;

/// Node-specific RPC methods for interaction with state trie migration.
#[cfg(feature = "substrate-state-trie-migration-rpc")]
pub use substrate_state_trie_migration_rpc;

/// Utility for building WASM binaries.
#[cfg(feature = "substrate-wasm-builder")]
pub use substrate_wasm_builder;
