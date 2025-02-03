use bincode::{Decode, Encode};
use hex::{decode, encode};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use sdk::{identity_provider::IdentityVerification, Digestable, HyleOutput};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha3::Keccak256;
use std::collections::BTreeMap;

/// Entry point of the contract's logic
pub fn execute(contract_input: sdk::ContractInput) -> HyleOutput {
    // Parse contract inputs
    let (input, action) =
        sdk::guest::init_raw::<sdk::identity_provider::IdentityAction>(contract_input);

    // Parse initial state
    let mut state: IdentityContractState = input.initial_state.clone().into();

    // Extract private information
    let signature = core::str::from_utf8(&input.private_blob.0).unwrap();

    // Execute the given action
    let res = sdk::identity_provider::execute_action(&mut state, action, signature);

    sdk::utils::as_hyle_output(input, state, res)
}

/// Struct to hold account's information
#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AccountInfo {
    pub pub_key_hash: String,
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

    pub fn get_nonce(&self, account: &str) -> Result<u32, &'static str> {
        let info = self.identities.get(account).ok_or("Identity not found")?;
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
        // Parse the signature
        let pub_key = account.trim_end_matches(".metamask_identity");

        let valid = k256_verifier(pub_key, private_input, "hyle registration");

        if !valid {
            return Err("Invalid signature");
        }

        let pub_key_hash = Keccak256::digest(account.as_bytes());
        let pub_key_hash_hex = encode(pub_key_hash);

        let account_info = AccountInfo {
            pub_key_hash: pub_key_hash_hex,
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
        _private_input: &str,
    ) -> Result<bool, &'static str> {
        match self.identities.get_mut(account) {
            Some(stored_info) => {
                if nonce != stored_info.nonce {
                    return Err("Invalid nonce");
                }

                // ✅ Step 2: Compute Keccak256 hash of the account (to match register_identity)
                let pub_key_hash = Keccak256::digest(account.as_bytes());
                let computed_hash = encode(pub_key_hash);

                if *stored_info.pub_key_hash != computed_hash {
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

pub fn k256_verifier(mut pub_key: &str, mut signature_hex: &str, message: &str) -> bool {
    pub_key = sanitize_hex(&pub_key);
    signature_hex = sanitize_hex(&signature_hex);

    let msg = message.as_bytes();

    // Apply Ethereum Signed Message Prefix (EIP-191)
    let eth_message = format!(
        "\x19Ethereum Signed Message:\n{}{}",
        msg.len(),
        String::from_utf8_lossy(msg)
    );

    let signature_bytes = decode(&signature_hex).expect("Invalid hex string");
    let recovery_id_byte = signature_bytes.last().copied().expect("Signature is empty");
    // Normalize Ethereum's recovery ID
    let recovery_id_byte = if recovery_id_byte >= 27 {
        recovery_id_byte - 27
    } else {
        recovery_id_byte
    };
    let recovery_id = RecoveryId::try_from(recovery_id_byte).expect("Wrong recover id byte");

    let signature = Signature::from_slice(&signature_bytes[..64]).expect("Wrong signature");

    let recovered_key = VerifyingKey::recover_from_digest(
        Keccak256::new_with_prefix(eth_message),
        &signature,
        recovery_id,
    )
    .expect("Error when recovering public key");

    let encoded_point = recovered_key.to_encoded_point(false);
    let pub_key_bytes = encoded_point.as_bytes();

    // Hash the public key (skip the first byte which is always 0x04)
    let hashed_key = Keccak256::digest(&pub_key_bytes[1..]);

    // Extract the last 20 bytes (Ethereum address format)
    let recovered_address = &hashed_key[12..]; // Last 20 bytes
    hex::encode(recovered_address) == pub_key
}

fn sanitize_hex(hex_str: &str) -> &str {
    hex_str.strip_prefix("0x").unwrap_or(hex_str)
}
