echo "Compiling release..."
cargo build --release 
rm -rf target/release/* 
echo "Generating bindings..."
deno_bindgen --release --lazy-init