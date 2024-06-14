set -euxo pipefail

cargo build -p canister --release --target wasm32-wasi
wasi2ic target/wasm32-wasi/release/canister.wasm canister.wasm
cargo test -- --nocapture