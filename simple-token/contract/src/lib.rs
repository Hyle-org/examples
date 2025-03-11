#![no_std]

extern crate alloc;

use alloc::{
    collections::btree_map::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};

use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use sdk::RunResult;

impl sdk::HyleContract for SimpleToken {
    /// Entry point of the contract's logic
    fn execute(&mut self, contract_input: &sdk::ContractInput) -> RunResult {
        // Parse contract inputs
        let (action, ctx) =
            sdk::utils::parse_raw_contract_input::<SimpleTokenAction>(contract_input)?;

        // Execute the given action
        let res = match action {
            SimpleTokenAction::Transfer { recipient, amount } => {
                self.transfer(&ctx.caller.0, &recipient, amount)?
            }
        };

        Ok((res, ctx, alloc::vec![]))
    }

    /// In this example, we serialize the full state on-chain.
    fn commit(&self) -> sdk::StateCommitment {
        sdk::StateCommitment(self.as_bytes().expect("Failed to encode Balances"))
    }
}

/// The state of the contract, that is totally serialized on-chain
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct SimpleToken {
    pub total_supply: u128,
    pub balances: BTreeMap<String, u128>, // Balances for each account
}

/// Enum representing possible calls to the contract functions.
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum SimpleTokenAction {
    Transfer { recipient: String, amount: u128 },
}

impl SimpleToken {
    /// Creates a new token with the specified initial supply.
    pub fn new(initial_supply: u128, faucet_id: String) -> Self {
        let mut balances = BTreeMap::new();
        balances.insert(faucet_id, initial_supply); // Assign initial supply to faucet
        SimpleToken {
            total_supply: initial_supply,
            balances,
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}

impl SimpleToken {
    pub fn balance_of(&self, account: &str) -> Result<u128, String> {
        match self.balances.get(account) {
            Some(&balance) => Ok(balance),
            None => Err(format!("Account {account} not found")),
        }
    }

    pub fn transfer(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: u128,
    ) -> Result<String, String> {
        let sender_balance = self.balance_of(sender)?;

        if sender_balance < amount {
            return Err("Insufficient balance".to_string());
        }

        *self.balances.entry(sender.to_string()).or_insert(0) -= amount;
        *self.balances.entry(recipient.to_string()).or_insert(0) += amount;

        Ok(format!(
            "Transferred {amount} from {sender} to {recipient}",
            amount = amount,
            sender = sender,
            recipient = recipient
        ))
    }
}

impl From<sdk::StateCommitment> for SimpleToken {
    fn from(state: sdk::StateCommitment) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode hyllar state".to_string())
            .unwrap()
    }
}
