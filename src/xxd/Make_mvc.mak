# Build xxd using Rust crate for Win32 using Microsoft Visual C++

!INCLUDE ..\\auto\\nmake\\tools.mak

xxd: xxd.exe

xxd.exe:
	cargo build --release --manifest-path=../rust/xxd/Cargo.toml
	copy ..\\rust\\xxd\\target\\release\\xxd.exe xxd.exe

clean:
	cargo clean --manifest-path=../rust/xxd/Cargo.toml
	- if exist xxd.exe $(RM) xxd.exe

# vim: set noet sw=8 ts=8 sts=0 wm=0 tw=79 ft=make:

