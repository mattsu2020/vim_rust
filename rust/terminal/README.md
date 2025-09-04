# vim-terminal

This crate provides a small experimental terminal abstraction and a simple
Rust-based test harness.

## Running tests

The harness is exercised via the standard Cargo test command:

```bash
cargo test -p vim-terminal
```

The `TestHarness` type allows tests to drive a [`Terminal`] instance with
hexadecimal byte sequences and query lines from the scrollback buffer.
