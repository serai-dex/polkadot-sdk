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

// Benchmarks for Utility Pallet

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{
	v1::{benchmarks, whitelisted_caller},
	BenchmarkError,
};
use frame_support::{dispatch::DispatchClass, traits::Get};
use frame_system::{Call, Pallet as System, RawOrigin};
use sp_std::{prelude::*, vec};

mod mock;

pub struct Pallet<T: Config>(System<T>);
pub trait Config: frame_system::Config {
	/// Adds ability to the Runtime to test against their sample code.
	///
	/// Default is `../res/kitchensink_runtime.compact.compressed.wasm`.
	fn prepare_set_code_data() -> Vec<u8> {
		include_bytes!("../res/kitchensink_runtime.compact.compressed.wasm").to_vec()
	}

	/// Adds ability to the Runtime to prepare/initialize before running benchmark `set_code`.
	fn setup_set_code_requirements(_code: &Vec<u8>) -> Result<(), BenchmarkError> {
		Ok(())
	}

	/// Adds ability to the Runtime to do custom validation after benchmark.
	///
	/// Default is checking for `CodeUpdated` event .
	fn verify_set_code() {
		System::<Self>::assert_last_event(frame_system::Event::<Self>::CodeUpdated.into());
	}
}

benchmarks! {
	remark {
		let b in 0 .. *T::BlockLength::get().max.get(DispatchClass::Normal) as u32;
		let remark_message = vec![1; b as usize];
		let caller = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), remark_message)

	set_code {
		let runtime_blob = T::prepare_set_code_data();
		T::setup_set_code_requirements(&runtime_blob)?;
	}: _(RawOrigin::Root, runtime_blob)
	verify {
		T::verify_set_code()
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
