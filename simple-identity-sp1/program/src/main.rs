#![no_main]

extern crate alloc;

use contract::IdentityContractState;
use sdk::guest::execute;
use sdk::guest::GuestEnv;
use sdk::guest::SP1Env;

sp1_zkvm::entrypoint!(main);

fn main() {
    let env = SP1Env {};
    let input = env.read();
    let (_, output) = execute::<IdentityContractState>(&input);
    env.commit(&output);
}
