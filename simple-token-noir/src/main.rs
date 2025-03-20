use clap::{Parser, Subcommand};
use noir_rs::{
    barretenberg::{
        prove::prove_ultra_honk,
        srs::{setup_srs_from_bytecode},
        verify::verify_ultra_honk,
        utils::get_honk_verification_key,
    },
    witness::from_vec_str_to_witness_map,
};
use sdk::api::APIRegisterContract;
use sdk::{StateCommitment, ProgramId, BlobTransaction, ProofTransaction, ProofData};
use tracing::info;
use serde::Deserialize;

#[derive(Deserialize)]
struct NoirCircuit {
    bytecode: String,
}

// Load and parse the circuit JSON to get the bytecode
const CIRCUIT_JSON: &str = include_str!("../contract/target/simple_token_noir.json");
lazy_static::lazy_static! {
    static ref BYTECODE: String = {
        let circuit: NoirCircuit = serde_json::from_str(CIRCUIT_JSON)
            .expect("Failed to parse circuit JSON");
        circuit.bytecode
    };
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[clap(long, short)]
    reproducible: bool,

    #[arg(long, default_value = "http://localhost:4321")]
    pub host: String,

    #[arg(long, default_value = "simple_token_noir")]
    pub contract_name: String,
}

#[derive(Subcommand)]
enum Commands {
    Register,
    Execute {
        identity: String,
    },
}

#[derive(borsh::BorshSerialize)]
enum SimpleTokenAction {
    Transfer {
        recipient: String,
        amount: u128,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let client = client_sdk::rest_client::NodeApiHttpClient::new(cli.host)
        .map_err(|e| format!("Failed to create client: {}", e))?;
    let contract_name = &cli.contract_name;

    // Setup Noir prover
    setup_srs_from_bytecode(BYTECODE.as_str(), None, false)
        .map_err(|e| format!("Failed to setup SRS: {}", e))?;

    match cli.command {
        Commands::Register => {
            // Get verification key for the circuit
            let vk = get_honk_verification_key(BYTECODE.as_str(), false)
                .map_err(|e| format!("Failed to get verification key: {}", e))?;

            // Send the transaction to register the contract
            let res = client
                .register_contract(&APIRegisterContract {
                    verifier: "noir-1".into(),
                    program_id: ProgramId(vk.to_vec()), // Use verification key as program ID
                    state_commitment: StateCommitment::default(), // Empty state since contract is stateless
                    contract_name: contract_name.clone().into(),
                })
                .await
                .map_err(|e| format!("Failed to register contract: {}", e))?;
            println!("✅ Register contract tx sent. Tx hash: {}", res);        }
        Commands::Execute { identity } => {
            // Create an empty blob
            let blobs = vec![sdk::Blob {
                contract_name: contract_name.clone().into(),
                data: sdk::BlobData(vec![]),
            }];
            let blob_tx = BlobTransaction::new(identity.clone(), blobs.clone());

            // Send the blob transaction
            let blob_tx_hash = client.send_tx_blob(&blob_tx).await
                .map_err(|e| format!("Failed to send blob transaction: {}", e))?;
            println!("✅ Blob tx sent. Tx hash: {}", blob_tx_hash);

            // Create witness for the proof
            let witness = from_vec_str_to_witness_map(vec![
                "1", // version
                "4", // initial_state_len
                "0,0,0,0", // initial_state
                "4", // next_state_len
                "0,0,0,0", // next_state
                "24", // identity_len
                &identity, // identity
                "0", // tx_hash_len
                "", // tx_hash (empty)
                "0", // index
                "0", // blobs_len
                "", // blobs (empty)
                "1", // success
            ]).map_err(|e| format!("Failed to create witness: {}", e))?;

            // Generate the proof
            let start = std::time::Instant::now();
            let proof = prove_ultra_honk(BYTECODE.as_str(), witness, false)
                .map_err(|e| format!("Failed to generate proof: {}", e))?;
            info!("Proof generation time: {:?}", start.elapsed());

            // Get verification key and verify the proof
            let vk = get_honk_verification_key(BYTECODE.as_str(), false)
                .map_err(|e| format!("Failed to get verification key: {}", e))?;
            let verdict = verify_ultra_honk(proof.clone(), vk)
                .map_err(|e| format!("Failed to verify proof: {}", e))?;
            info!("Proof verification verdict: {}", verdict);

            let proof_tx = ProofTransaction {
                proof: ProofData(proof.to_vec()),
                contract_name: contract_name.clone().into(),
            };

            // Send the proof transaction
            let proof_tx_hash = client.send_tx_proof(&proof_tx).await
                .map_err(|e| format!("Failed to send proof transaction: {}", e))?;
            println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);
        }
    }
    Ok(())
}
