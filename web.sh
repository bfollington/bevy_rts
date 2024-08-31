cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web \
    --out-dir ./out/ \
    --out-name "bevy_platformer" \
    ./target/wasm32-unknown-unknown/release/bevy_platformer.wasm
cp index.html out/
cp -r assets out/
