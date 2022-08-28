cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --out-name wasm --out-dir wasm --target web target/wasm32-unknown-unknown/release/flock-fusion.wasm
cp -Force -r assets wasm/
http-serve-folder --header "Cross-Origin-Opener-Policy: same-origin" --header "Cross-Origin-Embedder-Policy: require-corp" wasm/