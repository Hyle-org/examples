use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use sdk::{erc20::ERC20Action, BlobIndex, ContractName, Digestable, Identity, RunResult};

/// Entry point of the contract's logic
pub fn execute(contract_input: sdk::ContractInput) -> RunResult<TicketAppContract> {
    let (input, ticket_app_action) = sdk::guest::init_raw::<TicketAppAction>(contract_input);
    let ticket_app_contract_name = input
        .blobs
        .get(input.index.0)
        .unwrap()
        .contract_name
        .clone();
    let ticket_app_action = ticket_app_action.ok_or("failed to parse action")?;

    let transfer_action =
        sdk::utils::parse_blob::<ERC20Action>(input.blobs.as_slice(), &BlobIndex(1))
            .ok_or("failed to parse action")?;

    let transfer_action_contract_name = input.blobs.get(1).unwrap().contract_name.clone();

    let ticket_app_state = input.initial_state.clone().into();

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

    res.map(|output| (output, ticket_app_contract, vec![]))
}

/// Enum representing the actions that can be performed by the Amm contract.
#[derive(Encode, Decode, Debug, Clone)]
pub enum TicketAppAction {
    BuyTicket {},
    HasTicket {},
}

/// A struct that holds the logic of the contract
pub struct TicketAppContract {
    identity: Identity,
    contract_name: ContractName,
    pub state: TicketAppState,
}

/// The state of the contract, that is totally serialized on-chain
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

    pub fn buy_ticket(
        &mut self,
        erc20_action: ERC20Action,
        erc20_name: ContractName,
    ) -> Result<String, String> {
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

    pub fn has_ticket(&mut self) -> Result<String, String> {
        // Check that a blob exists matching the given action, pop it from the callee blobs.

        if self.state.tickets.contains(&self.identity) {
            Ok(format!("Ticket present for {:?}", &self.identity))
        } else {
            Err(format!("No Ticket for {:?}", &self.identity))
        }
    }
}

/// Helpers to transform the contrat's state in its on-chain state digest version.
/// In an optimal version, you would here only returns a hash of the state,
/// while storing the full-state off-chain
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
impl Digestable for TicketAppContract {
    fn as_digest(&self) -> sdk::StateDigest {
        self.state.as_digest()
    }
}
