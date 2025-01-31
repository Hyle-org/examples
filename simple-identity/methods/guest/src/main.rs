#![no_main]
#![no_std]

extern crate alloc;

use sdk::guest::commit;
use sdk::guest::GuestEnv;
use sdk::guest::Risc0Env;

use contract_identity::execute;
use sdk::ContractInput;

risc0_zkvm::guest::entry!(main);

fn main() {
    let env = Risc0Env {};
    let input: ContractInput = env.read();
    commit(env, input.clone(), execute(input));
}
