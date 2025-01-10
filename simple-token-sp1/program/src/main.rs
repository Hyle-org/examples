#![no_main]
sp1_zkvm::entrypoint!(main);

extern crate alloc;

use contract::{Token, TokenContract};
use sdk::erc20::ERC20Action;

fn main() {
    // Parse contract inputs
    let (input, parsed_blob, caller) = match sdk::guest::init_with_caller::<ERC20Action>() {
        Ok(res) => res,
        Err(err) => {
            panic!("Hyllar contract initialization failed {}", err);
        }
    };

    // Parse initial state as Token
    let state: Token = input.initial_state.clone().into();

    // Execute the given action
    let mut contract = TokenContract::init(state, caller);
    let execution_result = sdk::erc20::execute_action(&mut contract, parsed_blob.data.parameters);
    let new_state = contract.state();

    sdk::guest::env::log(alloc::format!("commit {:?}", execution_result).as_str());

    // Commit the result
    sdk::guest::commit(input, new_state, execution_result);
}
