# Build xxd using Rust crate
xxd:
	cargo build --release --manifest-path=../rust/xxd/Cargo.toml
	cp ../rust/xxd/target/release/xxd xxd

clean:
	cargo clean --manifest-path=../rust/xxd/Cargo.toml
	rm -f xxd
