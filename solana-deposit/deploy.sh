# build.sh
#!/bin/bash
echo "Contract assembly"
cd contract

if command -v cargo-build-sbf &> /dev/null; then
    cargo build-sbf
elif command -v cargo-build-bpf &> /dev/null; then
    cargo build-bpf
else
    cargo install solana-cargo-build-sbf
    cargo build-sbf
fi

cd ..
echo "Contract done"

# deploy.sh
#!/bin/bash

if [ -f "contract/target/deploy/solana_deposit.so" ]; then
    SO_FILE="contract/target/deploy/solana_deposit.so"
elif [ -f "contract/target/sbf-solana-solana/release/solana_deposit.so" ]; then
    SO_FILE="contract/target/sbf-solana-solana/release/solana_deposit.so"
else
    echo "Err: .so file not found"
    find contract/target -name "*.so" -type f
    exit 1
fi

solana program deploy "$SO_FILE" --keypair ~/.config/solana/id.json --url devnet
echo "Program ID"