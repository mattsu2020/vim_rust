# Simplistic Makefile for building xxd on Amiga using Rust

Xxd:
	cargo build --release --manifest-path=../rust/xxd/Cargo.toml
	cp ../rust/xxd/target/release/xxd Xxd

clean:
	cargo clean --manifest-path=../rust/xxd/Cargo.toml
	-delete Xxd

# vim: set noet sw=8 ts=8 sts=0 wm=0 tw=79 ft=make:

