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

// Tests for Utility Pallet

#![cfg(test)]

use super::*;

use crate as utility;
use frame_support::{
	assert_err_ignore_postinfo, assert_noop, assert_ok, derive_impl,
	dispatch::{DispatchErrorWithPostInfo, Pays},
	error::BadOrigin,
	parameter_types,
	traits::{ConstU32, ConstU64, Contains},
	weights::Weight,
};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, Dispatchable, IdentityLookup},
	BuildStorage, DispatchError, TokenError,
};

type BlockNumber = u64;

// example module to test behaviors.
#[frame_support::pallet(dev_mode)]
pub mod example {
	use frame_support::{dispatch::WithPostDispatchInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(*_weight)]
		pub fn noop(_origin: OriginFor<T>, _weight: Weight) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(*_start_weight)]
		pub fn foobar(
			origin: OriginFor<T>,
			err: bool,
			_start_weight: Weight,
			end_weight: Option<Weight>,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			if err {
				let error: DispatchError = "The cake is a lie.".into();
				if let Some(weight) = end_weight {
					Err(error.with_weight(weight))
				} else {
					Err(error)?
				}
			} else {
				Ok(end_weight.into())
			}
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn big_variant(_origin: OriginFor<T>, _arg: [u8; 400]) -> DispatchResult {
			Ok(())
		}
	}
}

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		RootTesting: pallet_root_testing,
		Utility: utility,
		Example: example,
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(Weight::MAX);
}
#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type BaseCallFilter = TestBaseCallFilter;
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type Hash = H256;
	type RuntimeCall = RuntimeCall;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type SystemWeightInfo = frame_system::weights::SubstrateWeight<Test>;
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
}

impl pallet_root_testing::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<3>;
	type WeightInfo = ();
}

const MOTION_DURATION_IN_BLOCKS: BlockNumber = 3;
parameter_types! {
	pub const MultisigDepositBase: u64 = 1;
	pub const MultisigDepositFactor: u64 = 1;
	pub const MaxSignatories: u32 = 3;
	pub const MotionDuration: BlockNumber = MOTION_DURATION_IN_BLOCKS;
	pub const MaxProposals: u32 = 100;
	pub const MaxMembers: u32 = 100;
	pub MaxProposalWeight: Weight = sp_runtime::Perbill::from_percent(50) * BlockWeights::get().max_block;
}

impl example::Config for Test {}

pub struct TestBaseCallFilter;
impl Contains<RuntimeCall> for TestBaseCallFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match *c {
			// Transfer works. Use `transfer_keep_alive` for a call that doesn't pass the filter.
			RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death { .. }) => true,
			RuntimeCall::Utility(_) => true,
			// For benchmarking, this acts as a noop call
			RuntimeCall::System(frame_system::Call::remark { .. }) => true,
			// For tests
			RuntimeCall::Example(_) => true,
			_ => false,
		}
	}
}
impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

type ExampleCall = example::Call<Test>;
type UtilityCall = crate::Call<Test>;

use frame_system::Call as SystemCall;
use pallet_balances::Call as BalancesCall;
use pallet_root_testing::Call as RootTestingCall;
use pallet_timestamp::Call as TimestampCall;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10), (2, 10), (3, 10), (4, 10), (5, 2)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn call_transfer(dest: u64, value: u64) -> RuntimeCall {
	RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest, value })
}

fn call_foobar(err: bool, start_weight: Weight, end_weight: Option<Weight>) -> RuntimeCall {
	RuntimeCall::Example(ExampleCall::foobar { err, start_weight, end_weight })
}

#[test]
fn as_derivative_works() {
	new_test_ext().execute_with(|| {
		let sub_1_0 = Utility::derivative_account_id(1, 0);
		assert_ok!(Balances::transfer_allow_death(RuntimeOrigin::signed(1), sub_1_0, 5));
		assert_err_ignore_postinfo!(
			Utility::as_derivative(RuntimeOrigin::signed(1), 1, Box::new(call_transfer(6, 3)),),
			TokenError::FundsUnavailable,
		);
		assert_ok!(Utility::as_derivative(
			RuntimeOrigin::signed(1),
			0,
			Box::new(call_transfer(2, 3)),
		));
		assert_eq!(Balances::free_balance(sub_1_0), 2);
		assert_eq!(Balances::free_balance(2), 13);
	});
}

#[test]
fn as_derivative_handles_weight_refund() {
	new_test_ext().execute_with(|| {
		let start_weight = Weight::from_parts(100, 0);
		let end_weight = Weight::from_parts(75, 0);
		let diff = start_weight - end_weight;

		// Full weight when ok
		let inner_call = call_foobar(false, start_weight, None);
		let call = RuntimeCall::Utility(UtilityCall::as_derivative {
			index: 0,
			call: Box::new(inner_call),
		});
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_ok!(result);
		assert_eq!(extract_actual_weight(&result, &info), info.weight);

		// Refund weight when ok
		let inner_call = call_foobar(false, start_weight, Some(end_weight));
		let call = RuntimeCall::Utility(UtilityCall::as_derivative {
			index: 0,
			call: Box::new(inner_call),
		});
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_ok!(result);
		// Diff is refunded
		assert_eq!(extract_actual_weight(&result, &info), info.weight - diff);

		// Full weight when err
		let inner_call = call_foobar(true, start_weight, None);
		let call = RuntimeCall::Utility(UtilityCall::as_derivative {
			index: 0,
			call: Box::new(inner_call),
		});
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_noop!(
			result,
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					// No weight is refunded
					actual_weight: Some(info.weight),
					pays_fee: Pays::Yes,
				},
				error: DispatchError::Other("The cake is a lie."),
			}
		);

		// Refund weight when err
		let inner_call = call_foobar(true, start_weight, Some(end_weight));
		let call = RuntimeCall::Utility(UtilityCall::as_derivative {
			index: 0,
			call: Box::new(inner_call),
		});
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_noop!(
			result,
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					// Diff is refunded
					actual_weight: Some(info.weight - diff),
					pays_fee: Pays::Yes,
				},
				error: DispatchError::Other("The cake is a lie."),
			}
		);
	});
}

#[test]
fn as_derivative_filters() {
	new_test_ext().execute_with(|| {
		assert_err_ignore_postinfo!(
			Utility::as_derivative(
				RuntimeOrigin::signed(1),
				1,
				Box::new(RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive {
					dest: 2,
					value: 1
				})),
			),
			DispatchError::from(frame_system::Error::<Test>::CallFiltered),
		);
	});
}

#[test]
fn batch_with_root_works() {
	new_test_ext().execute_with(|| {
		let call =
			RuntimeCall::Balances(BalancesCall::force_transfer { source: 1, dest: 2, value: 5 });
		assert!(!TestBaseCallFilter::contains(&call));
		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
		assert_ok!(Utility::batch(
			RuntimeOrigin::root(),
			vec![
				// These calls aren't in the filter, checking the filter was correctly bypassed
				call,
				RuntimeCall::Balances(BalancesCall::force_transfer {
					source: 1,
					dest: 2,
					value: 5
				}),
			]
		));
		assert_eq!(Balances::free_balance(1), 0);
		assert_eq!(Balances::free_balance(2), 20);
	});
}

#[test]
fn batch_with_signed_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
		assert_ok!(Utility::batch(
			RuntimeOrigin::signed(1),
			vec![call_transfer(2, 5), call_transfer(2, 5)]
		),);
		assert_eq!(Balances::free_balance(1), 0);
		assert_eq!(Balances::free_balance(2), 20);
	});
}

#[test]
fn batch_with_signed_filters() {
	new_test_ext().execute_with(|| {
		assert_ok!(Utility::batch(
			RuntimeOrigin::signed(1),
			vec![RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive {
				dest: 2,
				value: 1
			})]
		),);
		System::assert_last_event(
			utility::Event::BatchInterrupted {
				index: 0,
				error: frame_system::Error::<Test>::CallFiltered.into(),
			}
			.into(),
		);
	});
}

#[test]
fn batch_early_exit_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
		assert_ok!(Utility::batch(
			RuntimeOrigin::signed(1),
			vec![call_transfer(2, 5), call_transfer(2, 10), call_transfer(2, 5),]
		),);
		assert_eq!(Balances::free_balance(1), 5);
		assert_eq!(Balances::free_balance(2), 15);
	});
}

#[test]
fn batch_weight_calculation_doesnt_overflow() {
	use sp_runtime::Perbill;
	new_test_ext().execute_with(|| {
		let big_call = RuntimeCall::RootTesting(RootTestingCall::fill_block {
			ratio: Perbill::from_percent(50),
		});
		assert_eq!(big_call.get_dispatch_info().weight, Weight::MAX / 2);

		// 3 * 50% saturates to 100%
		let batch_call = RuntimeCall::Utility(crate::Call::batch {
			calls: vec![big_call.clone(), big_call.clone(), big_call.clone()],
		});

		assert_eq!(batch_call.get_dispatch_info().weight, Weight::MAX);
	});
}

#[test]
fn batch_handles_weight_refund() {
	new_test_ext().execute_with(|| {
		let start_weight = Weight::from_parts(100, 0);
		let end_weight = Weight::from_parts(75, 0);
		let diff = start_weight - end_weight;
		let batch_len = 4;

		// Full weight when ok
		let inner_call = call_foobar(false, start_weight, None);
		let batch_calls = vec![inner_call; batch_len as usize];
		let call = RuntimeCall::Utility(UtilityCall::batch { calls: batch_calls });
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_ok!(result);
		assert_eq!(extract_actual_weight(&result, &info), info.weight);

		// Refund weight when ok
		let inner_call = call_foobar(false, start_weight, Some(end_weight));
		let batch_calls = vec![inner_call; batch_len as usize];
		let call = RuntimeCall::Utility(UtilityCall::batch { calls: batch_calls });
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_ok!(result);
		// Diff is refunded
		assert_eq!(extract_actual_weight(&result, &info), info.weight - diff * batch_len);

		// Full weight when err
		let good_call = call_foobar(false, start_weight, None);
		let bad_call = call_foobar(true, start_weight, None);
		let batch_calls = vec![good_call, bad_call];
		let call = RuntimeCall::Utility(UtilityCall::batch { calls: batch_calls });
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_ok!(result);
		System::assert_last_event(
			utility::Event::BatchInterrupted { index: 1, error: DispatchError::Other("") }.into(),
		);
		// No weight is refunded
		assert_eq!(extract_actual_weight(&result, &info), info.weight);

		// Refund weight when err
		let good_call = call_foobar(false, start_weight, Some(end_weight));
		let bad_call = call_foobar(true, start_weight, Some(end_weight));
		let batch_calls = vec![good_call, bad_call];
		let batch_len = batch_calls.len() as u64;
		let call = RuntimeCall::Utility(UtilityCall::batch { calls: batch_calls });
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_ok!(result);
		System::assert_last_event(
			utility::Event::BatchInterrupted { index: 1, error: DispatchError::Other("") }.into(),
		);
		assert_eq!(extract_actual_weight(&result, &info), info.weight - diff * batch_len);

		// Partial batch completion
		let good_call = call_foobar(false, start_weight, Some(end_weight));
		let bad_call = call_foobar(true, start_weight, Some(end_weight));
		let batch_calls = vec![good_call, bad_call.clone(), bad_call];
		let call = RuntimeCall::Utility(UtilityCall::batch { calls: batch_calls });
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_ok!(result);
		System::assert_last_event(
			utility::Event::BatchInterrupted { index: 1, error: DispatchError::Other("") }.into(),
		);
		assert_eq!(
			extract_actual_weight(&result, &info),
			// Real weight is 2 calls at end_weight
			<Test as Config>::WeightInfo::batch(2) + end_weight * 2,
		);
	});
}

#[test]
fn batch_all_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
		assert_ok!(Utility::batch_all(
			RuntimeOrigin::signed(1),
			vec![call_transfer(2, 5), call_transfer(2, 5)]
		),);
		assert_eq!(Balances::free_balance(1), 0);
		assert_eq!(Balances::free_balance(2), 20);
	});
}

#[test]
fn batch_all_revert() {
	new_test_ext().execute_with(|| {
		let call = call_transfer(2, 5);
		let info = call.get_dispatch_info();

		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
		let batch_all_calls = RuntimeCall::Utility(crate::Call::<Test>::batch_all {
			calls: vec![call_transfer(2, 5), call_transfer(2, 10), call_transfer(2, 5)],
		});
		assert_noop!(
			batch_all_calls.dispatch(RuntimeOrigin::signed(1)),
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(
						<Test as Config>::WeightInfo::batch_all(2) + info.weight * 2
					),
					pays_fee: Pays::Yes
				},
				error: TokenError::FundsUnavailable.into(),
			}
		);
		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
	});
}

#[test]
fn batch_all_handles_weight_refund() {
	new_test_ext().execute_with(|| {
		let start_weight = Weight::from_parts(100, 0);
		let end_weight = Weight::from_parts(75, 0);
		let diff = start_weight - end_weight;
		let batch_len = 4;

		// Full weight when ok
		let inner_call = call_foobar(false, start_weight, None);
		let batch_calls = vec![inner_call; batch_len as usize];
		let call = RuntimeCall::Utility(UtilityCall::batch_all { calls: batch_calls });
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_ok!(result);
		assert_eq!(extract_actual_weight(&result, &info), info.weight);

		// Refund weight when ok
		let inner_call = call_foobar(false, start_weight, Some(end_weight));
		let batch_calls = vec![inner_call; batch_len as usize];
		let call = RuntimeCall::Utility(UtilityCall::batch_all { calls: batch_calls });
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_ok!(result);
		// Diff is refunded
		assert_eq!(extract_actual_weight(&result, &info), info.weight - diff * batch_len);

		// Full weight when err
		let good_call = call_foobar(false, start_weight, None);
		let bad_call = call_foobar(true, start_weight, None);
		let batch_calls = vec![good_call, bad_call];
		let call = RuntimeCall::Utility(UtilityCall::batch_all { calls: batch_calls });
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_err_ignore_postinfo!(result, "The cake is a lie.");
		// No weight is refunded
		assert_eq!(extract_actual_weight(&result, &info), info.weight);

		// Refund weight when err
		let good_call = call_foobar(false, start_weight, Some(end_weight));
		let bad_call = call_foobar(true, start_weight, Some(end_weight));
		let batch_calls = vec![good_call, bad_call];
		let batch_len = batch_calls.len() as u64;
		let call = RuntimeCall::Utility(UtilityCall::batch_all { calls: batch_calls });
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_err_ignore_postinfo!(result, "The cake is a lie.");
		assert_eq!(extract_actual_weight(&result, &info), info.weight - diff * batch_len);

		// Partial batch completion
		let good_call = call_foobar(false, start_weight, Some(end_weight));
		let bad_call = call_foobar(true, start_weight, Some(end_weight));
		let batch_calls = vec![good_call, bad_call.clone(), bad_call];
		let call = RuntimeCall::Utility(UtilityCall::batch_all { calls: batch_calls });
		let info = call.get_dispatch_info();
		let result = call.dispatch(RuntimeOrigin::signed(1));
		assert_err_ignore_postinfo!(result, "The cake is a lie.");
		assert_eq!(
			extract_actual_weight(&result, &info),
			// Real weight is 2 calls at end_weight
			<Test as Config>::WeightInfo::batch_all(2) + end_weight * 2,
		);
	});
}

#[test]
fn batch_all_does_not_nest() {
	new_test_ext().execute_with(|| {
		let batch_all = RuntimeCall::Utility(UtilityCall::batch_all {
			calls: vec![call_transfer(2, 1), call_transfer(2, 1), call_transfer(2, 1)],
		});

		let info = batch_all.get_dispatch_info();

		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
		// A nested batch_all call will not pass the filter, and fail with `BadOrigin`.
		assert_noop!(
			Utility::batch_all(RuntimeOrigin::signed(1), vec![batch_all.clone()]),
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(<Test as Config>::WeightInfo::batch_all(1) + info.weight),
					pays_fee: Pays::Yes
				},
				error: frame_system::Error::<Test>::CallFiltered.into(),
			}
		);

		// And for those who want to get a little fancy, we check that the filter persists across
		// other kinds of dispatch wrapping functions... in this case
		// `batch_all(batch(batch_all(..)))`
		let batch_nested = RuntimeCall::Utility(UtilityCall::batch { calls: vec![batch_all] });
		// Batch will end with `Ok`, but does not actually execute as we can see from the event
		// and balances.
		assert_ok!(Utility::batch_all(RuntimeOrigin::signed(1), vec![batch_nested]));
		System::assert_has_event(
			utility::Event::BatchInterrupted {
				index: 0,
				error: frame_system::Error::<Test>::CallFiltered.into(),
			}
			.into(),
		);
		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
	});
}

#[test]
fn batch_limit() {
	new_test_ext().execute_with(|| {
		let calls = vec![RuntimeCall::System(SystemCall::remark { remark: vec![] }); 40_000];
		assert_noop!(
			Utility::batch(RuntimeOrigin::signed(1), calls.clone()),
			Error::<Test>::TooManyCalls
		);
		assert_noop!(
			Utility::batch_all(RuntimeOrigin::signed(1), calls),
			Error::<Test>::TooManyCalls
		);
	});
}

#[test]
fn force_batch_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
		assert_ok!(Utility::force_batch(
			RuntimeOrigin::signed(1),
			vec![
				call_transfer(2, 5),
				call_foobar(true, Weight::from_parts(75, 0), None),
				call_transfer(2, 10),
				call_transfer(2, 5),
			]
		));
		System::assert_last_event(utility::Event::BatchCompletedWithErrors.into());
		System::assert_has_event(
			utility::Event::ItemFailed { error: DispatchError::Other("") }.into(),
		);
		assert_eq!(Balances::free_balance(1), 0);
		assert_eq!(Balances::free_balance(2), 20);

		assert_ok!(Utility::force_batch(
			RuntimeOrigin::signed(2),
			vec![call_transfer(1, 5), call_transfer(1, 5),]
		));
		System::assert_last_event(utility::Event::BatchCompleted.into());

		assert_ok!(Utility::force_batch(RuntimeOrigin::signed(1), vec![call_transfer(2, 50),]),);
		System::assert_last_event(utility::Event::BatchCompletedWithErrors.into());
	});
}

#[test]
fn none_origin_does_not_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(Utility::force_batch(RuntimeOrigin::none(), vec![]), BadOrigin);
		assert_noop!(Utility::batch(RuntimeOrigin::none(), vec![]), BadOrigin);
		assert_noop!(Utility::batch_all(RuntimeOrigin::none(), vec![]), BadOrigin);
	})
}

#[test]
fn batch_doesnt_work_with_inherents() {
	new_test_ext().execute_with(|| {
		// fails because inherents expect the origin to be none.
		assert_ok!(Utility::batch(
			RuntimeOrigin::signed(1),
			vec![RuntimeCall::Timestamp(TimestampCall::set { now: 42 }),]
		));
		System::assert_last_event(
			utility::Event::BatchInterrupted {
				index: 0,
				error: frame_system::Error::<Test>::CallFiltered.into(),
			}
			.into(),
		);
	})
}

#[test]
fn force_batch_doesnt_work_with_inherents() {
	new_test_ext().execute_with(|| {
		// fails because inherents expect the origin to be none.
		assert_ok!(Utility::force_batch(
			RuntimeOrigin::root(),
			vec![RuntimeCall::Timestamp(TimestampCall::set { now: 42 }),]
		));
		System::assert_last_event(utility::Event::BatchCompletedWithErrors.into());
	})
}

#[test]
fn batch_all_doesnt_work_with_inherents() {
	new_test_ext().execute_with(|| {
		let batch_all = RuntimeCall::Utility(UtilityCall::batch_all {
			calls: vec![RuntimeCall::Timestamp(TimestampCall::set { now: 42 })],
		});
		let info = batch_all.get_dispatch_info();

		// fails because inherents expect the origin to be none.
		assert_noop!(
			batch_all.dispatch(RuntimeOrigin::signed(1)),
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(info.weight),
					pays_fee: Pays::Yes
				},
				error: frame_system::Error::<Test>::CallFiltered.into(),
			}
		);
	})
}

#[test]
fn with_weight_works() {
	new_test_ext().execute_with(|| {
		use frame_system::WeightInfo;
		let remark_call = Box::new(RuntimeCall::System(frame_system::Call::remark {
			remark: b"remark".to_vec(),
		}));
		// Weight before is max.
		assert_eq!(
			remark_call.get_dispatch_info().weight,
			<Test as frame_system::Config>::SystemWeightInfo::remark(6)
		);
		assert_eq!(
			remark_call.get_dispatch_info().class,
			frame_support::dispatch::DispatchClass::Normal
		);

		let with_weight_call =
			Call::<Test>::with_weight { call: remark_call, weight: Weight::from_parts(123, 456) };
		// Weight after is set by Root.
		assert_eq!(with_weight_call.get_dispatch_info().weight, Weight::from_parts(123, 456));
		assert_eq!(
			with_weight_call.get_dispatch_info().class,
			frame_support::dispatch::DispatchClass::Normal
		);
	})
}
