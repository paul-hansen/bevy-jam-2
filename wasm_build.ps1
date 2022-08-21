cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --out-name bevy-jam-2 --out-dir wasm --target web target/wasm32-unknown-unknown/release/bevy-jam-2.wasm
cp -Force -r assets wasm/
http-serve-folder --header "Cross-Origin-Opener-Policy: same-origin" --header "Cross-Origin-Embedder-Policy: require-corp" wasm/