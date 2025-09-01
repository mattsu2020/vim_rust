#include <stddef.h>
#include <stdint.h>

// Forward declaration of the opaque Rust type.
typedef struct MemBuffer MemBuffer;

// FFI functions provided by the Rust implementation.
extern MemBuffer *ml_buffer_new(void);
extern void ml_buffer_free(MemBuffer *buf);
extern int ml_append(MemBuffer *buf, size_t lnum, const char *line);
extern int ml_delete(MemBuffer *buf, size_t lnum);
extern int ml_replace(MemBuffer *buf, size_t lnum, const char *line);
