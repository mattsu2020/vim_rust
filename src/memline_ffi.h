#include <stddef.h>
#include <stdint.h>

// Forward declaration of the opaque Rust type.
typedef struct MemBuffer MemBuffer;

// FFI functions provided by the Rust implementation.
extern MemBuffer *rs_ml_buffer_new(void);
extern void rs_ml_buffer_free(MemBuffer *buf);
extern int rs_ml_append(MemBuffer *buf, size_t lnum, const char *line);
extern int rs_ml_delete(MemBuffer *buf, size_t lnum);
extern int rs_ml_replace(MemBuffer *buf, size_t lnum, const char *line);
extern const char *rs_ml_get_line(MemBuffer *buf, size_t lnum);

typedef struct SwapFile SwapFile;
extern SwapFile *rs_swap_file_open(const char *path, size_t size);
extern void rs_swap_file_close(SwapFile *sf);
