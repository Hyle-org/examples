#![no_std]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ERC20Input<T> {
    pub initial_state: Vec<u8>,
    pub tx_hash: Vec<u8>,
    pub sender: String,
    pub receiver: String,
    pub program_inputs: T,
}
