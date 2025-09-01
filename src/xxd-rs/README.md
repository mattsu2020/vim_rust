# xxd-rs

Rust reimplementation of Vim's `xxd` utility. It currently supports:

- Standard hexdump output.
- Plain hexdump via `-p`.
- Reverse operation via `-r` for both formats.

This crate uses `std::io` for I/O and propagates errors via `Result`.
