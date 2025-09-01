! Simplistic MMS file for building xxd on VMS using Rust

xxd.exe :
	cargo build --release --manifest-path=../rust/xxd/Cargo.toml
	copy ../rust/xxd/target/release/xxd.exe xxd.exe

clean :
	cargo clean --manifest-path=../rust/xxd/Cargo.toml
	delete xxd.exe;*


