use std::collections::BTreeMap;

use bincode::{Decode, Encode};
use sdk::{erc20::ERC20, Digestable, Identity};
use serde::{Deserialize, Serialize};

/// Struct representing the Token state.
#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    total_supply: u128,
    balances: BTreeMap<String, u128>, // Balances for each account
}

#[derive(Debug)]
pub struct TokenContract {
    state: Token,
    caller: Identity,
}

impl Token {
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
        Token {
            total_supply: initial_supply,
            balances,
        }
    }
}

impl TokenContract {
    pub fn init(state: Token, caller: Identity) -> TokenContract {
        TokenContract { state, caller }
    }
    pub fn state(self) -> Token {
        self.state
    }
}

impl TokenContract {
    fn caller(&self) -> &Identity {
        &self.caller
    }
}

/// A more feature-complete implementation is available in hyle repo
/// under contracts/hyllar
impl ERC20 for TokenContract {
    fn total_supply(&self) -> Result<u128, String> {
        Ok(self.state.total_supply)
    }

    fn balance_of(&self, account: &str) -> Result<u128, String> {
        match self.state.balances.get(account) {
            Some(&balance) => Ok(balance),
            None => Err(format!("Account {account} not found")),
        }
    }

    fn transfer(&mut self, recipient: &str, amount: u128) -> Result<(), String> {
        let sender = self.caller();
        let sender = sender.0.as_str();
        let sender_balance = self.balance_of(sender)?;

        if sender_balance < amount {
            return Err("Insufficient balance".to_string());
        }

        *self.state.balances.entry(sender.to_string()).or_insert(0) -= amount;
        *self
            .state
            .balances
            .entry(recipient.to_string())
            .or_insert(0) += amount;

        Ok(())
    }

    fn transfer_from(
        &mut self,
        _sender: &str,
        _recipient: &str,
        _amount: u128,
    ) -> Result<(), String> {
        todo!()
    }

    fn approve(&mut self, _spender: &str, _amount: u128) -> Result<(), String> {
        todo!()
    }

    fn allowance(&self, _owner: &str, _spender: &str) -> Result<u128, String> {
        todo!()
    }
}

impl Digestable for Token {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(
            bincode::encode_to_vec(self, bincode::config::standard())
                .expect("Failed to encode Balances"),
        )
    }
}
impl From<sdk::StateDigest> for Token {
    fn from(state: sdk::StateDigest) -> Self {
        let (state, _) = bincode::decode_from_slice(&state.0, bincode::config::standard())
            .map_err(|_| "Could not decode hyllar state".to_string())
            .unwrap();
        state
    }
}
