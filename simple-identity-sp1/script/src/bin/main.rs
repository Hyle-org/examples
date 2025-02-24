use clap::{Parser, Subcommand};
use contract::IdentityContractState;
use sdk::api::APIRegisterContract;
use sdk::BlobTransaction;
use sdk::ProofTransaction;
use sdk::{ContractInput, Digestable};

use sp1_sdk::{include_elf, ProverClient};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const CONTRACT_ELF: &[u8] = include_elf!("simple_identity");

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value = "http://localhost:4321")]
    pub host: String,

    #[arg(long, default_value = "simple_identity")]
    pub contract_name: String,
}

#[derive(Subcommand)]
enum Commands {
    RegisterContract {},
    RegisterIdentity {
        identity: String,
        password: String,
    },
    VerifyIdentity {
        identity: String,
        password: String,
        nonce: u32,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let client = client_sdk::rest_client::NodeApiHttpClient::new(cli.host).unwrap();
    let contract_name = &cli.contract_name;

    match cli.command {
        Commands::RegisterContract {} => {
            // Build initial state of contract
            let initial_state = IdentityContractState::new();
            println!("Initial state: {:?}", initial_state);

            let prover_client = ProverClient::from_env();
            let (_, vk) = prover_client.setup(CONTRACT_ELF);
            let vk = serde_json::to_vec(&vk).unwrap();

            // Send the transaction to register the contract
            let register_tx = APIRegisterContract {
                verifier: "sp1".into(),
                program_id: sdk::ProgramId(vk),
                state_digest: initial_state.as_digest(),
                contract_name: contract_name.clone().into(),
            };
            let res = client.register_contract(&register_tx).await.unwrap();

            println!("✅ Register contract tx sent. Tx hash: {}", res);
        }
        Commands::RegisterIdentity { identity, password } => {
            // Fetch the initial state from the node
            let initial_state: IdentityContractState = client
                .get_contract(&contract_name.clone().into())
                .await
                .unwrap()
                .state
                .into();

            println!("Initial state {:?}", initial_state.clone());
            println!("Identity {:?}", identity.clone());

            // Build the blob transaction
            let action = sdk::identity_provider::IdentityAction::RegisterIdentity {
                account: identity.clone(),
            };

            let blobs = vec![sdk::Blob {
                contract_name: contract_name.clone().into(),
                data: sdk::BlobData(borsh::to_vec(&action).expect("failed to encode BlobData")),
            }];
            let blob_tx = BlobTransaction::new(identity, blobs.clone());

            println!("blob_tx: {:#?}", blob_tx);
            // Send the blob transaction
            let blob_tx_hash = client.send_tx_blob(&blob_tx).await.unwrap();
            println!("✅ Blob tx sent. Tx hash: {}", blob_tx_hash);

            // Add delay to ensure blob transaction is processed
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Build the contract input
            let inputs = ContractInput {
                state: initial_state.as_bytes().unwrap(),
                identity: blob_tx.identity.clone(),
                tx_hash: blob_tx_hash,
                private_input: password.into_bytes().to_vec(),
                blobs: blobs.clone(),
                index: sdk::BlobIndex(0),
                tx_ctx: None,
            };

            println!("inputs: {:#?}", inputs);

            // Generate the zk proof
            println!("🔍 Proving state transition...");
            let (proof, _) = client_sdk::helpers::sp1::prove(CONTRACT_ELF, &inputs).unwrap();

            let proof_tx = ProofTransaction {
                proof,
                contract_name: contract_name.clone().into(),
            };

            // Add delay before sending proof transaction
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // Send the proof transaction
            let proof_tx_hash = client.send_tx_proof(&proof_tx).await.unwrap();
            println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);
        }
        Commands::VerifyIdentity {
            identity,
            password,
            nonce,
        } => {
            // Fetch the initial state from the node
            let initial_state: IdentityContractState = client
                .get_contract(&contract_name.clone().into())
                .await
                .unwrap()
                .state
                .into();

            // Build the blob transaction
            let action = sdk::identity_provider::IdentityAction::VerifyIdentity {
                account: identity.clone(),
                nonce,
            };

            let blobs = vec![sdk::Blob {
                contract_name: contract_name.clone().into(),
                data: sdk::BlobData(borsh::to_vec(&action).expect("failed to encode BlobData")),
            }];
            let blob_tx = BlobTransaction::new(identity.clone(), blobs.clone());

            println!("blob_tx: {:#?}", blob_tx);
            // Send the blob transaction
            let blob_tx_hash = client.send_tx_blob(&blob_tx).await.unwrap();
            println!("✅ Blob tx sent. Tx hash: {}", blob_tx_hash);

            // Add delay to ensure blob transaction is processed
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Build the contract input
            let inputs = ContractInput {
                state: initial_state.as_bytes().unwrap(),
                identity: blob_tx.identity.clone(),
                tx_hash: blob_tx_hash,
                private_input: password.into_bytes().to_vec(),
                blobs: blobs.clone(),
                index: sdk::BlobIndex(0),
                tx_ctx: None,
            };

            println!("inputs: {:#?}", inputs);

            // Generate the zk proof
            println!("🔍 Proving state transition...");
            let (proof, _) = client_sdk::helpers::sp1::prove(CONTRACT_ELF, &inputs).unwrap();

            let proof_tx = ProofTransaction {
                proof,
                contract_name: contract_name.clone().into(),
            };

            // Add delay before sending proof transaction
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // Send the proof transaction
            let proof_tx_hash = client.send_tx_proof(&proof_tx).await.unwrap();
            println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);
        }
    }
    Ok(())
}
