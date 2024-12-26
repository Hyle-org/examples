use contract::{Token, TokenContract};
use sdk::erc20::ERC20Action;

fn main() {
    // Parse contract inputs
    let (input, action) = sdk::guest::init_raw::<Token, ERC20Action>();

    // Parse initial state as Token
    let state: Token = input.initial_state.clone();

    // Execute the given action
    let mut contract = TokenContract::init(state, input.identity.clone());
    let execution_result = sdk::erc20::execute_action(&mut contract, action);
    let new_state = contract.state();

    // Commit the result
    sdk::guest::commit(input, new_state, execution_result);
}
