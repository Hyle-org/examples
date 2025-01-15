#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use contract_ticket_app::{TicketAppAction, TicketAppContract};
use sdk::erc20::ERC20Action;
use sdk::BlobIndex;

risc0_zkvm::guest::entry!(main);

fn main() {
    let (input, ticket_app_action) = sdk::guest::init_raw::<TicketAppAction>();
    let ticket_app_contract_name = input
        .blobs
        .get(input.index.0)
        .unwrap()
        .contract_name
        .clone();

    let transfer_action =
        sdk::utils::parse_blob::<ERC20Action>(input.blobs.as_slice(), &BlobIndex(1));
    let transfer_action_contract_name = input.blobs.get(1).unwrap().contract_name.clone();

    let ticket_app_state = input
        .initial_state
        .clone()
        .try_into()
        .expect("Failed to decode state");

    let mut ticket_app_contract = TicketAppContract::new(
        input.identity.clone(),
        ticket_app_contract_name,
        ticket_app_state,
    );

    let res = match ticket_app_action {
        TicketAppAction::BuyTicket {} => {
            ticket_app_contract.buy_ticket(transfer_action, transfer_action_contract_name)
        }
        TicketAppAction::HasTicket {} => ticket_app_contract.has_ticket(),
    };

    sdk::guest::commit(input, ticket_app_contract.state, res);
}
