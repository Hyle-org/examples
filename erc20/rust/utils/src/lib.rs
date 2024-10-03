#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use alloc::{string::String, vec};
use anyhow::{bail, Error};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Account {
    name: String,
    balance: u64,
}

impl Account {
    pub fn new(name: String, balance: u64) -> Self {
        Account { name, balance }
    }

    pub fn send(&mut self, amount: u64) -> Result<(), Error> {
        if self.balance >= amount {
            self.balance -= amount;
            Ok(())
        } else {
            bail!("Not enough funds in account '{}'", self.name);
        }
    }

    pub fn receive(&mut self, amount: u64) {
        self.balance += amount;
    }
}

#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct Balances {
    accounts: Vec<Account>,
}
impl Default for Balances {
    fn default() -> Self {
        Balances {
            accounts: vec![Account::new("faucet".to_owned(), 1_000_000)],
        }
    }
}

impl Balances {
    pub fn add_account(&mut self, account: Account) {
        self.accounts.push(account);
    }

    pub fn get_account_index(&self, name: &str) -> Option<usize> {
        self.accounts.iter().position(|acc| acc.name == name)
    }

    pub fn get_balance(&self, name: &str) -> Result<u64, Error> {
        match self.get_account_index(name) {
            Some(index) => Ok(self.accounts[index].balance),
            None => bail!("Account '{}' does not exist", name),
        }
    }

    pub fn send(&mut self, from: &str, to: &str, amount: u64) -> Result<(), Error> {
        let from_index = match self.get_account_index(from) {
            Some(index) => index,
            None => bail!("Account '{}' does not exist", from),
        };

        self.accounts[from_index].send(amount)?;

        let to_index = match self.get_account_index(to) {
            Some(index) => index,
            None => {
                self.add_account(Account::new(to.to_owned(), 0));
                self.accounts.len() - 1
            }
        };

        self.accounts[to_index].receive(amount);
        Ok(())
    }

    pub fn mint(&mut self, to: &str, amount: u64) -> Result<(), Error> {
        let faucet_account_index = match self.get_account_index("faucet") {
            Some(index) => index,
            None => bail!("Account faucet does not exist. This should not happen"),
        };

        self.accounts[faucet_account_index].send(amount)?;

        let to_index = match self.get_account_index(to) {
            Some(index) => index,
            None => {
                self.add_account(Account::new(to.to_owned(), 0));
                self.accounts.len() - 1
            }
        };
        self.accounts[to_index].receive(amount);
        Ok(())
    }

    pub fn hash(&self) -> Vec<u8> {
        // TODO: maybe we could do something smarter here to allow Merkle inclusion proof
        let mut hasher = Sha3_256::new();
        for account in &self.accounts {
            hasher.update(account.name.as_bytes());
            hasher.update(account.balance.to_be_bytes());
        }
        return hasher.finalize().as_slice().to_owned();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenContractInput {
    pub balances: Balances,
    pub tx_hash: Vec<u8>,
    pub payloads: Vec<Vec<u8>>,
    pub index: usize,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
pub enum ContractFunction {
    Transfer {
        from: String,
        to: String,
        amount: u64,
    },
    Mint {
        to: String,
        amount: u64,
    },
}
impl ContractFunction {
    pub fn encode(&self) -> Vec<u8> {
        bincode::encode_to_vec(self, bincode::config::standard())
            .expect("Failed to encode ContractFunction")
    }

    pub fn decode(data: &[u8]) -> Self {
        let (v, _) = bincode::decode_from_slice(data, bincode::config::standard())
            .expect("Failed to decode ContractFunction");
        v
    }
}
