use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct DepositAccount {
    pub balance: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum ProgramInstruction {
    Deposit { amount: u64 },
    Withdraw { amount: u64 },
}

pub struct Client {
    rpc_client: RpcClient,
    program_id: Pubkey,
    payer: Keypair,
}

impl Client {
    pub fn new(
        rpc_url: &str,
        program_id: &str,
        keypair_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let program_id = Pubkey::from_str(program_id)
            .map_err(|e| format!("Err: parsing Program ID '{}': {}", program_id, e))?;

        let keypair_data = std::fs::read_to_string(keypair_path)
            .map_err(|e| format!("Err: read file '{}': {}", keypair_path, e))?;

        let keypair_bytes: Vec<u8> = serde_json::from_str(&keypair_data)
            .map_err(|e| format!("Err: parsing JSON keypair: {}", e))?;

        let payer = Keypair::from_bytes(&keypair_bytes)
            .map_err(|e| format!("Err: create keypair: {}", e))?;

        Ok(Self {
            rpc_client,
            program_id,
            payer,
        })
    }

    pub async fn deposit(
        &self,
        deposit_account: &str,
        amount_sol: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let amount = (amount_sol * 1_000_000_000.0) as u64;

        let deposit_pubkey = Pubkey::from_str(deposit_account).map_err(|e| {
            format!(
                "Err: parsing pb key {} : {}",
                deposit_account, e
            )
        })?;

        let mut instruction_data = vec![0u8];
        instruction_data.extend_from_slice(&amount.to_le_bytes());

        let instruction = Instruction::new(
            self.program_id,
            &instruction_data,
            vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(deposit_pubkey, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            blockhash,
        );

        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        println!("Dep compilte: {}", signature);
        Ok(())
    }

    pub async fn withdraw(
        &self,
        deposit_account: &str,
        amount_sol: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let amount = (amount_sol * 1_000_000_000.0) as u64;

        let deposit_pubkey = Pubkey::from_str(deposit_account).map_err(|e| {
            format!(
                "Err: parsing pb key {}: {}",
                deposit_account, e
            )
        })?;

        let mut instruction_data = vec![1u8];
        instruction_data.extend_from_slice(&amount.to_le_bytes());

        let instruction = Instruction::new(
            self.program_id,
            &instruction_data,
            vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(deposit_pubkey, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            blockhash,
        );

        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        println!("Withdrawal completed: {}", signature);
        Ok(())
    }

    pub async fn get_balance(
        &self,
        deposit_account: &str,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        println!("Parsing deposit account: {}", deposit_account);
        let deposit_pubkey = Pubkey::from_str(deposit_account)
            .map_err(|e| format!("Err: parsing pb key {}: {}", deposit_account, e))?;

        let balance_lamports = self.rpc_client.get_balance(&deposit_pubkey)?;
        let balance_sol = balance_lamports as f64 / 1_000_000_000.0;

        println!("Balance: {} SOL ({} lapms)", balance_sol, balance_lamports);
        Ok(balance_sol)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 6 {
        println!(
            "  {} <rpc_url> <program_id> <keypair_path> deposit <deposit_account> <amount>",
            args[0]
        );
        println!(
            "  {} <rpc_url> <program_id> <keypair_path> withdraw <deposit_account> <amount>",
            args[0]
        );
        println!(
            "  {} <rpc_url> <program_id> <keypair_path> balance <deposit_account>",
            args[0]
        );
        return Ok(());
    }

    let client = Client::new(&args[1], &args[2], &args[3])?;

    match args[4].as_str() {
        "deposit" => {
            let amount: f64 = args[6].parse()?;
            client.deposit(&args[5], amount).await?;
        }
        "withdraw" => {
            let amount: f64 = args[6].parse()?;
            client.withdraw(&args[5], amount).await?;
        }
        "balance" => {
            client.get_balance(&args[5]).await?;
        }
        _ => println!("Err: args 404 check client-main: {}", args[4]),
    }

    Ok(())
}
