set -euxo pipefail

cargo build -p canister --release --target wasm32-wasi
cargo test -- --nocapture