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

//! Utilities related to VRF input, pre-output and signatures.

use crate::{Randomness, TicketBody, TicketId};
use scale_codec::Encode;
use sp_consensus_slots::Slot;
use sp_std::vec::Vec;

pub use sp_core::bandersnatch::{
	ring_vrf::{RingProver, RingVerifier, RingVerifierData, RingVrfSignature},
	vrf::{VrfInput, VrfPreOutput, VrfSignData, VrfSignature},
};

/// Ring VRF domain size for Sassafras consensus.
pub const RING_VRF_DOMAIN_SIZE: u32 = 2048;

/// Bandersnatch VRF [`RingContext`] specialization for Sassafras using [`RING_VRF_DOMAIN_SIZE`].
pub type RingContext = sp_core::bandersnatch::ring_vrf::RingContext<RING_VRF_DOMAIN_SIZE>;

fn vrf_input_from_data(
	domain: &[u8],
	data: impl IntoIterator<Item = impl AsRef<[u8]>>,
) -> VrfInput {
	let buf = data.into_iter().fold(Vec::new(), |mut buf, item| {
		let bytes = item.as_ref();
		buf.extend_from_slice(bytes);
		let len = u8::try_from(bytes.len()).expect("private function with well known inputs");
		buf.push(len);
		buf
	});
	VrfInput::new(domain, buf)
}

/// VRF input to claim slot ownership during block production.
pub fn slot_claim_input(randomness: &Randomness, slot: Slot, epoch: u64) -> VrfInput {
	vrf_input_from_data(
		b"sassafras-claim-v1.0",
		[randomness.as_slice(), &slot.to_le_bytes(), &epoch.to_le_bytes()],
	)
}

/// Signing-data to claim slot ownership during block production.
pub fn slot_claim_sign_data(randomness: &Randomness, slot: Slot, epoch: u64) -> VrfSignData {
	let input = slot_claim_input(randomness, slot, epoch);
	VrfSignData::new_unchecked(
		b"sassafras-slot-claim-transcript-v1.0",
		Option::<&[u8]>::None,
		Some(input),
	)
}

/// VRF input to generate the ticket id.
pub fn ticket_id_input(randomness: &Randomness, attempt: u32, epoch: u64) -> VrfInput {
	vrf_input_from_data(
		b"sassafras-ticket-v1.0",
		[randomness.as_slice(), &attempt.to_le_bytes(), &epoch.to_le_bytes()],
	)
}

/// VRF input to generate the revealed key.
pub fn revealed_key_input(randomness: &Randomness, attempt: u32, epoch: u64) -> VrfInput {
	vrf_input_from_data(
		b"sassafras-revealed-v1.0",
		[randomness.as_slice(), &attempt.to_le_bytes(), &epoch.to_le_bytes()],
	)
}

/// Data to be signed via ring-vrf.
pub fn ticket_body_sign_data(ticket_body: &TicketBody, ticket_id_input: VrfInput) -> VrfSignData {
	VrfSignData::new_unchecked(
		b"sassafras-ticket-body-transcript-v1.0",
		Some(ticket_body.encode().as_slice()),
		Some(ticket_id_input),
	)
}

/// Make ticket-id from the given VRF input and pre-output.
///
/// Input should have been obtained via [`ticket_id_input`].
/// Pre-output should have been obtained from the input directly using the vrf
/// secret key or from the vrf signature pre-outputs.
pub fn make_ticket_id(input: &VrfInput, pre_output: &VrfPreOutput) -> TicketId {
	let bytes = pre_output.make_bytes::<16>(b"ticket-id", input);
	u128::from_le_bytes(bytes)
}

/// Make revealed key seed from a given VRF input and pre-ouput.
///
/// Input should have been obtained via [`revealed_key_input`].
/// Pre-output should have been obtained from the input directly using the vrf
/// secret key or from the vrf signature pre-outputs.
pub fn make_revealed_key_seed(input: &VrfInput, pre_output: &VrfPreOutput) -> [u8; 32] {
	pre_output.make_bytes::<32>(b"revealed-seed", input)
}
