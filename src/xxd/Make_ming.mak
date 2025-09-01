# Simplistic Makefile for building xxd with MinGW or Cygwin using Rust

xxd.exe:
	cargo build --release --manifest-path=../rust/xxd/Cargo.toml
	cp ../rust/xxd/target/release/xxd.exe xxd.exe

clean:
	cargo clean --manifest-path=../rust/xxd/Cargo.toml
	- rm -f xxd.exe

# vim: set noet sw=8 ts=8 sts=0 wm=0 tw=79 ft=make:

