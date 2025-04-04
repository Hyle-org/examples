use std::sync::Arc;

use anyhow::{anyhow, Result};
use client_sdk::{
    helpers::risc0::Risc0Prover,
    rest_client::{IndexerApiHttpClient, NodeApiHttpClient},
};
use contract_identity::IdentityContractState;
use hyle::{
    log_error,
    model::CommonRunContext,
    module_handle_messages,
    node_state::module::NodeStateEvent,
    utils::modules::{module_bus_client, Module},
};
use sdk::{
    BlobTransaction, BlobVec, Block, BlockHeight, ContractInput, ContractName, Hashed,
    HyleContract, ProofTransaction, TransactionData, TxHash, HYLE_TESTNET_CHAIN_ID,
};
use tracing::{error, info};

pub struct ProverModule {
    bus: ProverModuleBusClient,
    ctx: Arc<ProverModuleCtx>,
    unsettled_txs: Vec<BlobTransaction>,
    contract: IdentityContractState,
}

module_bus_client! {
#[derive(Debug)]
pub struct ProverModuleBusClient {
    receiver(NodeStateEvent),
}
}
pub struct ProverModuleCtx {
    pub common: Arc<CommonRunContext>,
    pub node_client: Arc<NodeApiHttpClient>,
    pub indexer_client: Arc<IndexerApiHttpClient>,
    pub start_height: BlockHeight,
    pub contract_name: ContractName,
}

impl Module for ProverModule {
    type Context = Arc<ProverModuleCtx>;

    async fn build(ctx: Self::Context) -> Result<Self> {
        let bus = ProverModuleBusClient::new_from_bus(ctx.common.bus.new_handle()).await;

        let contract = ctx
            .node_client
            .get_contract(&ctx.contract_name)
            .await?
            .state
            .into();

        Ok(ProverModule {
            bus,
            contract,
            ctx,
            unsettled_txs: vec![],
        })
    }

    async fn run(&mut self) -> Result<()> {
        module_handle_messages! {
            on_bus self.bus,
            listen<NodeStateEvent> event => {
                _ = log_error!(self.handle_node_state_event(event).await, "handle note state event")
            }

        };

        Ok(())
    }
}

impl ProverModule {
    async fn handle_node_state_event(&mut self, event: NodeStateEvent) -> Result<()> {
        let NodeStateEvent::NewBlock(block) = event;
        self.handle_processed_block(*block).await?;

        Ok(())
    }

    async fn handle_processed_block(&mut self, block: Block) -> Result<()> {
        for (_, tx) in block.txs {
            if let TransactionData::Blob(tx) = tx.transaction_data {
                let tx_ctx = sdk::TxContext {
                    block_height: block.block_height,
                    block_hash: block.hash.clone(),
                    timestamp: block.block_timestamp as u128,
                    lane_id: block.lane_ids.get(&tx.hashed()).unwrap().clone(),
                    chain_id: HYLE_TESTNET_CHAIN_ID,
                };

                self.handle_blob(tx, tx_ctx);
            }
        }

        for s_tx in block.successful_txs {
            self.settle_tx(s_tx)?;
        }

        for timedout in block.timed_out_txs {
            self.settle_tx(timedout)?;
        }

        for failed in block.failed_txs {
            self.settle_tx(failed)?;
        }

        Ok(())
    }

    fn handle_blob(&mut self, tx: BlobTransaction, tx_ctx: sdk::TxContext) {
        self.prove_identity_tx(tx.clone(), tx_ctx);
        self.unsettled_txs.push(tx);
    }

    fn settle_tx(&mut self, tx: TxHash) -> Result<usize> {
        let tx = self.unsettled_txs.iter().position(|t| t.hashed() == tx);
        if let Some(pos) = tx {
            self.unsettled_txs.remove(pos);
            Ok(pos)
        } else {
            Ok(0)
        }
    }

    fn prove_identity_tx(&mut self, tx: BlobTransaction, tx_ctx: sdk::TxContext) {
        if tx_ctx.block_height.0 < self.ctx.start_height.0 {
            return;
        }
        if let Some((index, blob)) = tx
            .blobs
            .iter()
            .enumerate()
            .find(|(_, b)| b.contract_name == self.ctx.contract_name)
        {
            let blobs = tx.blobs.clone();
            let tx_hash = tx.hashed();

            let prover = Risc0Prover::new(contract_identity::client::metadata::ELF);

            info!("Proving tx: {}. Blob for {}", tx_hash, blob.contract_name);

            let Ok(state) = self.contract.as_bytes() else {
                error!("Failed to serialize state on tx: {}", tx_hash);
                return;
            };

            let inputs = ContractInput {
                state,
                identity: tx.identity.clone(),
                tx_hash: tx_hash.clone(),
                private_input: vec![],
                blobs: BlobVec(blobs.clone()).into(),
                index: sdk::BlobIndex(index),
                tx_ctx: Some(tx_ctx),
                tx_blob_count: blobs.len(),
            };

            if let Err(e) = self.contract.execute(&inputs).map_err(|e| anyhow!(e)) {
                error!("error while executing contract: {e}");
            }

            let node_client = self.ctx.node_client.clone();
            let blob = blob.clone();
            tokio::task::spawn(async move {
                match prover.prove(inputs).await {
                    Ok(proof) => {
                        info!("Proof generated for tx: {}", tx_hash);
                        let tx = ProofTransaction {
                            contract_name: blob.contract_name.clone(),
                            proof,
                        };
                        let _ = log_error!(
                            node_client.send_tx_proof(&tx).await,
                            "failed to send proof to node"
                        );
                    }
                    Err(e) => {
                        error!("Error proving tx: {:?}", e);
                    }
                };
            });
        }
    }
}
