use clap::{Parser, Subcommand};
use sdk::api::APIRegisterContract;
use sdk::{
    flatten_blobs, BlobTransaction, ProgramId, ProofData, ProofTransaction, StateCommitment,
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
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
const CIRCUIT_JSON: &str = include_str!("../contract/target/check_secret.json");
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

    #[arg(long, default_value = "check_secret")]
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
    blob_number: u32,
    blob_index: u32,
    blob_contract_name_len: u32,
    blob_contract_name: String,
    blob_len: u32,
    blob: Vec<u8>,
    tx_blob_count: u32,
    success: u32,
    password: Vec<u8>,
}

fn generate_prover_toml(
    identity: &str,
    tx_hash: &str,
    blobs: &[sdk::Blob],
    hashed_password: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    info!(
        "Generating Prover.toml with identity: {} and tx_hash: {}",
        identity, tx_hash
    );

    println!("Identity: {}", identity);
    println!("Tx hash: {}", tx_hash);

    let prover_data = ProverData {
        version: 1,
        initial_state: vec![0, 0, 0, 0],
        initial_state_len: 4,
        next_state: vec![0, 0, 0, 0],
        next_state_len: 4,
        identity: format!("{:0<64}", identity),
        identity_len: identity.len() as u8,
        tx_hash: tx_hash.to_string(),
        tx_hash_len: tx_hash.len() as u32,
        index: 0,
        blob_number: 1,
        blob_index: 0,
        blob_contract_name_len: blobs[0].contract_name.0.len() as u32,
        blob_contract_name: format!("{:0<64}", blobs[0].contract_name.0.clone()),
        blob_len: blobs[0].data.0.len() as u32,
        blob: blobs[0].data.0.clone(),
        tx_blob_count: blobs.len() as u32,
        success: 1,
        password: hashed_password.to_vec(),
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
        .arg("./contract/target/check_secret.json")
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
    hashed_password: &[u8],
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
    let prover_toml = generate_prover_toml(identity, tx_hash, blobs, hashed_password)?;
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
        .arg("target/check_secret.json")
        .arg("-w")
        .arg("target/check_secret.gz")
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
                    state_commitment: StateCommitment(vec![0; 4]),
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

            let password = "password"; // TODO: replace with actual password

            // hash password
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            let hashed_password = hasher.finalize();
            let hashed_password = hashed_password.as_slice();

            println!("Hashed password: {:?}", hashed_password);
            let id = format!("{:0<64}:", identity_value);
            let mut id = id.as_bytes().to_vec();
            id.extend_from_slice(hashed_password);

            println!("Extended ID: {:?}", id);
            println!("Extended ID (hex): {:?}", hex::encode(&id));

            let mut hasher = Sha256::new();
            hasher.update(id);
            let hash_bytes = hasher.finalize();
            let hashed_id = hash_bytes.to_vec();

            println!("Hashed ID: {:?}", hashed_id);
            println!("Hashed ID (hex): {:?}", hex::encode(&hashed_id));

            // Create an empty blob
            info!("Creating blob transaction");
            let blob_data = hashed_id;
            info!("Blob data: {:?} (len: {})", blob_data, blob_data.len());
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
            execute_proof_generation(identity_value, &blob_tx_hash.0, &blobs, hashed_password)?;
            println!("✅ Proof generated successfully");

            // Read the proof
            info!("Reading generated proof from file");
            let proof = fs::read("./contract/target/proof")?;
            info!("Proof read successfully, size: {} bytes", proof.len());
            info!("Proof data: {:?}", proof);

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
