cargo clean
cargo build --release --workspace --exclude som-value && cargo build -p som-value --no-default-features --features use-lbits
