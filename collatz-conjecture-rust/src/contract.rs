#![no_main]
#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::vec;

use hyle_contract_sdk::guest;
use hyle_contract_sdk::BlobIndex;
use hyle_contract_sdk::HyleOutput;
use hyle_contract_sdk::Identity;
use hyle_contract_sdk::StateDigest;
use hyle_contract_sdk::TxHash;

#[cfg(feature = "risc0-guest")]
risc0_zkvm::guest::entry!(main);

#[cfg(feature = "sp1-guest")]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let (initial_state, suggested_number): (u32, u32) = guest::env::read();

    let next_state = if initial_state == 1 {
        match suggested_number {
            0 => None, // Cannot reset to 0 as that would block the contract,
            a => Some(a),
        }
    } else {
        // Calculate the next number in the collatz conjecture
        let mut n = initial_state;
        if n % 2 == 0 {
            n = n / 2;
        } else {
            n = 3 * n + 1;
        }
        Some(n)
    };

    guest::env::commit(&HyleOutput {
        version: 1,
        initial_state: StateDigest(initial_state.to_be_bytes().to_vec()),
        next_state: StateDigest(
            next_state
                .expect("Invalid next state")
                .to_be_bytes()
                .to_vec(),
        ),
        identity: Identity("".to_owned()),
        tx_hash: TxHash("".to_owned()),
        index: BlobIndex(0),
        blobs: vec![],
        success: true,
        program_outputs: vec![],
    });
}
