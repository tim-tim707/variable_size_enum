# Variable size enum for Rust

cargo build
cargo run --manifest-path example/Cargo.toml
cargo expand -p example
cargo run -p example

cargo test -p variable_size_enum_macro
