use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let proto_content = r#"
syntax = "proto3";

package geyser;

service Geyser {
    rpc Subscribe(SubscribeRequest) returns (stream SubscribeUpdate);
}

message SubscribeRequest {
    map<string, SubscribeRequestFilterSlots> slots = 1;
    map<string, SubscribeRequestFilterAccounts> accounts = 2;
    map<string, SubscribeRequestFilterTransactions> transactions = 3;
    map<string, SubscribeRequestFilterBlocks> blocks = 4;
    map<string, SubscribeRequestFilterBlocksMeta> blocks_meta = 5;
    repeated string accounts_data_slice = 6;
    SubscribeRequestPing ping = 7;
    int32 commitment_level = 8;
}

message SubscribeRequestFilterSlots {}

message SubscribeRequestFilterAccounts {
    repeated string account = 1;
    repeated string owner = 2;
    bool filters = 3;
}

message SubscribeRequestFilterTransactions {
    repeated string account = 1;
    repeated string account_include = 2;
    repeated string account_exclude = 3;
    repeated string account_required = 4;
}

message SubscribeRequestFilterBlocks {}

message SubscribeRequestFilterBlocksMeta {}

message SubscribeRequestPing {
    int32 id = 1;
}

message SubscribeUpdate {
    oneof update_oneof {
        SubscribeUpdateSlot slot = 1;
        SubscribeUpdateAccount account = 2;
        SubscribeUpdateTransaction transaction = 3;
        SubscribeUpdateBlock block = 4;
        SubscribeUpdatePing ping = 5;
        SubscribeUpdatePong pong = 6;
        SubscribeUpdateBlockMeta block_meta = 7;
    }
}

message SubscribeUpdateSlot {
    uint64 slot = 1;
    uint64 parent = 2;
    uint32 status = 3;
}

message SubscribeUpdateAccount {
    SubscribeUpdateAccountInfo account = 1;
    uint64 slot = 2;
    bool is_startup = 3;
}

message SubscribeUpdateAccountInfo {
    bytes pubkey = 1;
    uint64 lamports = 2;
    bytes owner = 3;
    bool executable = 4;
    uint64 rent_epoch = 5;
    bytes data = 6;
    uint64 write_version = 7;
    bytes txn_signature = 8;
}

message SubscribeUpdateTransaction {
    SubscribeUpdateTransactionInfo transaction = 1;
    uint64 slot = 2;
}

message SubscribeUpdateTransactionInfo {
    bytes signature = 1;
    bool is_vote = 2;
    SubscribeUpdateTransactionInfoMeta meta = 3;
    bytes transaction = 4;
}

message SubscribeUpdateTransactionInfoMeta {
    int32 err = 1;
    uint64 fee = 2;
    repeated uint64 pre_balances = 3;
    repeated uint64 post_balances = 4;
    repeated SubscribeUpdateTransactionInfoMetaInnerInstructions inner_instructions = 5;
    repeated string log_messages = 6;
    repeated bytes pre_token_balances = 7;
    repeated bytes post_token_balances = 8;
    repeated SubscribeUpdateTransactionInfoMetaRewards rewards = 9;
    repeated bytes loaded_writable_addresses = 10;
    repeated bytes loaded_readonly_addresses = 11;
    uint64 compute_units_consumed = 12;
}

message SubscribeUpdateTransactionInfoMetaInnerInstructions {
    uint32 index = 1;
    repeated SubscribeUpdateTransactionInfoMetaInnerInstruction instructions = 2;
}

message SubscribeUpdateTransactionInfoMetaInnerInstruction {
    uint32 program_id_index = 1;
    repeated uint32 accounts = 2;
    bytes data = 3;
}

message SubscribeUpdateTransactionInfoMetaRewards {
    bytes pubkey = 1;
    int64 lamports = 2;
    uint64 post_balance = 3;
    uint32 reward_type = 4;
    string commission = 5;
}

message SubscribeUpdateBlock {
    uint64 slot = 1;
    string blockhash = 2;
    repeated SubscribeUpdateBlockReward rewards = 3;
    SubscribeUpdateBlockTime block_time = 4;
    uint64 block_height = 5;
    uint64 parent_slot = 6;
    string parent_blockhash = 7;
    uint64 executed_transaction_count = 8;
    repeated SubscribeUpdateTransactionInfo transactions = 9;
}

message SubscribeUpdateBlockTime {
    int64 timestamp = 1;
}

message SubscribeUpdateBlockReward {
    string pubkey = 1;
    int64 lamports = 2;
    uint64 post_balance = 3;
    uint32 reward_type = 4;
    string commission = 5;
}

message SubscribeUpdatePing {}

message SubscribeUpdatePong {
    int32 id = 1;
}

message SubscribeUpdateBlockMeta {
    uint64 slot = 1;
    string blockhash = 2;
    repeated SubscribeUpdateBlockReward rewards = 3;
    SubscribeUpdateBlockTime block_time = 4;
    uint64 block_height = 5;
    uint64 parent_slot = 6;
    string parent_blockhash = 7;
    uint64 executed_transaction_count = 8;
    uint64 entries_count = 9;
}
"#;

    let proto_path = out_dir.join("geyser.proto");
    std::fs::write(&proto_path, proto_content)?;

    tonic_build::configure()
        .build_server(false)
        .out_dir(&out_dir)
        .compile(&[proto_path], &[out_dir])?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
