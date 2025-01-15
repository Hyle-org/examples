use bincode::{Decode, Encode};
use sdk::ContractName;
use sdk::{erc20::ERC20Action, Identity};
use sdk::{Digestable, RunResult};
use serde::{Deserialize, Serialize};

pub struct TicketAppContract {
    identity: Identity,
    contract_name: ContractName,
    pub state: TicketAppState,
}

#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, Default)]
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
}

impl TicketAppContract {
    pub fn new(identity: Identity, contract_name: ContractName, state: TicketAppState) -> Self {
        TicketAppContract {
            identity,
            contract_name,
            state,
        }
    }

    pub fn buy_ticket(&mut self, erc20_action: ERC20Action, erc20_name: ContractName) -> RunResult {
        // Check that a blob exists matching the given action, pop it from the callee blobs.

        if self.state.tickets.contains(&self.identity) {
            return Err(format!("Ticket already present for {:?}", &self.identity));
        }

        match erc20_action {
            ERC20Action::Transfer { recipient, amount } => {
                if recipient != self.contract_name.0 {
                    return Err(format!(
                        "Transfer recipient should be {} but was {}",
                        self.contract_name, &recipient
                    ));
                }

                if self.state.ticket_price.0 != erc20_name {
                    return Err(format!(
                        "Transfer token should be {} but was {}",
                        self.state.ticket_price.0, &erc20_name
                    ));
                }

                if amount < self.state.ticket_price.1 {
                    return Err(format!(
                        "Transfer amount should be at least {} but was {}",
                        self.state.ticket_price.0, &recipient
                    ));
                }
            }
            els => {
                return Err(format!(
                    "Wrong ERC20Action, should be a transfer {:?} to {:?} but was {:?}",
                    self.state.ticket_price, self.contract_name, els
                ));
            }
        }

        let program_outputs = format!("Ticket created for {:?}", self.identity.clone());

        self.state.tickets.push(self.identity.clone());

        Ok(program_outputs)
    }

    pub fn has_ticket(&mut self) -> RunResult {
        // Check that a blob exists matching the given action, pop it from the callee blobs.

        if self.state.tickets.contains(&self.identity) {
            Ok(format!("Ticket present for {:?}", &self.identity))
        } else {
            Err(format!("No Ticket for {:?}", &self.identity))
        }
    }
}

impl Digestable for TicketAppState {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(
            bincode::encode_to_vec(self, bincode::config::standard())
                .expect("Failed to encode TicketAppState"),
        )
    }
}
impl From<sdk::StateDigest> for TicketAppState {
    fn from(state: sdk::StateDigest) -> Self {
        let (ticket_app_state, _) =
            bincode::decode_from_slice(&state.0, bincode::config::standard())
                .expect("Could not decode TicketAppState");
        ticket_app_state
    }
}

/// Enum representing the actions that can be performed by the Amm contract.
#[derive(Encode, Decode, Debug, Clone)]
pub enum TicketAppAction {
    BuyTicket {},
    HasTicket {},
}
