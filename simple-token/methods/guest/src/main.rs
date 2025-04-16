#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use contract::SimpleToken;
use sdk::guest::execute;
use sdk::guest::GuestEnv;
use sdk::guest::Risc0Env;
use sdk::Calldata;

risc0_zkvm::guest::entry!(main);

fn main() {
    let env = Risc0Env {};
    let (commitment_metadata, calldata): (Vec<u8>, Calldata) = env.read();

    let output = execute::<SimpleToken>(&commitment_metadata, &calldata);
    env.commit(&output);
}
