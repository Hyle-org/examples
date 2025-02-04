use bincode::{Decode, Encode};
use hex;
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use sdk::{identity_provider::IdentityVerification, Digestable, RunResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone)]
pub struct WebAuthnCredential {
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub counter: u32,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone)]
pub struct WebAuthnContractState {
    users: BTreeMap<String, WebAuthnCredential>,
    challenges: BTreeMap<String, Vec<u8>>,
}

impl WebAuthnContractState {
    pub fn new() -> Self {
        WebAuthnContractState {
            users: BTreeMap::new(),
            challenges: BTreeMap::new(),
        }
    }

    pub fn register(
        &mut self,
        username: String,
        credential_id: Vec<u8>,
        public_key: Vec<u8>,
    ) -> Result<(), &'static str> {
        if self.users.contains_key(&username) {
            return Err("User already exists");
        }

        let credential = WebAuthnCredential {
            credential_id,
            public_key,
            counter: 0,
        };

        self.users.insert(username, credential);
        Ok(())
    }

    pub fn start_authentication(
        &mut self,
        username: String,
        challenge: Vec<u8>,
    ) -> Result<(), &'static str> {
        if !self.users.contains_key(&username) {
            return Err("User not found");
        }

        self.challenges.insert(username, challenge);
        Ok(())
    }

    pub fn verify_authentication(
        &mut self,
        username: String,
        signature: Vec<u8>,
        authenticator_data: Vec<u8>,
        client_data_json: Vec<u8>,
    ) -> Result<bool, &'static str> {
        // Get user's credential
        let credential = self.users.get(&username).ok_or("User not found")?;

        // Get stored challenge
        let challenge = self.challenges.get(&username).ok_or("No challenge found")?;

        // Verify the signature using the public key
        let is_valid = verify_p256_signature(
            &credential.public_key,
            &signature,
            &authenticator_data,
            &client_data_json,
            challenge,
        )?;

        if is_valid {
            // Update counter
            if let Some(credential) = self.users.get_mut(&username) {
                credential.counter += 1;
            }
            self.challenges.remove(&username);
        }

        Ok(is_valid)
    }
}

fn verify_p256_signature(
    public_key: &[u8],
    signature: &[u8],
    authenticator_data: &[u8],
    client_data_json: &[u8],
    challenge: &[u8],
) -> Result<bool, &'static str> {
    let client_data_hash = Sha256::digest(client_data_json);

    let mut message = Vec::new();
    message.extend_from_slice(authenticator_data);
    message.extend_from_slice(&client_data_hash);

    let verifying_key =
        VerifyingKey::from_sec1_bytes(public_key).map_err(|_| "Invalid public key")?;

    // Parse the DER-encoded signature.
    let sig = Signature::from_der(signature).map_err(|_| "Invalid signature format")?;

    // Verify the signature against the message.
    if verifying_key.verify(&message, &sig).is_ok() {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Entry point of the contract's logic
pub fn execute(contract_input: sdk::ContractInput) -> RunResult<WebAuthnContractState> {
    let (input, action) = sdk::guest::init_raw::<WebAuthnAction>(contract_input);
    let action = action.ok_or("Failed to parse action")?;
    let mut state: WebAuthnContractState = input.initial_state.clone().into();

    match action {
        WebAuthnAction::Register {
            username,
            credential_id,
            public_key,
        } => {
            state.register(username, credential_id, public_key)?;
        }
        WebAuthnAction::StartAuthentication {
            username,
            challenge,
        } => {
            state.start_authentication(username, challenge)?;
        }
        WebAuthnAction::VerifyAuthentication {
            username,
            signature,
            authenticator_data,
            client_data_json,
        } => {
            state.verify_authentication(
                username,
                signature,
                authenticator_data,
                client_data_json,
            )?;
        }
    }

    // Return the tuple that RunResult expects
    Ok((String::new(), state, vec![]))
}

/// Struct to hold account's information
#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AccountInfo {
    pub hash: String,
    pub nonce: u32,
}

/// The state of the contract, that is totally serialized on-chain
#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone)]
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
        sdk::StateDigest(
            bincode::encode_to_vec(self, bincode::config::standard())
                .expect("Failed to encode Balances"),
        )
    }
}
impl From<sdk::StateDigest> for IdentityContractState {
    fn from(state: sdk::StateDigest) -> Self {
        let (state, _) = bincode::decode_from_slice(&state.0, bincode::config::standard())
            .map_err(|_| "Could not decode identity state".to_string())
            .unwrap();
        state
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug)]
pub enum WebAuthnAction {
    Register {
        username: String,
        credential_id: Vec<u8>,
        public_key: Vec<u8>,
    },
    StartAuthentication {
        username: String,
        challenge: Vec<u8>,
    },
    VerifyAuthentication {
        username: String,
        signature: Vec<u8>,
        authenticator_data: Vec<u8>,
        client_data_json: Vec<u8>,
    },
}

impl From<sdk::StateDigest> for WebAuthnContractState {
    fn from(state: sdk::StateDigest) -> Self {
        let (state, _) = bincode::decode_from_slice(&state.0, bincode::config::standard())
            .map_err(|_| "Could not decode WebAuthn state".to_string())
            .unwrap();
        state
    }
}

// Also implement Digestable for WebAuthnContractState
impl Digestable for WebAuthnContractState {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(
            bincode::encode_to_vec(self, bincode::config::standard())
                .expect("Failed to encode WebAuthn state"),
        )
    }
}
