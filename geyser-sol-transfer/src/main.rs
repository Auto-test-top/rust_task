use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tokio_stream::StreamExt;
use tonic::{metadata::MetadataValue, transport::Channel, Request};

pub mod geyser {
    include!(concat!(env!("OUT_DIR"), "/geyser.rs"));
}

use geyser::{
    geyser_client::GeyserClient, SubscribeRequest, SubscribeRequestFilterBlocks, SubscribeUpdate,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    geyser: GeyserConfig,
    solana: SolanaConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GeyserConfig {
    endpoint: String,
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SolanaConfig {
    rpc_url: String,
    private_key: String,
    recipient_address: String,
    transfer_amount: u64,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
    #[arg(long)]
    dry_run: bool,
}

struct SolTransfer {
    config: Config,
    rpc_client: RpcClient,
    keypair: Keypair,
    recipient_pubkey: Pubkey,
    dry_run: bool,
}

impl SolTransfer {
    fn new(config: Config, dry_run: bool) -> Result<Self> {
        let rpc_client = RpcClient::new_with_commitment(
            config.solana.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        let private_key_bytes = bs58::decode(&config.solana.private_key)
            .into_vec()
            .context("Err: format private key")?;

        let keypair = Keypair::from_bytes(&private_key_bytes)
            .context("Failed to create keypair from private key")?;

        let recipient_pubkey =
            Pubkey::from_str(&config.solana.recipient_address).context("Err: address recipient")?;

        println!(
            "Sender: {}, Recipient: {}",
            keypair.pubkey(),
            recipient_pubkey
        );

        Ok(Self {
            config,
            rpc_client,
            keypair,
            recipient_pubkey,
            dry_run,
        })
    }

    async fn send_sol_transfer(&self, block_slot: u64) -> Result<()> {
        if self.dry_run {
            println!(
                "DRY RUN: {} lamports -> {} {}",
                self.config.solana.transfer_amount, self.recipient_pubkey, block_slot
            );
            return Ok(());
        }

        let recent_blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .context(":: Failed to receive blockhash")?;

        let instruction = system_instruction::transfer(
            &self.keypair.pubkey(),
            &self.recipient_pubkey,
            self.config.solana.transfer_amount,
        );

        let message = Message::new(&[instruction], Some(&self.keypair.pubkey()));
        let mut transaction = Transaction::new_unsigned(message);
        transaction.sign(&[&self.keypair], recent_blockhash);

        match self.rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                println!("TX: {} {}", signature, block_slot);
            }
            Err(e) => {
                println!("Err: TX: {} {}", e, block_slot);
                return Err(e.into());
            }
        }

        Ok(())
    }
}

async fn setup_geyser_connection(config: &GeyserConfig) -> Result<GeyserClient<Channel>> {
    println!("Connect to {}", config.endpoint);

    let channel = Channel::from_shared(config.endpoint.clone())
        .context("Err: endpoint")?
        .connect()
        .await
        .context("Err: fail connect to geyser")?;

    let client = GeyserClient::new(channel);
    Ok(client)
}

async fn subscribe_to_blocks(
    sol_transfer: &SolTransfer,
    mut client: GeyserClient<Channel>,
) -> Result<()> {
    let mut blocks_filter = HashMap::new();
    blocks_filter.insert("client".to_string(), SubscribeRequestFilterBlocks {});

    let subscribe_request = SubscribeRequest {
        slots: HashMap::new(),
        accounts: HashMap::new(),
        transactions: HashMap::new(),
        blocks: blocks_filter,
        blocks_meta: HashMap::new(),
        accounts_data_slice: vec![],
        ping: None,
        commitment_level: 1,
    };

    let mut request = Request::new(subscribe_request);

    let api_key: MetadataValue<_> = sol_transfer
        .config
        .geyser
        .api_key
        .parse()
        .context("Err: incorrect API key")?;
    request.metadata_mut().insert("x-api-key", api_key);

    let mut stream = client
        .subscribe(request)
        .await
        .context("Err:  fail to subscribe geyser")?
        .into_inner();

    while let Some(update) = stream.next().await {
        match update {
            Ok(subscribe_update) => {
                if let Some(update_oneof) = subscribe_update.update_oneof {
                    match update_oneof {
                        geyser::subscribe_update::UpdateOneof::Block(block) => {
                            println!(
                                "Block {} {}, TX: {})",
                                block.slot, block.block_height, block.executed_transaction_count
                            );

                            if let Err(e) = sol_transfer.send_sol_transfer(block.slot).await {
                                println!("Err: send: {}", e);
                            }

                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                println!("Err stream: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn load_config(config_path: &PathBuf) -> Result<Config> {
    let config_content = fs::read_to_string(config_path)
        .with_context(|| format!("Err: read config: {:?}", config_path))?;

    let config: Config = serde_yaml::from_str(&config_content).context("Err: invalide YAML")?;

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = load_config(&args.config)?;

    let sol_transfer = SolTransfer::new(config.clone(), args.dry_run)?;

    loop {
        match setup_geyser_connection(&config.geyser).await {
            Ok(client) => {
                println!("Connect geyser");

                if let Err(e) = subscribe_to_blocks(&sol_transfer, client).await {
                    println!("Err: subscription: {}", e);
                }
            }
            Err(e) => {
                println!("Err: connect: {}", e);
            }
        }
    }
}
