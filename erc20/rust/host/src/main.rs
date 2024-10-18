use methods::METHOD_ELF;

use anyhow::{bail, Error, Result};
use sdk::HyleOutput;
use serde::Deserialize;
use utils::{Balances, ContractFunction, TokenContractInput};

use borsh::to_vec;

use clap::{Parser, Subcommand};
use risc0_zkvm::{default_prover, sha::Digestible, ExecutorEnv};

#[derive(Debug, Deserialize)]
pub struct ContractName(pub String);

#[derive(Deserialize, Debug)]
pub struct Contract {
    pub name: ContractName,
    pub program_id: Vec<u8>,
    pub state: sdk::StateDigest,
    pub verifier: String,
}

#[derive(Subcommand)]
pub enum ContractFunctionCommand {
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
impl From<ContractFunctionCommand> for ContractFunction {
    fn from(cmd: ContractFunctionCommand) -> Self {
        match cmd {
            ContractFunctionCommand::Transfer { from, to, amount } => {
                ContractFunction::Transfer { from, to, amount }
            }
            ContractFunctionCommand::Mint { to, amount } => ContractFunction::Mint { to, amount },
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: ContractFunctionCommand,

    #[clap(long, short)]
    init: bool,

    #[clap(long, short)]
    reproducible: bool,

    #[arg(long, default_value = "localhost")]
    pub host: String,

    #[arg(long, default_value = "4321")]
    pub port: u32,

    #[arg(long, alias = "n", default_value = "erc20_rust")]
    pub contract_name: String,
}

fn fetch_current_state(cli: &Cli) -> Result<Balances, Error> {
    let url = format!("http://{}:{}", cli.host, cli.port);
    let resp = reqwest::blocking::get(format!("{}/v1/contract/{}", url, cli.contract_name))?;

    let status = resp.status();
    let body = resp.text()?;

    if let Ok(contract) = serde_json::from_str::<Contract>(&body) {
        println!("Fetched contract: {:?}", contract);
        return Ok(contract.state.try_into()?);
    } else {
        bail!(
            "Failed to parse JSON response, status: {}, body: {}",
            status,
            body
        );
    }
}

fn main() {
    let cli = Cli::parse();

    let initial_state = if cli.init {
        Balances::default()
    } else {
        match fetch_current_state(&cli) {
            Ok(s) => s,
            Err(e) => {
                println!("fetch current state error: {}", e);
                return;
            }
        }
    };

    println!("Inital state: {:?}", initial_state);

    if cli.reproducible {
        println!("Running with reproducible ELF binary.");
    } else {
        println!("Running non-reproducibly");
    }

    let program_inputs: ContractFunction = cli.command.into();
    let hex_program_inputs = hex::encode(program_inputs.encode());
    println!("program_inputs: {:?}", program_inputs);
    println!("program_inputs (hex): {:?}", hex_program_inputs);
    let prove_info = prove(cli.reproducible, program_inputs, initial_state);

    let receipt = prove_info.receipt;
    let encoded_receipt = to_vec(&receipt).expect("Unable to encode receipt");
    std::fs::write("erc20.risc0.proof", encoded_receipt).unwrap();

    let claim = receipt.claim().unwrap().value().unwrap();

    println!("receipt.journal :{:?}", receipt.journal);
    let hyle_output = receipt
        .journal
        .decode::<HyleOutput>()
        .expect("Failed to decode journal");

    println!("{}", "-".repeat(20));
    let method_id = claim.pre.digest();
    let initial_state = hex::encode(&hyle_output.initial_state.0);
    println!("Method ID: {:?} (hex)", method_id);
    println!(
        "erc20.risc0.proof written, transition from {:?} to {:?}",
        initial_state,
        hex::encode(&hyle_output.next_state.0)
    );
    println!("{:?}", hyle_output);

    println!("{}", "-".repeat(20));
    if cli.init {
        println!("You can register the contract by running:");
        println!(
            "hyled contract default risc0 {} {} {}",
            method_id, cli.contract_name, initial_state
        );
    }
    println!("You can send the blob tx:");
    println!("hyled blob IDENTITY erc20_rust {}", hex_program_inputs);
    println!("You can send the proof tx:");
    println!(
        "hyled proof BLOB_TX_HASH 0 {} erc20.risc0.proof",
        cli.contract_name
    );

    receipt
        .verify(claim.pre.digest())
        .expect("Verification 2 failed");
}

fn prove(
    reproducible: bool,
    program_inputs: ContractFunction,
    balances: Balances,
) -> risc0_zkvm::ProveInfo {
    // TODO: Allow user to add real tx_hash
    let tx_hash = "".to_string();
    // TODO: Allow user to add multiple values in payload
    let blobs = vec![program_inputs.encode()];
    let index = 0;

    let env = ExecutorEnv::builder()
        .write(&TokenContractInput {
            balances,
            tx_hash,
            blobs,
            index,
        })
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let binary = if reproducible {
        std::fs::read("target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/method/method")
            .expect("Could not read ELF binary at target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/method/method")
    } else {
        METHOD_ELF.to_vec()
    };
    prover.prove(env, &binary).unwrap()
}
