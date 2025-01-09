use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use bincode::{Decode, Encode};
use sdk::caller::{CalleeBlobs, CallerCallee, CheckCalleeBlobs, ExecutionContext, MutCalleeBlobs};
use sdk::erc20::{ERC20BlobChecker, ERC20};
use sdk::{erc20::ERC20Action, Identity};
use sdk::{Blob, BlobIndex, ContractAction, Digestable, RunResult};
use sdk::{BlobData, ContractName, StructuredBlobData};
use serde::{Deserialize, Serialize};

pub type TokenPair = (String, String);
pub type TokenPairAmount = (u128, u128);

#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, Ord, PartialOrd)]
pub struct UnorderedTokenPair {
    a: String,
    b: String,
}

impl UnorderedTokenPair {
    pub fn new(x: String, y: String) -> Self {
        if x <= y {
            UnorderedTokenPair { a: x, b: y }
        } else {
            UnorderedTokenPair { a: y, b: x }
        }
    }
}

impl PartialEq for UnorderedTokenPair {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b
    }
}

impl Eq for UnorderedTokenPair {}

impl Hash for UnorderedTokenPair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.a.hash(state);
        self.b.hash(state);
    }
}

pub struct AmmContract {
    pub exec_ctx: ExecutionContext,
    contract_name: ContractName,
    state: AmmState,
}

impl CallerCallee for AmmContract {
    fn caller(&self) -> &Identity {
        &self.exec_ctx.caller
    }
    fn callee_blobs(&self) -> CalleeBlobs {
        CalleeBlobs(self.exec_ctx.callees_blobs.borrow())
    }
    fn mut_callee_blobs(&self) -> MutCalleeBlobs {
        MutCalleeBlobs(self.exec_ctx.callees_blobs.borrow_mut())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, Default)]
pub struct AmmState {
    pairs: BTreeMap<UnorderedTokenPair, TokenPairAmount>,
}

impl AmmState {
    pub fn new(pairs: BTreeMap<UnorderedTokenPair, TokenPairAmount>) -> Self {
        AmmState { pairs }
    }

    pub fn get_paired_amount(
        &self,
        token_a: String,
        token_b: String,
        amount_a: u128,
    ) -> Option<u128> {
        let pair = UnorderedTokenPair::new(token_a, token_b);
        if let Some((k, x, y)) = self.pairs.get(&pair).map(|(x, y)| (x * y, *x, *y)) {
            let amount_b = y - (k / (x + amount_a));
            return Some(amount_b);
        }
        None
    }
}

impl AmmContract {
    pub fn new(exec_ctx: ExecutionContext, contract_name: ContractName, state: AmmState) -> Self {
        AmmContract {
            exec_ctx,
            contract_name,
            state,
        }
    }

    pub fn state(self) -> AmmState {
        self.state
    }

    pub fn contract_name(self) -> ContractName {
        self.contract_name
    }

    pub fn create_new_pair(
        &mut self,
        pair: (String, String),
        amounts: TokenPairAmount,
    ) -> RunResult {
        // Check that new pair is about two different tokens and that there is one blob for each
        if pair.0 == pair.1 {
            return Err("Swap can only happen between two different tokens".to_string());
        }

        // Check that a blob exists matching the given action, pop it from the callee blobs.
        self.is_in_callee_blobs(
            &ContractName(pair.0.clone()),
            ERC20Action::TransferFrom {
                sender: self.caller().0.clone(),
                recipient: self.contract_name.0.clone(),
                amount: amounts.0,
            },
        )?;

        // Sugared version using traits
        ERC20BlobChecker::new(&ContractName(pair.1.clone()), &self).transfer_from(
            &self.caller().0,
            &self.contract_name.0,
            amounts.1,
        )?;

        let normalized_pair = UnorderedTokenPair::new(pair.0, pair.1);

        if self.state.pairs.contains_key(&normalized_pair) {
            return Err(format!("Pair {:?} already exists", normalized_pair));
        }

        let program_outputs = format!("Pair {:?} created", normalized_pair);

        self.state.pairs.insert(normalized_pair, amounts);

        Ok(program_outputs)
    }

    pub fn verify_swap(
        &mut self,
        pair: TokenPair,
        from_amount: u128,
        to_amount: u128,
    ) -> RunResult {
        // Check that swap is only about two different tokens
        if pair.0 == pair.1 {
            return Err("Swap can only happen between two different tokens".to_string());
        }

        // Check that we were transferred the correct amount of tokens
        self.is_in_callee_blobs(
            &ContractName(pair.0.clone()),
            ERC20Action::TransferFrom {
                sender: self.caller().0.clone(),
                recipient: self.contract_name.0.clone(),
                amount: from_amount,
            },
        )?;

        // Compute x,y and check swap is legit (x*y=k)
        let normalized_pair = UnorderedTokenPair::new(pair.0.clone(), pair.1.clone());
        let is_normalized_order = pair.0 <= pair.1;
        let Some((prev_x, prev_y)) = self.state.pairs.get_mut(&normalized_pair) else {
            return Err(format!("Pair {:?} not found in AMM state", pair));
        };
        let expected_to_amount = if is_normalized_order {
            let amount = *prev_y - (*prev_x * *prev_y / (*prev_x + from_amount));
            *prev_x += from_amount;
            *prev_y -= amount; // we need to remove the full amount to avoid slipping
            amount
        } else {
            let amount = *prev_x - (*prev_y * *prev_x / (*prev_y + from_amount));
            *prev_y += from_amount;
            *prev_x -= amount; // we need to remove the full amount to avoid slipping
            amount
        };

        // Assert that we transferred less than that, within 2%
        if to_amount > expected_to_amount || to_amount < expected_to_amount * 98 / 100 {
            return Err(format!(
                "Invalid swap: expected to receive between {} and {} {}",
                expected_to_amount * 100 / 102,
                expected_to_amount,
                pair.1
            ));
        }

        // At this point the contract has effectively taken some fees but we don't actually count them.

        // Check that we transferred the correct amount of tokens
        self.is_in_callee_blobs(
            &ContractName(pair.1.clone()),
            ERC20Action::Transfer {
                recipient: self.caller().0.clone(),
                amount: to_amount,
            },
        )?;

        Ok(format!(
            "Swap of {} {} for {} {} is valid",
            self.caller(),
            pair.0,
            self.caller(),
            pair.1
        ))
    }
}

impl Digestable for AmmState {
    fn as_digest(&self) -> sdk::StateDigest {
        sdk::StateDigest(
            bincode::encode_to_vec(self, bincode::config::standard())
                .expect("Failed to encode AmmState"),
        )
    }
}
impl From<sdk::StateDigest> for AmmState {
    fn from(state: sdk::StateDigest) -> Self {
        let (amm_state, _) = bincode::decode_from_slice(&state.0, bincode::config::standard())
            .expect("Could not decode amm state");
        amm_state
    }
}

/// Enum representing the actions that can be performed by the Amm contract.
#[derive(Encode, Decode, Debug, Clone)]
pub enum AmmAction {
    Swap {
        pair: TokenPair, // User swaps the first token of the pair for the second token
        amounts: TokenPairAmount,
    },
    NewPair {
        pair: TokenPair,
        amounts: TokenPairAmount,
    },
}

impl ContractAction for AmmAction {
    fn as_blob(
        &self,
        contract_name: ContractName,
        caller: Option<BlobIndex>,
        callees: Option<Vec<BlobIndex>>,
    ) -> Blob {
        Blob {
            contract_name,
            data: BlobData::from(StructuredBlobData {
                caller,
                callees,
                parameters: self.clone(),
            }),
        }
    }
}
