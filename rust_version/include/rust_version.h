#ifndef RUST_VERSION_H
#define RUST_VERSION_H

#ifdef __cplusplus
extern "C" {
#endif

// Returns 1 if an option was handled (e.g. --version/--help), 0 otherwise.
int rust_handle_args(int argc, const char* argv[]);

#ifdef __cplusplus
}
#endif

#endif // RUST_VERSION_H

