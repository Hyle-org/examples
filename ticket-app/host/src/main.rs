use clap::{Parser, Subcommand};

use client_sdk::helpers::risc0::Risc0Prover;
use contract::SimpleToken;
use contract::SimpleTokenAction;
use contract_identity::IdentityAction;
use contract_ticket_app::TicketAppAction;
use contract_ticket_app::TicketAppState;
use sdk::api::APIRegisterContract;
use sdk::BlobTransaction;
use sdk::Identity;
use sdk::ProofTransaction;
use sdk::{ContractInput, ContractName, HyleContract};

// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use methods_ticket_app::GUEST_ID;

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

    #[arg(long, default_value = "simple_ticket_app")]
    pub contract_name: String,

    #[arg(long, default_value = "examples.simple_ticket_app")]
    pub user: String,

    #[arg(long, default_value = "pass")]
    pub pass: String,

    #[arg(long, default_value = "0")]
    pub nonce: String,
}

#[derive(Subcommand)]
enum Commands {
    Register { token: String, price: u128 },
    BuyTicket {},
    HasTicket {},
}

#[tokio::main]
async fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let client = client_sdk::rest_client::NodeApiHttpClient::new(cli.host).unwrap();

    let contract_name = &cli.contract_name.clone();

    let ticket_prover = Risc0Prover::new(methods_ticket_app::GUEST_ELF);
    let identity_prover = Risc0Prover::new(methods_identity::GUEST_ELF);
    let token_prover = Risc0Prover::new(methods::GUEST_ELF);

    match cli.command {
        Commands::Register { token, price } => {
            // Build initial state of contract
            let initial_state = TicketAppState::new(vec![], (ContractName(token), price));
            println!("Initial state: {:?}", initial_state);
            println!("Initial State {:?}", initial_state.commit());

            // Send the transaction to register the contract
            let res = client
                .register_contract(&APIRegisterContract {
                    verifier: "risc0-1".into(),
                    program_id: sdk::ProgramId(sdk::to_u8_array(&GUEST_ID).to_vec()),
                    state_commitment: initial_state.commit(),
                    contract_name: contract_name.clone().into(),
                })
                .await
                .unwrap();

            println!("✅ Register contract tx sent. Tx hash: {}", res);
        }
        Commands::BuyTicket {} => {
            // Build initial state of contract
            let initial_state: TicketAppState = client
                .get_contract(&contract_name.clone().into())
                .await
                .unwrap()
                .state
                .into();

            println!("Initial State {:?}", &initial_state);
            println!("Initial State {:?}", initial_state.commit());
            println!("Identity {:?}", cli.user.clone());
            println!("Nonce {:?}", cli.nonce.clone());

            let identity = Identity(cli.user.clone());

            let identity_cf: IdentityAction = IdentityAction::VerifyIdentity {
                account: identity.0.clone(),
                nonce: cli.nonce.parse().unwrap(),
            };

            let identity_contract_name = cli.user.rsplit_once(".").unwrap().1.to_string();

            let blobs = vec![
                sdk::Blob {
                    contract_name: identity_contract_name.clone().into(),
                    data: sdk::BlobData(
                        borsh::to_vec(&identity_cf).expect("Failed to encode Identity action"),
                    ),
                },
                // Init pair 0 amount
                sdk::Blob {
                    contract_name: initial_state.ticket_price.0.clone(),
                    data: sdk::BlobData(
                        borsh::to_vec(&SimpleTokenAction::Transfer {
                            recipient: contract_name.clone(),
                            amount: initial_state.ticket_price.1,
                        })
                        .expect("Failed to encode Erc20 transfer action"),
                    ),
                },
                sdk::Blob {
                    contract_name: contract_name.clone().into(),
                    data: sdk::BlobData(
                        borsh::to_vec(&TicketAppAction::BuyTicket {})
                            .expect("Failed to encode Buy Ticket action"),
                    ),
                },
            ];

            println!("Blobs {:?}", blobs.clone());

            let blob_tx = BlobTransaction::new(identity.clone(), blobs.clone());

            // Send the blob transaction
            let blob_tx_hash = client.send_tx_blob(&blob_tx).await.unwrap();
            println!("✅ Blob tx sent. Tx hash: {}", blob_tx_hash);

            // prove tx

            println!("Running and proving TicketApp blob");

            // Build the contract input
            let inputs = ContractInput {
                state: initial_state.as_bytes().unwrap(),
                identity: identity.clone(),
                tx_hash: blob_tx_hash.clone().into(),
                private_input: vec![],
                tx_ctx: None,
                blobs: blobs.clone(),
                index: sdk::BlobIndex(2),
            };

            // Generate the zk proof
            //
            let proof = ticket_prover.prove(inputs).await.unwrap();

            let proof_tx = ProofTransaction {
                proof,
                contract_name: contract_name.clone().into(),
            };

            // Send the proof transaction
            let proof_tx_hash = client.send_tx_proof(&proof_tx).await.unwrap();
            println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);

            println!("Running and proving Transfer blob");

            // Build the transfer a input
            let initial_state_a: SimpleToken = client
                .get_contract(&initial_state.ticket_price.0.clone().into())
                .await
                .unwrap()
                .state
                .into();

            let inputs = ContractInput {
                state: initial_state_a.as_bytes().unwrap(),
                identity: identity.clone(),
                tx_hash: blob_tx_hash.clone().into(),
                private_input: vec![],
                tx_ctx: None,
                blobs: blobs.clone(),
                index: sdk::BlobIndex(1),
            };

            // Generate the zk proof
            let proof = token_prover.prove(inputs).await.unwrap();

            let proof_tx = ProofTransaction {
                proof,
                contract_name: initial_state.ticket_price.0.clone(),
            };

            // Send the proof transaction
            let proof_tx_hash = client.send_tx_proof(&proof_tx).await.unwrap();
            println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);

            println!("Running and proving Identity blob");

            // Fetch the initial state from the node
            let initial_state_id: contract_identity::IdentityContractState = client
                .get_contract(&identity_contract_name.clone().into())
                .await
                .unwrap()
                .state
                .into();

            // Build the contract input
            let inputs = ContractInput {
                state: initial_state_id.as_bytes().unwrap(),
                identity: identity.clone(),
                tx_hash: blob_tx_hash.clone().into(),
                private_input: cli.pass.into_bytes().to_vec(),
                tx_ctx: None,
                blobs: blobs.clone(),
                index: sdk::BlobIndex(0),
            };

            // Generate the zk proof
            let proof = identity_prover.prove(inputs).await.unwrap();

            let proof_tx = ProofTransaction {
                proof,
                contract_name: identity_contract_name.clone().into(),
            };

            // Send the proof transaction
            let proof_tx_hash = client.send_tx_proof(&proof_tx).await.unwrap();
            println!("✅ Proof tx sent. Tx hash: {}", proof_tx_hash);
        }
        Commands::HasTicket {} => {
            let initial_state: TicketAppState = client
                .get_contract(&contract_name.clone().into())
                .await
                .unwrap()
                .state
                .into();

            println!("Initial State {:?}", &initial_state);
            println!("Initial State {:?}", initial_state.commit());

            if initial_state.tickets.contains(&Identity(cli.user.clone())) {
                println!("{} has a ticket", cli.user);
            } else {
                println!("{} has no ticket", cli.user);
            }
        }
    }
}
