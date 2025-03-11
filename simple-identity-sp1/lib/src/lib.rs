use std::collections::BTreeMap;

use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use sdk::RunResult;
use sha2::{Digest, Sha256};

impl sdk::HyleContract for IdentityContractState {
    /// Entry point of the contract's logic
    fn execute(&mut self, contract_input: &sdk::ContractInput) -> RunResult {
        // Parse contract inputs
        let (action, ctx) = sdk::utils::parse_raw_contract_input::<IdentityAction>(contract_input)?;

        // Extract private information
        let password = core::str::from_utf8(&contract_input.private_input).unwrap();

        // Execute the given action
        let res = match action {
            IdentityAction::RegisterIdentity { account } => {
                self.register_identity(&account, password)?
            }
            IdentityAction::VerifyIdentity { account, nonce } => {
                self.verify_identity(&account, nonce, password)?
            }
        };

        Ok((res, ctx, vec![]))
    }

    /// In this example, we serialize the full state on-chain.
    fn commit(&self) -> sdk::StateCommitment {
        sdk::StateCommitment(borsh::to_vec(&self).expect("Failed to encode state"))
    }
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

/// Enum representing the actions that can be performed by the IdentityVerification contract.
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum IdentityAction {
    RegisterIdentity { account: String },
    VerifyIdentity { account: String, nonce: u32 },
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

    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}

impl IdentityContractState {
    fn register_identity(&mut self, account: &str, private_input: &str) -> Result<String, String> {
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
            return Err("Identity already exists".to_string());
        }
        Ok("Successfully registered identity for account: {}".to_string())
    }

    fn verify_identity(
        &mut self,
        account: &str,
        nonce: u32,
        private_input: &str,
    ) -> Result<String, String> {
        match self.identities.get_mut(account) {
            Some(stored_info) => {
                if nonce != stored_info.nonce {
                    return Err("Invalid nonce".to_string());
                }
                let id = format!("{account}:{private_input}");
                let mut hasher = Sha256::new();
                hasher.update(id.as_bytes());
                let hashed = hex::encode(hasher.finalize());
                if *stored_info.hash != hashed {
                    return Err("Invalid private input".to_string());
                }
                stored_info.nonce += 1;
                Ok("Identity verified".to_string())
            }
            None => Err("Identity not found".to_string()),
        }
    }
}

impl Default for IdentityContractState {
    fn default() -> Self {
        Self::new()
    }
}

impl From<sdk::StateCommitment> for IdentityContractState {
    fn from(state: sdk::StateCommitment) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode identity state".to_string())
            .unwrap()
    }
}
