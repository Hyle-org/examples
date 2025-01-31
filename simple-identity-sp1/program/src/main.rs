#![no_main]

extern crate alloc;

use sdk::guest::GuestEnv;
use sdk::guest::SP1Env;
use contract::execute;

sp1_zkvm::entrypoint!(main);

fn main() {
    let env = SP1Env {};
    env.commit(&execute(env.read()));
}