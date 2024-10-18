#![no_main]
#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::ToString;
use risc0_zkvm::guest::env;
use sdk::HyleOutput;
use utils::{ContractFunction, TokenContractInput};

risc0_zkvm::guest::entry!(main);

fn main() {
    let mut input: TokenContractInput = env::read();

    let initial_balances = input.balances.clone();

    let payload = match input.blobs.get(input.index) {
        Some(v) => v,
        None => {
            env::log("Unable to find the payload");
            let flattened_blobs = input.blobs.into_iter().flatten().collect();
            env::commit(&HyleOutput {
                version: 1,
                initial_state: initial_balances.as_state(),
                next_state: initial_balances.as_state(),
                identity: sdk::Identity("".to_string()),
                tx_hash: sdk::TxHash(input.tx_hash),
                index: sdk::BlobIndex(input.index as u32),
                blobs: flattened_blobs,
                success: false,
                program_outputs: "Payload not found".to_string().into_bytes(),
            });
            return;
        }
    };

    let contract_function = ContractFunction::decode(payload);

    let (success, identity, program_outputs) = match contract_function {
        ContractFunction::Transfer { from, to, amount } => {
            let success = match input.balances.send(&from, &to, amount) {
                Ok(()) => true,
                Err(e) => {
                    env::log(&format!("Failed to Transfer: {:?}", e));
                    false
                }
            };
            let program_outputs = format!("Transferred {} from {} to {}", amount, from, to)
                .to_string()
                .into_bytes();

            (success, from, program_outputs)
        }
        ContractFunction::Mint { to, amount } => {
            let success = match input.balances.mint(&to, amount) {
                Ok(()) => true,
                Err(e) => {
                    env::log(&format!("Failed to Mint: {:?}", e));
                    false
                }
            };
            let program_outputs = format!("Minted {} to {}", amount, to)
                .to_string()
                .into_bytes();

            (success, to, program_outputs)
        }
    };
    env::log(&format!("New balances: {:?}", input.balances));
    let next_balances = input.balances;

    let flattened_blobs = input.blobs.into_iter().flatten().collect();
    env::commit(&HyleOutput {
        version: 1,
        initial_state: initial_balances.as_state(),
        next_state: next_balances.as_state(),
        identity: sdk::Identity(identity),
        tx_hash: sdk::TxHash(input.tx_hash),
        index: sdk::BlobIndex(input.index as u32),
        blobs: flattened_blobs,
        success,
        program_outputs,
    })
}
