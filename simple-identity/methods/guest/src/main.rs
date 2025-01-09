use contract::Identity;
use sdk::identity_provider::IdentityAction;

use core::str::from_utf8;

fn main() {
    // Parse contract inputs
    let (input, action) = sdk::guest::init_raw::<IdentityAction>();

    // Parse initial state
    let mut state: Identity = input
        .initial_state
        .clone()
        .try_into()
        .expect("Failed to decode state");

    // Extract private information
    let password = from_utf8(&input.private_blob.0).unwrap();

    // Execute the given action
    let res = sdk::identity_provider::execute_action(&mut state, action, password);

    sdk::guest::commit(input, state, res);
}
