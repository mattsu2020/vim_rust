# Global State and Error Handling

## 6.1 C global variables

The list of global variables declared on the C side was generated from
`src/globals.h` using `rg '^EXTERN'` and is available in
[`c_global_variables.txt`](./c_global_variables.txt).

## 6.2 State management plan

Many of these variables represent screen or editor state that can be
encapsulated inside dedicated structs.  A possible migration path is to
introduce structures such as `ScreenState` and use `once_cell::sync::OnceCell`
(or `lazy_static`) to provide safe, late initialization for global data.
Each module can then own its state, reducing the number of cross-module
mutable globals.

## 6.3 setjmp/longjmp usage

Occurrences of `setjmp` and `longjmp` were extracted with ripgrep and are
listed in [`setjmp_longjmp_usage.txt`](./setjmp_longjmp_usage.txt).
These primarily appear in language bindings and platform specific code.  The
plan is to replace these with Rust's `panic!`/`catch_unwind` mechanisms to
propagate errors in a controlled manner.  Wrapper functions can convert
between the C style long jumps and Rust panics during the transition period.

## 6.4 Clippy

`cargo clippy -p diff --no-deps` was executed to check for warnings after the
investigation.  Remaining warnings relate to missing safety documentation and a
manual `unwrap_or` implementation; they will be addressed alongside the actual
state migration work.
