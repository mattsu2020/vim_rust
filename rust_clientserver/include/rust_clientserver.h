#ifndef RUST_CLIENTSERVER_H
#define RUST_CLIENTSERVER_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int socket_server_init(const unsigned char *servername);
void socket_server_uninit(void);
int socket_server_valid(void);
int socket_server_waiting_accept(void);

int socket_server_send(const unsigned char *servername,
                       const unsigned char *keys,
                       unsigned char **result,
                       unsigned char **client,
                       long expr_flag_or_ptr,
                       int timeout_ms,
                       int flags);

int socket_server_read_reply(const unsigned char *receiver,
                             unsigned char **out,
                             int timeout_ms);

int socket_server_peek_reply(const unsigned char *serverid,
                             unsigned char **out);

int socket_server_send_reply(const unsigned char *server,
                             const unsigned char *reply);

unsigned char *socket_server_list_sockets(void);

#ifdef __cplusplus
}
#endif

#endif // RUST_CLIENTSERVER_H
