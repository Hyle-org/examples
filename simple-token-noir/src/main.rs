use clap::{Parser, Subcommand};
use sdk::api::APIRegisterContract;
use sdk::{BlobTransaction, ProgramId, ProofData, ProofTransaction, StateCommitment, flatten_blobs};
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use toml::to_string_pretty;
use tracing::{error, info, warn};

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
    Prove { identity: String },
}

#[derive(serde::Serialize)]
struct ProverData {
    version: u32,
    initial_state: Vec<u8>,
    initial_state_len: u32,
    next_state: Vec<u8>,
    next_state_len: u32,
    identity: String,
    identity_len: u8,
    tx_hash: String,
    tx_hash_len: u32,
    index: u32,
    blobs: Vec<String>,
    blobs_len: u32,
    success: u32,
}

fn generate_prover_toml(
    identity: &str,
    tx_hash: &str,
    blobs: &[sdk::Blob],
) -> Result<String, Box<dyn std::error::Error>> {
    info!(
        "Generating Prover.toml with identity: {} and tx_hash: {}",
        identity, tx_hash
    );

    // Format blobs according to the parsing logic
    let mut formatted_blobs = Vec::new();
    
    // Add number of blobs
    formatted_blobs.push(format!("0x{:02x}", blobs.len()));
    
    // For each blob, add its size and data
    for blob in blobs {
        // Calculate total size (contract name + blob data)
        let contract_name_bytes = blob.contract_name.0.as_bytes();
        let total_size = contract_name_bytes.len() + blob.data.0.len();
        formatted_blobs.push(format!("0x{:02x}", total_size));
        
        // Add contract name bytes
        for &byte in contract_name_bytes {
            formatted_blobs.push(format!("0x{:02x}", byte));
        }
        
        // Add blob data bytes
        for &byte in &blob.data.0 {
            formatted_blobs.push(format!("0x{:02x}", byte));
        }
    }

    // Pad to 2800 bytes
    while formatted_blobs.len() < 2800 {
        formatted_blobs.push("0x00".to_string());
    }

    println!("Blob data {:?}", formatted_blobs);
    println!("Identity: {}", identity);
    println!("Tx hash: {}", tx_hash);

    let prover_data = ProverData {
        version: 1,
        initial_state: vec![0, 0, 0, 0],
        initial_state_len: 4,
        next_state: vec![0, 0, 0, 0],
        next_state_len: 4,
        identity: identity.to_string(),
        identity_len: identity.len() as u8,
        tx_hash: tx_hash.to_string(),
        tx_hash_len: tx_hash.len() as u32,
        index: 0,
        blobs: formatted_blobs.clone(),
        blobs_len: formatted_blobs.len() as u32,
        success: 1,
    };

    let toml = to_string_pretty(&prover_data)?;
    info!("Prover.toml generated successfully");
    Ok(toml)
}

fn generate_verification_key() -> Result<(), Box<dyn std::error::Error>> {
    info!("Generating verification key...");

    // Generate verification key
    info!("Generating verification key using bb...");
    let bb_vk_output = std::process::Command::new("bb")
        .arg("write_vk")
        .arg("--scheme")
        .arg("ultra_honk")
        .arg("-b")
        .arg("./contract/target/simple_token_noir.json")
        .arg("-o")
        .arg("./contract/target/vk")
        .output()?;

    if !bb_vk_output.status.success() {
        error!(
            "BB write_vk failed: {}",
            String::from_utf8_lossy(&bb_vk_output.stderr)
        );
        return Err(format!(
            "BB write_vk failed: {}",
            String::from_utf8_lossy(&bb_vk_output.stderr)
        )
        .into());
    }

    info!("Verification key generated successfully");

     Ok(())
}

fn execute_proof_generation(
    identity: &str,
    tx_hash: &str,
    blobs: &[sdk::Blob],
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Starting proof generation for identity: {} and tx_hash: {}",
        identity, tx_hash
    );

    // Store current directory (nargo needs to be executed in the contract directory i believe)
    let current_dir = env::current_dir()?;
    info!("Current directory: {}", current_dir.display());

    // Change to contract directory
    let contract_dir = current_dir.join("contract");
    info!("Changing to contract directory: {}", contract_dir.display());
    env::set_current_dir(&contract_dir)?;

    // Create Prover.toml
    info!("Creating Prover.toml...");
    let prover_toml = generate_prover_toml(identity, tx_hash, blobs)?;
    fs::write("Prover.toml", &prover_toml)?;
    info!("Prover.toml created successfully");

    // Execute nargo
    info!("Executing nargo...");
    let nargo_output = std::process::Command::new("nargo")
        .arg("execute")
        .arg("-p")
        .arg("Prover.toml")
        .output()?;

    if !nargo_output.status.success() {
        error!(
            "Nargo execution failed: {}",
            String::from_utf8_lossy(&nargo_output.stderr)
        );
        return Err(format!(
            "Nargo execution failed: {}",
            String::from_utf8_lossy(&nargo_output.stderr)
        )
        .into());
    }
    info!("Nargo execution completed successfully");

    // Generate proof using bb
    info!("Generating proof using bb...");
    let bb_prove_output = std::process::Command::new("bb")
        .arg("prove")
        .arg("--scheme")
        .arg("ultra_honk")
        .arg("-b")
        .arg("target/simple_token_noir.json")
        .arg("-w")
        .arg("target/simple_token_noir.gz")
        .arg("-o")
        .arg("./target/proof")
        .output()?;

    if !bb_prove_output.status.success() {
        error!(
            "BB prove failed: {}",
            String::from_utf8_lossy(&bb_prove_output.stderr)
        );
        return Err(format!(
            "BB prove failed: {}",
            String::from_utf8_lossy(&bb_prove_output.stderr)
        )
        .into());
    }
    info!("Proof generated successfully");

    // Change back to original directory
    env::set_current_dir(&current_dir)?;
    info!(
        "Changed back to original directory: {}",
        current_dir.display()
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with more detailed format
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true)
        .init();

    info!("Starting simple-token-noir application");
    let cli = Cli::parse();

    let client = client_sdk::rest_client::NodeApiHttpClient::new(cli.host)
        .map_err(|e| format!("Failed to create client: {}", e))?;
    let contract_name = &cli.contract_name;

    match cli.command {
        Commands::Register => {
            info!("Starting contract registration process");
            println!("Generating verification key...");
            generate_verification_key()?;
            println!("✅ Verification key generated successfully");

            // Read the verification key
            info!("Reading verification key from file");
            let vk = fs::read("./contract/target/vk")?;
            info!(
                "Verification key read successfully, size: {} bytes",
                vk.len()
            );

            // Send the transaction to register the contract
            info!("Sending contract registration transaction");
            let res = client
                .register_contract(&APIRegisterContract {
                    verifier: "noir".into(),
                    program_id: ProgramId(vk),
                    state_commitment: StateCommitment(vec![0;4]),
                    contract_name: contract_name.clone().into(),
                })
                .await
                .map_err(|e| format!("Failed to register contract: {}", e))?;
            println!("✅ Register contract tx sent. Tx hash: {}", res);
            info!("Contract registration completed successfully");
        }
        Commands::Prove { identity } => {
            info!("Starting contract execution process");
            // Extract just the value part after the equals sign
            let identity_value = if identity.starts_with("--identity=") {
                identity.trim_start_matches("--identity=")
            } else {
                &identity
            };

            info!(
                "Processing identity: {} (length: {})",
                identity_value,
                identity_value.len()
            );

            // TODO: change identity length to a bigger one, and pad it with 0s like blobs
            if identity_value.len() != 24 {
                error!("Invalid identity length: {}", identity_value.len());
                return Err("Identity must be exactly 24 characters long".into());
            }

            // Create an empty blob
            info!("Creating blob transaction");
            let blob_data = vec![0; 4];
            let blob = sdk::Blob {
                contract_name: contract_name.clone().into(),
                data: sdk::BlobData(blob_data.clone()),
            };
            let blobs = vec![blob.clone()];
            let blob_tx = BlobTransaction::new(identity_value.to_string(), blobs.clone());

            // Send the blob transaction
            info!("Sending blob transaction");
            let blob_tx_hash = client
                .send_tx_blob(&blob_tx)
                .await
                .map_err(|e| format!("Failed to send blob transaction: {}", e))?;
            println!("✅ Blob tx sent. Tx hash: {}", blob_tx_hash);
            info!("Blob transaction sent successfully");

            // Create a padded version of the blob for proof generation
           
            info!("Starting proof generation with blob tx hash");
            println!("Generating proof...");
            execute_proof_generation(identity_value, &blob_tx_hash.0, &blobs)?;
            println!("✅ Proof generated successfully");

            // Read the proof
            info!("Reading generated proof from file");
            let proof = fs::read("./contract/target/proof")?;
            info!("Proof read successfully, size: {} bytes", proof.len());

            // Create proof transaction with the generated proof
            info!("Creating proof transaction");
            let proof_tx = ProofTransaction {
                proof: ProofData(proof),
                contract_name: contract_name.clone().into(),
            };

            // Send the proof transaction
            info!("Sending proof transaction");
            let proof_tx_hash = client
                .send_tx_proof(&proof_tx)
                .await
                .map_err(|e| format!("Failed to send proof transaction: {}", e))?;
            println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);
            info!("Proof transaction sent successfully");
        }
        
    }
    info!("Application completed successfully");
    Ok(())
}
