use anyhow::Context;
use clap::{Parser, Subcommand};
use contract::Token;
use contract::TokenContract;
use hyle::model::BlobTransaction;
use hyle::model::ProofTransaction;
use hyle::model::RegisterContractTransaction;
use sdk::erc20::ERC20;
use sdk::ContractAction;
use sdk::{ContractInput, Digestable};

use sp1_sdk::{include_elf, ProverClient};

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

    let client = hyle::tools::rest_api_client::NodeApiHttpClient::new(cli.host);

    let contract_name = &cli.contract_name;

    match cli.command {
        Commands::Register { supply } => {
            // Build initial state of contract
            let initial_state = Token::new(supply, format!("faucet.{}", contract_name));
            println!("Initial state: {:?}", initial_state);

            let prover_client = ProverClient::from_env();
            let (_, vk) = prover_client.setup(CONTRACT_ELF);

            let vk = serde_json::to_vec(&vk).unwrap();

            // Send the transaction to register the contract
            let register_tx = RegisterContractTransaction {
                owner: "examples".to_string(),
                verifier: "sp1".into(),
                program_id: sdk::ProgramId(vk),
                state_digest: initial_state.as_digest(),
                contract_name: contract_name.clone().into(),
            };
            let res = client
                .send_tx_register_contract(&register_tx)
                .await
                .context("failed to send tx")?
                .text()
                .await
                .context("failed to parse response")?;

            println!("✅ Register contract tx sent. Tx hash: {}", res);
        }
        Commands::Balance { of } => {
            // Fetch the state from the node
            let state: Token = client
                .get_contract(&contract_name.clone().into())
                .await
                .context("failed to get contract")?
                .state
                .into();

            let contract = TokenContract::init(state, "".into());
            let balance = contract
                .balance_of(&of)
                .map_err(|e| anyhow::anyhow!(e))
                .context("failed to fetch balance")?;
            println!("Balance of {}: {}", of, balance);
        }
        Commands::Transfer { from, to, amount } => {
            // Fetch the initial state from the node
            let initial_state: Token = client
                .get_contract(&contract_name.clone().into())
                .await
                .context("failed to get contract")?
                .state
                .into();
            // ----
            // Build the blob transaction
            // ----

            let action = sdk::erc20::ERC20Action::Transfer {
                recipient: to.clone(),
                amount,
            };
            println!("Action: {:#?}", action);
            let blobs = vec![action.as_blob(contract_name.clone().into(), None, None)];
            let blob_tx = BlobTransaction {
                identity: from.clone().into(),
                blobs: blobs.clone(),
            };

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
                initial_state: initial_state.as_digest(),
                identity: from.clone().into(),
                tx_hash: "".into(),
                private_blob: sdk::BlobData(vec![]),
                blobs: blobs.clone(),
                index: sdk::BlobIndex(0),
            };

            println!("inputs: {:#?}", inputs);

            // Generate the zk proof
            println!("🔍 Proving state transition...");
            let (proof, _) = client_sdk::helpers::sp1::prove(CONTRACT_ELF, &inputs)
                .await
                .context("failed to prove")?;

            let proof_tx = ProofTransaction {
                tx_hashes: vec![blob_tx_hash],
                proof,
                contract_name: contract_name.clone().into(),
            };

            println!("proof_tx: {:#?}", proof_tx);
            serde_json::to_writer(std::fs::File::create("proof_tx.json")?, &proof_tx)?;

            // Send the proof transaction
            let proof_tx_hash = client
                .send_tx_proof(&proof_tx)
                .await
                .context("failed to send proof")?
                .text()
                .await
                .context("failed to parse response")?;
            println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);
        }
    };
    Ok(())
}

//pub async fn prove(
//    binary: &[u8],
//    contract_input: &ContractInput,
//) -> anyhow::Result<(ProofData, HyleOutput)> {
//    let client = ProverClient::from_env();
//
//    // Setup the inputs.
//    let mut stdin = SP1Stdin::new();
//    stdin.write(&contract_input);
//
//    // Setup the program for proving.
//    let (pk, _vk) = client.setup(binary);
//
//    // Generate the proof
//    let proof = client
//        .prove(&pk, &stdin)
//        //.compressed()
//        .run()
//        .expect("failed to generate proof");
//
//    let hyle_output = bincode::deserialize::<HyleOutput>(proof.public_values.as_slice())
//        .context("Failed to extract HyleOuput from SP1 proof")?;
//
//    if !hyle_output.success {
//        let program_error = std::str::from_utf8(&hyle_output.program_outputs).unwrap();
//        anyhow::bail!(
//            "\x1b[91mExecution failed ! Program output: {}\x1b[0m",
//            program_error
//        );
//    }
//
//    let encoded_receipt = bincode::serialize(&proof)?;
//    Ok((ProofData::Bytes(encoded_receipt), hyle_output))
//}
