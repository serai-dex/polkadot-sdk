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
	arg_enums::{
		WasmExecutionMethod, WasmtimeInstantiationStrategy,
		DEFAULT_WASMTIME_INSTANTIATION_STRATEGY, DEFAULT_WASM_EXECUTION_METHOD,
	},
	params::{DatabaseParams, PruningParams},
};
use clap::Args;
use std::path::PathBuf;

/// Parameters for block import.
#[derive(Debug, Clone, Args)]
pub struct ImportParams {
	#[allow(missing_docs)]
	#[clap(flatten)]
	pub pruning_params: PruningParams,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub database_params: DatabaseParams,

	/// Method for executing Wasm runtime code.
	#[arg(
		long = "wasm-execution",
		value_name = "METHOD",
		value_enum,
		ignore_case = true,
		default_value_t = DEFAULT_WASM_EXECUTION_METHOD,
	)]
	pub wasm_method: WasmExecutionMethod,

	/// The WASM instantiation method to use.
	///
	/// Only has an effect when `wasm-execution` is set to `compiled`.
	/// The copy-on-write strategies are only supported on Linux.
	/// If the copy-on-write variant of a strategy is unsupported
	/// the executor will fall back to the non-CoW equivalent.
	/// The fastest (and the default) strategy available is `pooling-copy-on-write`.
	/// The `legacy-instance-reuse` strategy is deprecated and will
	/// be removed in the future. It should only be used in case of
	/// issues with the default instantiation strategy.
	#[arg(
		long,
		value_name = "STRATEGY",
		default_value_t = DEFAULT_WASMTIME_INSTANTIATION_STRATEGY,
		value_enum,
	)]
	pub wasmtime_instantiation_strategy: WasmtimeInstantiationStrategy,

	/// Specify the path where local WASM runtimes are stored.
	///
	/// These runtimes will override on-chain runtimes when the version matches.
	#[arg(long, value_name = "PATH")]
	pub wasm_runtime_overrides: Option<PathBuf>,

	/// Specify the state cache size.
	///
	/// Providing `0` will disable the cache.
	#[arg(long, value_name = "Bytes", default_value_t = 67108864)]
	pub trie_cache_size: usize,

	/// DEPRECATED: switch to `--trie-cache-size`.
	#[arg(long)]
	state_cache_size: Option<usize>,
}

impl ImportParams {
	/// Specify the trie cache maximum size.
	pub fn trie_cache_maximum_size(&self) -> Option<usize> {
		if self.state_cache_size.is_some() {
			eprintln!("`--state-cache-size` was deprecated. Please switch to `--trie-cache-size`.");
		}

		if self.trie_cache_size == 0 {
			None
		} else {
			Some(self.trie_cache_size)
		}
	}

	/// Get the WASM execution method from the parameters
	pub fn wasm_method(&self) -> sc_service::config::WasmExecutionMethod {
		crate::execution_method_from_cli(self.wasmtime_instantiation_strategy)
	}

	/// Enable overriding on-chain WASM with locally-stored WASM
	/// by specifying the path where local WASM is stored.
	pub fn wasm_runtime_overrides(&self) -> Option<PathBuf> {
		self.wasm_runtime_overrides.clone()
	}
}
