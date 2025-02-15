// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
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

use codec::Encode;
use kitchensink_runtime::{
	constants::time::SLOT_DURATION,
	CheckedExtrinsic, Multiplier, RuntimeCall,
	TransactionPayment,
};
use node_testing::keyring::*;
use polkadot_sdk::*;
use sp_runtime::{traits::One, Perbill};

pub mod common;
use self::common::*;

#[test]
fn fee_multiplier_increases_and_decreases_on_big_weight() {
	let mut t = new_test_ext(compact_code_unwrap());

	// initial fee multiplier must be one.
	let mut prev_multiplier = Multiplier::one();

	t.execute_with(|| {
		assert_eq!(TransactionPayment::next_fee_multiplier(), prev_multiplier);
	});

	let mut tt = new_test_ext(compact_code_unwrap());

	let time1 = 42 * 1000;
	// big one in terms of weight.
	let block1 = construct_block(
		&mut tt,
		1,
		GENESIS_HASH.into(),
		vec![
			CheckedExtrinsic {
				format: sp_runtime::generic::ExtrinsicFormat::Bare,
				function: RuntimeCall::Timestamp(pallet_timestamp::Call::set { now: time1 }),
			},
			CheckedExtrinsic {
				format: sp_runtime::generic::ExtrinsicFormat::Signed(charlie(), tx_ext(0)),
				function: RuntimeCall::Sudo(pallet_sudo::Call::sudo {
					call: Box::new(RuntimeCall::RootTesting(
						pallet_root_testing::Call::fill_block { ratio: Perbill::from_percent(60) },
					)),
				}),
			},
		],
		(time1 / SLOT_DURATION).into(),
	);

	let time2 = 52 * 1000;
	// small one in terms of weight.
	let block2 = construct_block(
		&mut tt,
		2,
		block1.1,
		vec![
			CheckedExtrinsic {
				format: sp_runtime::generic::ExtrinsicFormat::Bare,
				function: RuntimeCall::Timestamp(pallet_timestamp::Call::set { now: time2 }),
			},
			CheckedExtrinsic {
				format: sp_runtime::generic::ExtrinsicFormat::Signed(charlie(), tx_ext(1)),
				function: RuntimeCall::System(frame_system::Call::remark { remark: vec![0; 1] }),
			},
		],
		(time2 / SLOT_DURATION).into(),
	);

	println!(
		"++ Block 1 size: {} / Block 2 size {}",
		block1.0.encode().len(),
		block2.0.encode().len(),
	);

	// execute a big block.
	executor_call(&mut t, "Core_execute_block", &block1.0).0.unwrap();

	// weight multiplier is increased for next block.
	t.execute_with(|| {
		let fm = TransactionPayment::next_fee_multiplier();
		println!("After a big block: {:?} -> {:?}", prev_multiplier, fm);
		assert!(fm > prev_multiplier);
		prev_multiplier = fm;
	});

	// execute a big block.
	executor_call(&mut t, "Core_execute_block", &block2.0).0.unwrap();

	// weight multiplier is increased for next block.
	t.execute_with(|| {
		let fm = TransactionPayment::next_fee_multiplier();
		println!("After a small block: {:?} -> {:?}", prev_multiplier, fm);
		assert!(fm < prev_multiplier);
	});
}
