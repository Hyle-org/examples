use methods::METHOD_ELF;

use utils::{Balances, ContractFunction, TokenContractInput};

use borsh::to_vec;

use clap::{Parser, Subcommand};
use hyle_contract::HyleOutput;
use risc0_zkvm::{default_prover, sha::Digestible, ExecutorEnv};

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
    reproducible: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.reproducible {
        println!("Running with reproducible ELF binary.");
    } else {
        println!("Running non-reproducibly");
    }

    let prove_info = prove(cli.reproducible, cli.command.into());

    let receipt = prove_info.receipt;
    let encoded_receipt = to_vec(&receipt).expect("Unable to encode receipt");
    std::fs::write("erc20.risc0.proof", encoded_receipt).unwrap();

    let claim = receipt.claim().unwrap().value().unwrap();

    let hyle_output = receipt
        .journal
        .decode::<HyleOutput<String>>()
        .expect("Failed to decode journal");

    println!("{}", "-".repeat(20));
    println!("Method ID: {:?} (hex)", claim.pre.digest());
    println!(
        "erc20.risc0.proof written, transition from {:?} to {:?}",
        hex::encode(&hyle_output.initial_state),
        hex::encode(&hyle_output.next_state)
    );
    println!("{:?}", hyle_output);

    receipt
        .verify(claim.pre.digest())
        .expect("Verification 2 failed");
}

fn prove(reproducible: bool, program_inputs: ContractFunction) -> risc0_zkvm::ProveInfo {
    // TODO: Allow user to add custom balance
    let balances = Balances::default();
    // TODO: Allow user to add real tx_hash
    let tx_hash = vec![1];
    // TODO: Allow user to add multiple values in payload
    let payloads = vec![program_inputs.encode()];
    let index = 0;

    let env = ExecutorEnv::builder()
        .write(&TokenContractInput {
            balances,
            tx_hash,
            payloads,
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
