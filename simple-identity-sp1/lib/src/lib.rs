use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use sdk::{identity_provider::IdentityVerification, Digestable, RunResult};
use sha2::{Digest, Sha256};

/// Entry point of the contract's logic
pub fn execute(contract_input: sdk::ContractInput) -> RunResult<IdentityContractState> {
    // Parse contract inputs
    let (input, action) =
        sdk::guest::init_raw::<sdk::identity_provider::IdentityAction>(contract_input);

    let action = action.ok_or("Failed to parse action")?;

    // Parse initial state
    let state: IdentityContractState = input.initial_state.clone().into();

    // Extract private information
    let password = core::str::from_utf8(&input.private_input).unwrap();

    // Execute the given action
    sdk::identity_provider::execute_action(state, action, password)
}

/// Struct to hold account's information
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AccountInfo {
    pub hash: String,
    pub nonce: u32,
}

/// The state of the contract, that is totally serialized on-chain
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct IdentityContractState {
    identities: BTreeMap<String, AccountInfo>,
}

/// Some helper methods for the state
impl IdentityContractState {
    pub fn new() -> Self {
        IdentityContractState {
            identities: BTreeMap::new(),
        }
    }

    pub fn get_nonce(&self, username: &str) -> Result<u32, &'static str> {
        let info = self.identities.get(username).ok_or("Identity not found")?;
        Ok(info.nonce)
    }
}

// The IdentityVerification trait is implemented for the IdentityContractState struct
// This trait is given by the sdk, as a "standard" for identity verification contracts
// but you could do the same logic without it.
impl IdentityVerification for IdentityContractState {
    fn register_identity(
        &mut self,
        account: &str,
        private_input: &str,
    ) -> Result<(), &'static str> {
        let id = format!("{account}:{private_input}");
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        let hash_bytes = hasher.finalize();
        let account_info = AccountInfo {
            hash: hex::encode(hash_bytes),
            nonce: 0,
        };

        if self
            .identities
            .insert(account.to_string(), account_info)
            .is_some()
        {
            return Err("Identity already exists");
        }
        Ok(())
    }

    fn verify_identity(
        &mut self,
        account: &str,
        nonce: u32,
        private_input: &str,
    ) -> Result<bool, &'static str> {
        match self.identities.get_mut(account) {
            Some(stored_info) => {
                if nonce != stored_info.nonce {
                    return Err("Invalid nonce");
                }
                let id = format!("{account}:{private_input}");
                let mut hasher = Sha256::new();
                hasher.update(id.as_bytes());
                let hashed = hex::encode(hasher.finalize());
                if *stored_info.hash != hashed {
                    return Ok(false);
                }
                stored_info.nonce += 1;
                Ok(true)
            }
            None => Err("Identity not found"),
        }
    }

    fn get_identity_info(&self, account: &str) -> Result<String, &'static str> {
        match self.identities.get(account) {
            Some(info) => Ok(serde_json::to_string(&info).map_err(|_| "Failed to serialize")?),
            None => Err("Identity not found"),
        }
    }
}

impl Default for IdentityContractState {
    fn default() -> Self {
        Self::new()
    }
}

/// Helpers to transform the contrat's state in its on-chain state digest version.
/// In an optimal version, you would here only returns a hash of the state,
/// while storing the full-state off-chain
impl Digestable for IdentityContractState {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(borsh::to_vec(self).expect("Failed to encode Balances"))
    }
}
impl From<sdk::StateDigest> for IdentityContractState {
    fn from(state: sdk::StateDigest) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode identity state".to_string())
            .unwrap()
    }
}
