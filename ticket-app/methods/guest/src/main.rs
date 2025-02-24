#![no_main]
#![no_std]

extern crate alloc;

use contract_ticket_app::TicketAppState;
use sdk::guest::execute;
use sdk::guest::GuestEnv;
use sdk::guest::Risc0Env;

risc0_zkvm::guest::entry!(main);

fn main() {
    let env = Risc0Env {};
    let input = env.read();
    let (_, output) = execute::<TicketAppState>(&input);
    env.commit(&output);
}
