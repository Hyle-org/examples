//! A simple script to generate and verify the proof of a given program.

use sp1_core::stark::StarkVerifyingKey;
use sp1_sdk::{CoreSC, ProverClient, SP1Proof, SP1Stdin, SP1VerifyingKey};
use clap::{Parser, Subcommand};
use hyle_contract::{HyleInput, HyleOutput};

use std::fs;
use base64::prelude::*;

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

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");
    

fn main() {
    let cli = Cli::parse();
    let mut initial_state = 1u32;
    match &cli.command {
        Commands::Next { input } => {
            initial_state = *input;
        },
        Commands::Reset { input } => {} // La c'est pas suffisant; il faut prendre en compte le cas ou l'input vaut 0
    };

    // Generate proof.
    let mut stdin = SP1Stdin::new();
    
    let hyle_input = HyleInput {
        initial_state: initial_state.to_be_bytes().to_vec(),
        sender: "".to_string(), //TODO
        caller: "".to_string(), //TODO
        block_number: 0, //TODO
        block_time: 0, //TODO
        tx_hash: vec![1], //TODO
        program_inputs: Some(24u32),
    };
    stdin.write(&hyle_input);

    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);

    if cli.reproducible {
        println!("Running with reproducible ELF binary.");
    } else {
        println!("Running non-reproducibly");
    }
    
    println!("PROVING");
    let mut proof = client.prove(&pk, stdin).expect("proving failed");
    // let mut proof_compressed = client.prove_compressed(&pk, stdin).expect("proving failed");

    // Save proof.
    proof
    .save("proof-with-io.json")
    .expect("saving proof failed");
    // proof_compressed
    // .save("proof_compressed.json")
    // .expect("saving proof failed");

    let vk_json = serde_json::to_string(&vk.vk).unwrap();
    let vk_b64 = BASE64_STANDARD.encode(vk_json.clone());
    fs::write("vk.b64", vk_b64).expect("Unable to write vk.b64");

    // Read output
    let hyle_output = proof.public_values.read::<HyleOutput<u32>>();
    // let hyle_output = proof_compressed.public_values.read::<HyleOutput<u32>>();

    let initial_state_b64 = BASE64_STANDARD.encode(&hyle_output.initial_state);
    let next_state_b64 = BASE64_STANDARD.encode(&hyle_output.next_state);
    let initial_state_u32: u32 = u32::from_be_bytes(hyle_output.initial_state.try_into().unwrap());
    let next_state_u32: u32 = u32::from_be_bytes(hyle_output.next_state.try_into().unwrap());
    let block_number: u64 = hyle_output.block_number;
    let block_time: u64= hyle_output.block_time;
    let program_outputs = hyle_output.program_outputs;

    println!("{}", "-".repeat(20));
    println!("proof-with-io.json written, transition from {} ({}) to {} ({})", initial_state_b64, initial_state_u32, next_state_b64, next_state_u32);
    println!("Aiming block {} at time {}.", block_number, block_time);
    println!("Program outputted {:?}", program_outputs);
}