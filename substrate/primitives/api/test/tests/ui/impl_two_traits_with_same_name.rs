// This file is part of a fork of Substrate which has had various changes.

// Copyright (C) Parity Technologies (UK) Ltd.
// Copyright (C) 2022-2023 Luke Parker
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// The declaration of the `Runtime` type  is done by the `construct_runtime!` macro in a real
/// runtime.
struct Runtime {}

sp_api::decl_runtime_apis! {
	pub trait Api {
		fn test(data: u64);
	}
}

#[allow(unused_imports)]
mod second {
	sp_api::decl_runtime_apis! {
		pub trait Api {
			fn test2(data: u64);
		}
	}
}

sp_api::impl_runtime_apis! {
	impl self::Api<Block> for Runtime {
		fn test(data: u64) {}
	}

	impl second::Api<Block> for Runtime {
		fn test2(data: u64) {}
	}
}

fn main() {}
