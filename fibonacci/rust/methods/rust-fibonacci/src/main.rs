#![no_main]
#![no_std]

use risc0_zkvm::guest::env;
use hyle_contract::{HyleInput, HyleOutput};

risc0_zkvm::guest::entry!(main);


pub fn main() {
  
    let input: HyleInput<u32> = env::read();
    let initial_state = u32::from_be_bytes(input.initial_state.clone().try_into().unwrap());
    let (result,success)  = fibonacci(input.program_inputs);
    env::commit(&HyleOutput {
        version: 1,
        initial_state: input.initial_state,
        next_state: result.to_be_bytes().to_vec(),
        identity: input.identity,
        tx_hash: input.tx_hash,
        payload_hash : result.to_be_bytes().to_vec(),
        success : success,
        program_outputs: "Calc done",
    
    })
}

fn fibonacci(n: u32) -> (u32,bool) {
    if n == 0 {
        return (0,true);
    } else if n == 1 {
        return (1,true);
    }

    let mut a: u32 = 0;
    let mut b: u32 = 1;

    for _ in 2..=n {
        let next = a + b;
        a = b;
        b = next;
    }

   (b,true)
}


    
