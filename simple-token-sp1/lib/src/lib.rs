use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use sdk::{erc20::ERC20, Digestable, Identity, RunResult};

/// Entry point of the contract's logic
pub fn execute(contract_input: sdk::ContractInput) -> RunResult<TokenContract> {
    // Parse contract inputs
    let (input, parsed_blob, caller) =
        match sdk::guest::init_with_caller::<sdk::erc20::ERC20Action>(contract_input) {
            Ok(res) => res,
            Err(err) => {
                panic!("Hyllar contract initialization failed {}", err);
            }
        };

    // Parse initial state as Token
    let state: TokenContractState = input.initial_state.clone().into();

    // Execute the given action
    let contract = TokenContract::init(state, caller);

    sdk::erc20::execute_action(contract, parsed_blob.data.parameters)
}

/// The state of the contract, that is totally serialized on-chain
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct TokenContractState {
    total_supply: u128,
    balances: BTreeMap<String, u128>, // Balances for each account
}

/// A struct that holds the logic of the contract
#[derive(Debug)]
pub struct TokenContract {
    state: TokenContractState,
    caller: Identity,
}

impl TokenContractState {
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
        TokenContractState {
            total_supply: initial_supply,
            balances,
        }
    }
}

impl TokenContract {
    pub fn init(state: TokenContractState, caller: Identity) -> TokenContract {
        TokenContract { state, caller }
    }
    pub fn state(self) -> TokenContractState {
        self.state
    }
}

impl TokenContract {
    fn caller(&self) -> &Identity {
        &self.caller
    }
}

// The ERC20 trait is implemented for the TokenContract struct
// This trait is given by the sdk, as a "standard" for identity verification contracts
// but you could do the same logic without it.
/// A more feature-complete implementation is available in hyle repo
/// under crates/contracts/hyllar
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

/// Helpers to transform the contrat's state in its on-chain state digest version.
/// In an optimal version, you would here only returns a hash of the state,
/// while storing the full-state off-chain
impl Digestable for TokenContractState {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(borsh::to_vec(self).expect("Failed to encode Balances"))
    }
}
impl From<sdk::StateDigest> for TokenContractState {
    fn from(state: sdk::StateDigest) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode hyllar state".to_string())
            .unwrap()
    }
}

impl Digestable for TokenContract {
    fn as_digest(&self) -> sdk::StateDigest {
        self.state.as_digest()
    }
}
