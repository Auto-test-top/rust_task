echo "Start"
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

echo "Contract deploy"
solana program deploy contract/target/deploy/solana_deposit.so --keypair ~/.config/solana/id.json --url devnet
echo "OK"
