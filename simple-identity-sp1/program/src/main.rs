#![no_main]

extern crate alloc;

use sdk::guest::commit;
use sdk::guest::GuestEnv;
use sdk::guest::SP1Env;

use contract::execute;
use sdk::ContractInput;

sp1_zkvm::entrypoint!(main);

fn main() {
    let env = SP1Env {};
    let input: ContractInput = env.read();
    commit(env, input.clone(), execute(input));
}

