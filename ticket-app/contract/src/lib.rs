use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use sdk::{caller::ExecutionContext, BlobIndex, ContractName, Identity, RunResult};
use simple_token::SimpleTokenAction;

impl sdk::HyleContract for TicketAppState {
    /// Entry point of the contract's logic
    fn execute(&mut self, contract_input: &sdk::ContractInput) -> RunResult {
        let (ticket_app_action, ctx) =
            sdk::utils::parse_raw_contract_input::<TicketAppAction>(contract_input)?;

        let transfer_action = sdk::utils::parse_blob::<SimpleTokenAction>(
            contract_input.blobs.as_slice(),
            &BlobIndex(1),
        )
        .ok_or("failed to parse action")?;

        let transfer_action_contract_name =
            contract_input.blobs.get(1).unwrap().contract_name.clone();

        let res = match ticket_app_action {
            TicketAppAction::BuyTicket {} => {
                self.buy_ticket(&ctx, transfer_action, transfer_action_contract_name)?
            }
            TicketAppAction::HasTicket {} => self.has_ticket(&ctx)?,
        };

        Ok((res, ctx, vec![]))
    }

    /// In this example, we serialize the full state on-chain.
    fn commit(&self) -> sdk::StateCommitment {
        sdk::StateCommitment(borsh::to_vec(self).expect("Failed to encode TicketAppState"))
    }
}
/// Enum representing the actions that can be performed by the Amm contract.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TicketAppAction {
    BuyTicket {},
    HasTicket {},
}

/// The state of the contract, that is totally serialized on-chain
#[derive(Debug, Serialize, Deserialize, Clone, BorshSerialize, BorshDeserialize, Default)]
pub struct TicketAppState {
    pub ticket_price: (ContractName, u128),
    pub tickets: Vec<Identity>,
}

impl TicketAppState {
    pub fn new(tickets: Vec<Identity>, ticket_price: (ContractName, u128)) -> Self {
        TicketAppState {
            tickets,
            ticket_price,
        }
    }

    pub fn buy_ticket(
        &mut self,
        ctx: &ExecutionContext,
        erc20_action: SimpleTokenAction,
        erc20_name: ContractName,
    ) -> Result<String, String> {
        // Check that a blob exists matching the given action, pop it from the callee blobs.

        if self.tickets.contains(&ctx.caller) {
            return Err(format!("Ticket already present for {:?}", &ctx.caller));
        }

        match erc20_action {
            SimpleTokenAction::Transfer { recipient, amount } => {
                if recipient != ctx.contract_name.0 {
                    return Err(format!(
                        "Transfer recipient should be {} but was {}",
                        ctx.contract_name, &recipient
                    ));
                }

                if self.ticket_price.0 != erc20_name {
                    return Err(format!(
                        "Transfer token should be {} but was {}",
                        self.ticket_price.0, &erc20_name
                    ));
                }

                if amount < self.ticket_price.1 {
                    return Err(format!(
                        "Transfer amount should be at least {} but was {}",
                        self.ticket_price.0, &recipient
                    ));
                }
            }
        }

        let program_outputs = format!("Ticket created for {:?}", ctx.caller);

        self.tickets.push(ctx.caller.clone());

        Ok(program_outputs)
    }

    pub fn has_ticket(&self, ctx: &ExecutionContext) -> Result<String, String> {
        // Check that a blob exists matching the given action, pop it from the callee blobs.

        if self.tickets.contains(&ctx.caller) {
            Ok(format!("Ticket present for {:?}", &ctx.caller))
        } else {
            Err(format!("No Ticket for {:?}", &ctx.caller))
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, std::io::Error> {
        borsh::to_vec(self)
    }
}

impl From<sdk::StateCommitment> for TicketAppState {
    fn from(state: sdk::StateCommitment) -> Self {
        borsh::from_slice(&state.0).expect("Could not decode TicketAppState")
    }
}
