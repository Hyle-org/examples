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

use sdk::{erc20::ERC20, Digestable, HyleContract, RunResult};

impl HyleContract for SimpleToken {
    /// Entry point of the contract's logic
    fn execute(&mut self, contract_input: &sdk::ContractInput) -> RunResult {
        // Parse contract inputs
        let (action, ctx) =
            sdk::utils::parse_raw_contract_input::<sdk::erc20::ERC20Action>(contract_input)?;

        // Execute the given action
        let res = self.execute_token_action(action, &ctx)?;

        Ok((res, ctx, alloc::vec![]))
    }
}

/// The state of the contract, that is totally serialized on-chain
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct SimpleToken {
    total_supply: u128,
    balances: BTreeMap<String, u128>, // Balances for each account
}

impl SimpleToken {
    /// Creates a new token with the specified initial supply.
    ///
    /// # Arguments
    ///
    /// * `initial_supply` - The initial supply of the token.
    ///
    /// # Returns
    ///
    /// * `HyllarToken` - A new instance of the Hyllar token.
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

// The ERC20 trait is implemented for the TokenContract struct
// This trait is given by the sdk, as a "standard" for identity verification contracts
// but you could do the same logic without it.
/// A more feature-complete implementation is available in hyle repo
/// under crates/contracts/hyllar
impl ERC20 for SimpleToken {
    fn total_supply(&self) -> Result<u128, String> {
        Ok(self.total_supply)
    }

    fn balance_of(&self, account: &str) -> Result<u128, String> {
        match self.balances.get(account) {
            Some(&balance) => Ok(balance),
            None => Err(format!("Account {account} not found")),
        }
    }

    fn transfer(&mut self, sender: &str, recipient: &str, amount: u128) -> Result<(), String> {
        let sender_balance = self.balance_of(sender)?;

        if sender_balance < amount {
            return Err("Insufficient balance".to_string());
        }

        *self.balances.entry(sender.to_string()).or_insert(0) -= amount;
        *self.balances.entry(recipient.to_string()).or_insert(0) += amount;

        Ok(())
    }

    fn transfer_from(
        &mut self,
        _owner: &str,
        _spender: &str,
        _recipient: &str,
        _amount: u128,
    ) -> Result<(), String> {
        todo!()
    }

    fn approve(&mut self, _owner: &str, _spender: &str, _amount: u128) -> Result<(), String> {
        todo!()
    }

    fn allowance(&self, _owner: &str, _spender: &str) -> Result<u128, String> {
        todo!()
    }
}

/// Helpers to transform the contrat's state in its on-chain state digest version.
/// In an optimal version, you would here only returns a hash of the state,
/// while storing the full-state off-chain
impl Digestable for SimpleToken {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(borsh::to_vec(self).expect("Failed to encode Balances"))
    }
}
impl From<sdk::StateDigest> for SimpleToken {
    fn from(state: sdk::StateDigest) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode hyllar state".to_string())
            .unwrap()
    }
}
