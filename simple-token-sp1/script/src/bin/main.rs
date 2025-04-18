use anyhow::Context;
use clap::{Parser, Subcommand};
use client_sdk::helpers::sp1::SP1Prover;
use contract::{SimpleToken, SimpleTokenAction};
use sdk::BlobTransaction;
use sdk::ContractAction;
use sdk::ProofTransaction;
use sdk::{ContractInput, HyleContract};

use sp1_sdk::include_elf;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const CONTRACT_ELF: &[u8] = include_elf!("simple_token");

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value = "http://localhost:4321")]
    pub host: String,

    #[arg(long, default_value = "simple_token")]
    pub contract_name: String,
}

#[derive(Subcommand)]
enum Commands {
    Register {
        supply: u128,
    },
    Transfer {
        from: String,
        to: String,
        amount: u128,
    },
    Balance {
        of: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let client = client_sdk::rest_client::NodeApiHttpClient::new(cli.host)?;

    let contract_name = &cli.contract_name;

    println!("🚀 Booting sp1...");
    let prover = SP1Prover::new(CONTRACT_ELF);

    match cli.command {
        Commands::Register { supply } => {
            // Build initial state of contract
            let initial_state = SimpleToken::new(supply, format!("faucet@{}", contract_name));
            println!("Initial state: {:?}", initial_state);

            let vk = serde_json::to_vec(&prover.vk).unwrap();

            // Send the transaction to register the contract
            let res = client
                .register_contract(&sdk::api::APIRegisterContract {
                    verifier: "sp1-4".into(),
                    program_id: sdk::ProgramId(vk),
                    state_commitment: initial_state.commit(),
                    contract_name: contract_name.clone().into(),
                })
                .await?;

            println!("✅ Register contract tx sent. Tx hash: {}", res);
        }
        Commands::Balance { of } => {
            // Fetch the state from the node
            let state: SimpleToken = client
                .get_contract(&contract_name.clone().into())
                .await
                .context("failed to get contract")?
                .state
                .into();

            let balance = state
                .balance_of(&of)
                .map_err(|e| anyhow::anyhow!(e))
                .context("failed to fetch balance")?;
            println!("Balance of {}: {}", of, balance);
        }
        Commands::Transfer { from, to, amount } => {
            // Fetch the initial state from the node
            let initial_state: SimpleToken = client
                .get_contract(&contract_name.clone().into())
                .await
                .context("failed to get contract")?
                .state
                .into();
            // ----
            // Build the blob transaction
            // ----

            let action = SimpleTokenAction::Transfer {
                recipient: to.clone(),
                amount,
            };
            println!("Action: {:#?}", action);
            let blobs = vec![action.as_blob(contract_name.clone().into(), None, None)];
            let blob_tx = BlobTransaction::new(from.clone(), blobs.clone());

            println!("blob_tx: {:#?}", blob_tx);
            // Send the blob transaction
            let blob_tx_hash = client
                .send_tx_blob(&blob_tx)
                .await
                .context("cannot send tx")?;
            println!("✅ Blob tx sent. Tx hash: {}", blob_tx_hash);

            // ----
            // Prove the state transition
            // ----

            // Build the contract input
            let inputs = ContractInput {
                state: initial_state.as_bytes()?,
                identity: from.clone().into(),
                tx_hash: blob_tx_hash,
                private_input: vec![],
                tx_ctx: None,
                blobs: blobs.clone(),
                index: sdk::BlobIndex(0),
            };

            println!("inputs: {:#?}", inputs);

            // Generate the zk proof
            println!("🔍 Proving state transition...");
            let proof = prover.prove(inputs).await.unwrap();

            let proof_tx = ProofTransaction {
                proof,
                contract_name: contract_name.clone().into(),
            };

            // Send the proof transaction
            let proof_tx_hash = client
                .send_tx_proof(&proof_tx)
                .await
                .context("failed to send proof")?;
            println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);
        }
    };
    Ok(())
}
