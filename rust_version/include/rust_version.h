#ifndef RUST_VERSION_H
#define RUST_VERSION_H

#ifdef __cplusplus
extern "C" {
#endif

const char *rust_short_version(void);
const char *rust_long_version(void);

#ifdef __cplusplus
}
#endif

#endif // RUST_VERSION_H
