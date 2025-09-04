# Makefile for building the Rust-based gvimext.dll using MSVC cargo toolchain

all: gvimext.dll

# Build the Rust crate and copy the resulting DLL here
# Assumes cargo is available in PATH
GVIMEXT_DLL=..\..\target\release\rust_gvimext.dll

gvimext.dll:
	cargo build -p rust_gvimext --release
	copy $(GVIMEXT_DLL) gvimext.dll

register: gvimext.dll
	regsvr32 /s gvimext.dll

clean:
	- if exist gvimext.dll del gvimext.dll

# vim: set noet sw=8 ts=8 sts=0 wm=0 tw=79 ft=make:
