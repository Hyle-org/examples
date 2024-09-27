#![no_main]
#![no_std]

extern crate alloc;
use alloc::format;
use alloc::string::String;

use risc0_zkvm::guest::env;
use hyle_contract::HyleOutput;


risc0_zkvm::guest::entry!(main);

#[derive(Debug, Clone)]
struct Account {
    name: String,
    balance: u64,
}


use utils::ERC20Input;

impl Account {
    fn new(name: String, balance: u64) -> Self {
        Account { name, balance }
    }

    fn transfer(&mut self, amount: u64) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            true
        } else {
            false
        }
    }

    fn receive(&mut self, amount: u64) {
        self.balance += amount;
    }
}

pub fn main() {
    let input: ERC20Input<u64> = env::read();

    let mut sender = Account::new(input.sender.clone(), 1000); // Initial balances for the example
    let mut receiver = Account::new(input.receiver.clone(), 500); 

    let success = if sender.transfer(input.program_inputs) {
        receiver.receive(input.program_inputs);
        true
    } else {
        false
    };

    let initial_state = u32::from_be_bytes(input.initial_state.clone().try_into().unwrap());
    env::commit(&HyleOutput {
        version: 1,
        identity: input.sender,
        tx_hash: input.tx_hash,
        program_outputs: format!(
            "Transferred {} from {} to {}. Sender new balance: {}, Receiver new balance: {}",
            input.program_inputs, sender.name, receiver.name, sender.balance, receiver.balance
        ),
        initial_state: input.initial_state,
        next_state: u32::to_be_bytes(if success { initial_state + 1 } else { initial_state }).to_vec(),
    })
}
