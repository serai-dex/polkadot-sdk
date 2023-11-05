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

//! Traits for dealing with the idea of membership.

use impl_trait_for_tuples::impl_for_tuples;
use sp_arithmetic::traits::AtLeast16BitUnsigned;
use sp_runtime::DispatchResult;
use sp_std::{marker::PhantomData, prelude::*};

/// A trait for querying whether a type can be said to "contain" a value.
pub trait Contains<T> {
	/// Return `true` if this "contains" the given value `t`.
	fn contains(t: &T) -> bool;
}

#[cfg_attr(all(not(feature = "tuples-96"), not(feature = "tuples-128")), impl_for_tuples(64))]
#[cfg_attr(all(feature = "tuples-96", not(feature = "tuples-128")), impl_for_tuples(96))]
#[cfg_attr(feature = "tuples-128", impl_for_tuples(128))]
impl<T> Contains<T> for Tuple {
	fn contains(t: &T) -> bool {
		for_tuples!( #(
			if Tuple::contains(t) { return true }
		)* );
		false
	}
}

/// A trait for querying whether a type can be said to "contain" a pair-value.
pub trait ContainsPair<A, B> {
	/// Return `true` if this "contains" the pair-value `(a, b)`.
	fn contains(a: &A, b: &B) -> bool;
}

#[cfg_attr(all(not(feature = "tuples-96"), not(feature = "tuples-128")), impl_for_tuples(64))]
#[cfg_attr(all(feature = "tuples-96", not(feature = "tuples-128")), impl_for_tuples(96))]
#[cfg_attr(feature = "tuples-128", impl_for_tuples(128))]
impl<A, B> ContainsPair<A, B> for Tuple {
	fn contains(a: &A, b: &B) -> bool {
		for_tuples!( #(
			if Tuple::contains(a, b) { return true }
		)* );
		false
	}
}

/// Converter `struct` to use a `ContainsPair` implementation for a `Contains` bound.
pub struct FromContainsPair<CP>(PhantomData<CP>);
impl<A, B, CP: ContainsPair<A, B>> Contains<(A, B)> for FromContainsPair<CP> {
	fn contains((ref a, ref b): &(A, B)) -> bool {
		CP::contains(a, b)
	}
}

/// A [`Contains`] implementation that contains every value.
pub enum Everything {}
impl<T> Contains<T> for Everything {
	fn contains(_: &T) -> bool {
		true
	}
}
impl<A, B> ContainsPair<A, B> for Everything {
	fn contains(_: &A, _: &B) -> bool {
		true
	}
}

/// A [`Contains`] implementation that contains no value.
pub enum Nothing {}
impl<T> Contains<T> for Nothing {
	fn contains(_: &T) -> bool {
		false
	}
}
impl<A, B> ContainsPair<A, B> for Nothing {
	fn contains(_: &A, _: &B) -> bool {
		false
	}
}

/// A [`Contains`] implementation that contains everything except the values in `Exclude`.
pub struct EverythingBut<Exclude>(PhantomData<Exclude>);
impl<T, Exclude: Contains<T>> Contains<T> for EverythingBut<Exclude> {
	fn contains(t: &T) -> bool {
		!Exclude::contains(t)
	}
}
impl<A, B, Exclude: ContainsPair<A, B>> ContainsPair<A, B> for EverythingBut<Exclude> {
	fn contains(a: &A, b: &B) -> bool {
		!Exclude::contains(a, b)
	}
}

/// A [`Contains`] implementation that contains all members of `These` excepting any members in
/// `Except`.
pub struct TheseExcept<These, Except>(PhantomData<(These, Except)>);
impl<T, These: Contains<T>, Except: Contains<T>> Contains<T> for TheseExcept<These, Except> {
	fn contains(t: &T) -> bool {
		These::contains(t) && !Except::contains(t)
	}
}
impl<A, B, These: ContainsPair<A, B>, Except: ContainsPair<A, B>> ContainsPair<A, B>
	for TheseExcept<These, Except>
{
	fn contains(a: &A, b: &B) -> bool {
		These::contains(a, b) && !Except::contains(a, b)
	}
}

/// A [`Contains`] implementation which contains all members of `These` which are also members of
/// `Those`.
pub struct InsideBoth<These, Those>(PhantomData<(These, Those)>);
impl<T, These: Contains<T>, Those: Contains<T>> Contains<T> for InsideBoth<These, Those> {
	fn contains(t: &T) -> bool {
		These::contains(t) && Those::contains(t)
	}
}
impl<A, B, These: ContainsPair<A, B>, Those: ContainsPair<A, B>> ContainsPair<A, B>
	for InsideBoth<These, Those>
{
	fn contains(a: &A, b: &B) -> bool {
		These::contains(a, b) && Those::contains(a, b)
	}
}

/// Create a type which implements the `Contains` trait for a particular type with syntax similar
/// to `matches!`.
#[macro_export]
macro_rules! match_types {
	(
		pub type $n:ident: impl Contains<$t:ty> = {
			$phead:pat_param $( | $ptail:pat )*
		};
		$( $rest:tt )*
	) => {
		pub struct $n;
		impl $crate::traits::Contains<$t> for $n {
			fn contains(l: &$t) -> bool {
				matches!(l, $phead $( | $ptail )* )
			}
		}
		$crate::match_types!( $( $rest )* );
	};
	(
		pub type $n:ident: impl ContainsPair<$a:ty, $b:ty> = {
			$phead:pat_param $( | $ptail:pat )*
		};
		$( $rest:tt )*
	) => {
		pub struct $n;
		impl $crate::traits::ContainsPair<$a, $b> for $n {
			fn contains(a: &$a, b: &$b) -> bool {
				matches!((a, b), $phead $( | $ptail )* )
			}
		}
		$crate::match_types!( $( $rest )* );
	};
	() => {}
}

/// Create a type which implements the `Contains` trait for a particular type with syntax similar
/// to `matches!`.
#[macro_export]
#[deprecated = "Use `match_types!` instead"]
macro_rules! match_type {
	($( $x:tt )*) => { $crate::match_types!( $( $x )* ); }
}

#[deprecated = "Use `Everything` instead"]
pub type AllowAll = Everything;
#[deprecated = "Use `Nothing` instead"]
pub type DenyAll = Nothing;
#[deprecated = "Use `Contains` instead"]
pub trait Filter<T> {
	fn filter(t: &T) -> bool;
}
#[allow(deprecated)]
impl<T, C: Contains<T>> Filter<T> for C {
	fn filter(t: &T) -> bool {
		Self::contains(t)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	match_types! {
		pub type OneOrTenToTwenty: impl Contains<u8> = { 1 | 10..=20 };
	}

	#[test]
	fn match_types_works() {
		for i in 0..=255 {
			assert_eq!(OneOrTenToTwenty::contains(&i), i == 1 || i >= 10 && i <= 20);
		}
	}
}

/// A trait for a set which can enumerate its members in order.
pub trait SortedMembers<T: Ord> {
	/// Get a vector of all members in the set, ordered.
	fn sorted_members() -> Vec<T>;

	/// Return `true` if this "contains" the given value `t`.
	fn contains(t: &T) -> bool {
		Self::sorted_members().binary_search(t).is_ok()
	}

	/// Get the number of items in the set.
	fn count() -> usize {
		Self::sorted_members().len()
	}

	/// Add an item that would satisfy `contains`. It does not make sure any other
	/// state is correctly maintained or generated.
	///
	/// **Should be used for benchmarking only!!!**
	#[cfg(feature = "runtime-benchmarks")]
	fn add(_t: &T) {
		unimplemented!()
	}
}

/// Adapter struct for turning an `OrderedMembership` impl into a `Contains` impl.
pub struct AsContains<OM>(PhantomData<(OM,)>);
impl<T: Ord + Eq, OM: SortedMembers<T>> Contains<T> for AsContains<OM> {
	fn contains(t: &T) -> bool {
		OM::contains(t)
	}
}

/// Trivial utility for implementing `Contains`/`OrderedMembership` with a `Vec`.
pub struct IsInVec<T>(PhantomData<T>);
impl<X: Eq, T: super::Get<Vec<X>>> Contains<X> for IsInVec<T> {
	fn contains(t: &X) -> bool {
		T::get().contains(t)
	}
}
impl<X: Ord + PartialOrd, T: super::Get<Vec<X>>> SortedMembers<X> for IsInVec<T> {
	fn sorted_members() -> Vec<X> {
		let mut r = T::get();
		r.sort();
		r
	}
}

/// A trait for querying bound for the length of an implementation of `Contains`
pub trait ContainsLengthBound {
	/// Minimum number of elements contained
	fn min_len() -> usize;
	/// Maximum number of elements contained
	fn max_len() -> usize;
}
