mkdir -p bin/default
cp -r default bin/
cargo build --release
cp target/release/maiq-web bin/