use anyhow::Result;
use clap::{Parser, Subcommand};
use client_sdk::helpers::risc0::Risc0Prover;
use contract::SimpleToken;
use contract::SimpleTokenAction;
use sdk::api::APIRegisterContract;
use sdk::BlobTransaction;
use sdk::ProofTransaction;
use sdk::{Calldata, ZkContract};

// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use methods::{GUEST_ELF, GUEST_ID};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value = "http://localhost:4321")]
    pub host: String,

    #[arg(long, default_value = "counter")]
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
async fn main() -> Result<()> {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
    let cli = Cli::parse();

    // Client to send requests to the node
    let client = client_sdk::rest_client::NodeApiHttpClient::new(cli.host)?;
    let contract_name = &cli.contract_name;

    // Will be used to generate zkProof of the execution.
    let prover = Risc0Prover::new(GUEST_ELF);

    match cli.command {
        Commands::Register { supply } => {
            // Build initial state of contract
            let initial_state =
                SimpleToken::new(supply, format!("faucet@{}", contract_name).into());
            println!("Initial state: {:?}", initial_state);

            // Send the transaction to register the contract
            let res = client
                .register_contract(&APIRegisterContract {
                    verifier: "risc0-1".into(),
                    program_id: sdk::ProgramId(sdk::to_u8_array(&GUEST_ID).to_vec()),
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
                .unwrap()
                .state
                .into();

            println!("Balances {:?}", &state);

            let balance = state.balance_of(&of).unwrap();
            println!("Balance of {}: {}", of, balance);
        }
        Commands::Transfer { from, to, amount } => {
            // Fetch the initial state from the node
            let mut initial_state: SimpleToken = client
                .get_contract(&contract_name.clone().into())
                .await
                .unwrap()
                .state
                .into();
            // ----
            // Build the blob transaction
            // ----

            let action = SimpleTokenAction::Transfer {
                recipient: to.clone(),
                amount,
            };
            let blobs = vec![sdk::Blob {
                contract_name: contract_name.clone().into(),
                data: sdk::BlobData(borsh::to_vec(&action).expect("failed to encode BlobData")),
            }];
            let blob_tx = BlobTransaction::new(from.clone(), blobs.clone());

            // Send the blob transaction
            let blob_tx_hash = client.send_tx_blob(&blob_tx).await.unwrap();
            println!("✅ Blob tx sent. Tx hash: {}", blob_tx_hash);

            // ----
            // Prove the state transition
            // ----

            // Build the contract input
            let inputs = Calldata {
                identity: from.clone().into(),
                tx_hash: blob_tx_hash,
                private_input: vec![],
                tx_ctx: None,
                blobs: blobs.clone().into(),
                tx_blob_count: blobs.len(),
                index: sdk::BlobIndex(0),
            };

            let res = initial_state.execute(&inputs).unwrap();
            println!("🚀 Executed: {}", res.0);

            // Generate the zk proof
            //
            let proof = prover.prove(initial_state.commit(), inputs).await.unwrap();

            // Build the Proof transaction
            let proof_tx = ProofTransaction {
                proof,
                contract_name: contract_name.clone().into(),
            };

            // Send the proof transaction
            let proof_tx_hash = client.send_tx_proof(&proof_tx).await.unwrap();
            println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);
        }
    }
    Ok(())
}
