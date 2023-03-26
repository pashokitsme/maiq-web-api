mkdir -p bin/default
cp -r default bin/default/
cargo build --release
cp target/release/maiq-web bin