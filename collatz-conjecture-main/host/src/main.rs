use methods::METHOD_ELF;
use risc0_zkvm::{default_prover, sha::Digestible, ExecutorEnv};
use serde_json;

use base64::prelude::*;
use clap::{Parser, Subcommand};
use hyle_contract::{HyleInput, HyleOutput};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[clap(long, short)]
    reproducible: bool,
}

#[derive(Subcommand)]
enum Commands {
    Next { input: u32 },
    Reset { input: u32 },
}

fn main() {
    let cli = Cli::parse();

    if cli.reproducible {
        println!("Running with reproducible ELF binary.");
    } else {
        println!("Running non-reproducibly");
    }

    let receipt = match &cli.command {
        Commands::Next { input } => prove(cli.reproducible, *input, 0),
        Commands::Reset { input } => prove(cli.reproducible, 1, *input)
    };

    let claim = receipt.inner.get_claim().unwrap();

    let receipt_json = serde_json::to_string(&receipt).unwrap();
    std::fs::write("proof.json", receipt_json).unwrap();

    let hyle_output = receipt.journal.decode::<HyleOutput<String>>().unwrap();

    let initial_state_b64 = BASE64_STANDARD.encode(&hyle_output.initial_state);
    let next_state_b64 = BASE64_STANDARD.encode(&hyle_output.next_state);
    let initial_state_u32 = u32::from_be_bytes(hyle_output.initial_state.try_into().unwrap());
    let next_state_u32 = u32::from_be_bytes(hyle_output.next_state.try_into().unwrap());
    let program_outputs = hyle_output.program_outputs;

    println!("{}", "-".repeat(20));
    println!("Method ID: {:?} (hex)", claim.pre.digest());
    println!("proof.json written, transition from {} ({}) to {} ({})", initial_state_b64, initial_state_u32, next_state_b64, next_state_u32);
    println!("Program outputted {:?}", program_outputs);
}

fn prove(reproducible: bool, initial_state: u32, suggested_number: u32) -> risc0_zkvm::Receipt {
    let env = ExecutorEnv::builder()
        .write(&HyleInput {
            initial_state: initial_state.to_be_bytes().to_vec(),
            identity: "".to_string(), //TODO
            tx_hash: vec![1], //TODO
            program_inputs: suggested_number,
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
