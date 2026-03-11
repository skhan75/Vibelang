#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <strings.h>
#include <ctype.h>
#include <errno.h>
#include <pthread.h>
#include <math.h>
#include <time.h>
#include <unistd.h>
#ifdef _WIN32
#include <direct.h>
#else
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#endif

// Bench runtime helpers are intentionally self-contained.
// They can call into core runtime for panic reporting.
void vibe_panic(const char *s);

typedef struct vibe_bench_string_builder {
    char *data;
    size_t len;
    size_t cap;
} vibe_bench_string_builder;

static void vibe_bench_builder_init(vibe_bench_string_builder *builder, size_t initial_cap) {
    size_t cap = initial_cap < 16 ? 16 : initial_cap;
    builder->data = (char *)calloc(cap, sizeof(char));
    if (builder->data == NULL) {
        vibe_panic("failed to allocate bench string builder");
    }
    builder->len = 0;
    builder->cap = cap;
    builder->data[0] = '\0';
}

static void vibe_bench_builder_reserve(vibe_bench_string_builder *builder, size_t extra) {
    size_t needed = builder->len + extra + 1;
    if (needed <= builder->cap) {
        return;
    }
    size_t next = builder->cap;
    while (next < needed) {
        next *= 2;
    }
    char *p = (char *)realloc(builder->data, next);
    if (p == NULL) {
        vibe_panic("failed to grow bench string builder");
    }
    builder->data = p;
    builder->cap = next;
}

static void vibe_bench_builder_append_bytes(
    vibe_bench_string_builder *builder,
    const char *bytes,
    size_t len
) {
    if (len == 0) {
        return;
    }
    vibe_bench_builder_reserve(builder, len);
    memcpy(builder->data + builder->len, bytes, len);
    builder->len += len;
    builder->data[builder->len] = '\0';
}

static char *vibe_bench_strdup_or_panic(const char *src) {
    const char *raw = src == NULL ? "" : src;
    size_t len = strlen(raw);
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate bench strdup");
    }
    memcpy(out, raw, len);
    out[len] = '\0';
    return out;
}

// --- MD5 (bench) ---

typedef struct vibe_bench_md5_ctx {
    uint32_t state[4];
    uint64_t bitlen;
    unsigned char buffer[64];
} vibe_bench_md5_ctx;

static uint32_t vibe_bench_md5_left_rotate(uint32_t x, uint32_t c) {
    return (x << c) | (x >> (32u - c));
}

static void vibe_bench_md5_transform(vibe_bench_md5_ctx *ctx, const unsigned char data[64]) {
    static const uint32_t k[64] = {
        0xd76aa478u, 0xe8c7b756u, 0x242070dbu, 0xc1bdceeeu, 0xf57c0fafu, 0x4787c62au,
        0xa8304613u, 0xfd469501u, 0x698098d8u, 0x8b44f7afu, 0xffff5bb1u, 0x895cd7beu,
        0x6b901122u, 0xfd987193u, 0xa679438eu, 0x49b40821u, 0xf61e2562u, 0xc040b340u,
        0x265e5a51u, 0xe9b6c7aau, 0xd62f105du, 0x02441453u, 0xd8a1e681u, 0xe7d3fbc8u,
        0x21e1cde6u, 0xc33707d6u, 0xf4d50d87u, 0x455a14edu, 0xa9e3e905u, 0xfcefa3f8u,
        0x676f02d9u, 0x8d2a4c8au, 0xfffa3942u, 0x8771f681u, 0x6d9d6122u, 0xfde5380cu,
        0xa4beea44u, 0x4bdecfa9u, 0xf6bb4b60u, 0xbebfbc70u, 0x289b7ec6u, 0xeaa127fau,
        0xd4ef3085u, 0x04881d05u, 0xd9d4d039u, 0xe6db99e5u, 0x1fa27cf8u, 0xc4ac5665u,
        0xf4292244u, 0x432aff97u, 0xab9423a7u, 0xfc93a039u, 0x655b59c3u, 0x8f0ccc92u,
        0xffeff47du, 0x85845dd1u, 0x6fa87e4fu, 0xfe2ce6e0u, 0xa3014314u, 0x4e0811a1u,
        0xf7537e82u, 0xbd3af235u, 0x2ad7d2bbu, 0xeb86d391u,
    };
    static const uint32_t r[64] = {
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    };

    uint32_t a = ctx->state[0];
    uint32_t b = ctx->state[1];
    uint32_t c = ctx->state[2];
    uint32_t d = ctx->state[3];

    uint32_t m[16];
    for (int i = 0; i < 16; i++) {
        m[i] = (uint32_t)data[i * 4] |
               ((uint32_t)data[i * 4 + 1] << 8u) |
               ((uint32_t)data[i * 4 + 2] << 16u) |
               ((uint32_t)data[i * 4 + 3] << 24u);
    }

    for (int i = 0; i < 64; i++) {
        uint32_t f;
        uint32_t g;
        if (i < 16) {
            f = (b & c) | ((~b) & d);
            g = (uint32_t)i;
        } else if (i < 32) {
            f = (d & b) | ((~d) & c);
            g = (uint32_t)((5 * i + 1) % 16);
        } else if (i < 48) {
            f = b ^ c ^ d;
            g = (uint32_t)((3 * i + 5) % 16);
        } else {
            f = c ^ (b | (~d));
            g = (uint32_t)((7 * i) % 16);
        }
        uint32_t temp = d;
        d = c;
        c = b;
        uint32_t sum = a + f + k[i] + m[g];
        b = b + vibe_bench_md5_left_rotate(sum, r[i]);
        a = temp;
    }

    ctx->state[0] += a;
    ctx->state[1] += b;
    ctx->state[2] += c;
    ctx->state[3] += d;
}

static void vibe_bench_md5_init(vibe_bench_md5_ctx *ctx) {
    ctx->bitlen = 0;
    ctx->state[0] = 0x67452301u;
    ctx->state[1] = 0xefcdab89u;
    ctx->state[2] = 0x98badcfeu;
    ctx->state[3] = 0x10325476u;
    memset(ctx->buffer, 0, sizeof(ctx->buffer));
}

static void vibe_bench_md5_update(vibe_bench_md5_ctx *ctx, const unsigned char *data, size_t len) {
    size_t idx = (size_t)((ctx->bitlen / 8u) % 64u);
    ctx->bitlen += (uint64_t)len * 8u;
    size_t i = 0;
    if (idx != 0) {
        size_t fill = 64u - idx;
        if (len < fill) {
            memcpy(ctx->buffer + idx, data, len);
            return;
        }
        memcpy(ctx->buffer + idx, data, fill);
        vibe_bench_md5_transform(ctx, ctx->buffer);
        i += fill;
        idx = 0;
    }
    while (i + 64u <= len) {
        vibe_bench_md5_transform(ctx, data + i);
        i += 64u;
    }
    if (i < len) {
        memcpy(ctx->buffer, data + i, len - i);
    }
}

static void vibe_bench_md5_final(vibe_bench_md5_ctx *ctx, unsigned char out[16]) {
    unsigned char pad[64];
    memset(pad, 0, sizeof(pad));
    pad[0] = 0x80;

    size_t idx = (size_t)((ctx->bitlen / 8u) % 64u);
    size_t pad_len = (idx < 56u) ? (56u - idx) : (120u - idx);
    uint64_t original_bits = ctx->bitlen;
    vibe_bench_md5_update(ctx, pad, pad_len);

    unsigned char len_bytes[8];
    uint64_t bits = original_bits;
    for (int i = 0; i < 8; i++) {
        len_bytes[i] = (unsigned char)((bits >> (8u * (uint32_t)i)) & 0xffu);
    }
    vibe_bench_md5_update(ctx, len_bytes, 8);

    for (int i = 0; i < 4; i++) {
        out[i * 4] = (unsigned char)(ctx->state[i] & 0xffu);
        out[i * 4 + 1] = (unsigned char)((ctx->state[i] >> 8u) & 0xffu);
        out[i * 4 + 2] = (unsigned char)((ctx->state[i] >> 16u) & 0xffu);
        out[i * 4 + 3] = (unsigned char)((ctx->state[i] >> 24u) & 0xffu);
    }
}

char *vibe_bench_md5_bytes_hex(void *handle) {
    if (handle == NULL) vibe_panic("md5_bytes_hex: null handle");
    int64_t *header = (int64_t *)handle;
    int64_t len = header[1];
    int64_t *items = *(int64_t **)(header + 3);
    size_t byte_len = (size_t)len;
    unsigned char *buf = (unsigned char *)malloc(byte_len);
    if (buf == NULL) vibe_panic("md5_bytes_hex: alloc failed");
    for (size_t i = 0; i < byte_len; i++) {
        buf[i] = (unsigned char)(items[i] & 0xff);
    }
    vibe_bench_md5_ctx ctx;
    vibe_bench_md5_init(&ctx);
    vibe_bench_md5_update(&ctx, buf, byte_len);
    unsigned char digest[16];
    vibe_bench_md5_final(&ctx, digest);
    free(buf);

    static const char hex[] = "0123456789abcdef";
    char *out = (char *)calloc(33, sizeof(char));
    if (out == NULL) vibe_panic("md5_bytes_hex: alloc failed");
    for (int i = 0; i < 16; i++) {
        out[i * 2] = hex[(digest[i] >> 4) & 0x0f];
        out[i * 2 + 1] = hex[digest[i] & 0x0f];
    }
    out[32] = '\0';
    return out;
}

char *vibe_bench_md5_hex(const char *raw) {
    const unsigned char *bytes = (const unsigned char *)(raw == NULL ? "" : raw);
    size_t len = strlen((const char *)bytes);
    vibe_bench_md5_ctx ctx;
    vibe_bench_md5_init(&ctx);
    vibe_bench_md5_update(&ctx, bytes, len);
    unsigned char digest[16];
    vibe_bench_md5_final(&ctx, digest);

    static const char hex[] = "0123456789abcdef";
    char *out = (char *)calloc(33, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate md5 hex string");
    }
    for (int i = 0; i < 16; i++) {
        out[i * 2] = hex[(digest[i] >> 4) & 0x0f];
        out[i * 2 + 1] = hex[digest[i] & 0x0f];
    }
    out[32] = '\0';
    return out;
}

// --- JSON canonicalization (bench) ---

static void vibe_bench_format_shortest_f64(double value, char out[64]) {
    char candidate[64];
    for (int precision = 1; precision <= 17; precision++) {
        int wrote = snprintf(candidate, sizeof(candidate), "%.*g", precision, value);
        if (wrote <= 0 || wrote >= (int)sizeof(candidate)) {
            continue;
        }
        char *end = NULL;
        double roundtrip = strtod(candidate, &end);
        if (end != NULL && *end == '\0' && memcmp(&roundtrip, &value, sizeof(double)) == 0) {
            memcpy(out, candidate, (size_t)wrote + 1);
            return;
        }
    }
    (void)snprintf(out, 64, "%.17g", value);
}

char *vibe_bench_json_canonical(const char *raw) {
    if (raw == NULL) {
        return vibe_bench_strdup_or_panic("");
    }
    size_t len = strlen(raw);
    vibe_bench_string_builder builder;
    vibe_bench_builder_init(&builder, len + 32);

    int in_string = 0;
    int escaped = 0;
    size_t i = 0;
    while (i < len) {
        char ch = raw[i];
        if (in_string) {
            vibe_bench_builder_append_bytes(&builder, &ch, 1);
            if (escaped) {
                escaped = 0;
            } else if (ch == '\\') {
                escaped = 1;
            } else if (ch == '"') {
                in_string = 0;
            }
            i += 1;
            continue;
        }
        if (ch == '"') {
            in_string = 1;
            vibe_bench_builder_append_bytes(&builder, &ch, 1);
            i += 1;
            continue;
        }
        if (isspace((unsigned char)ch)) {
            i += 1;
            continue;
        }
        if (ch == '-' || (ch >= '0' && ch <= '9')) {
            size_t start = i;
            int saw_float = 0;
            while (i < len) {
                char c = raw[i];
                if ((c >= '0' && c <= '9') || c == '-' || c == '+' || c == '.' || c == 'e' ||
                    c == 'E') {
                    if (c == '.' || c == 'e' || c == 'E') {
                        saw_float = 1;
                    }
                    i += 1;
                    continue;
                }
                break;
            }
            size_t tok_len = i - start;
            if (!saw_float) {
                vibe_bench_builder_append_bytes(&builder, raw + start, tok_len);
            } else {
                char num_buf[128];
                size_t copy_len = tok_len < sizeof(num_buf) - 1 ? tok_len : sizeof(num_buf) - 1;
                memcpy(num_buf, raw + start, copy_len);
                num_buf[copy_len] = '\0';
                char *endptr = NULL;
                double value = strtod(num_buf, &endptr);
                if (endptr == num_buf) {
                    vibe_bench_builder_append_bytes(&builder, raw + start, tok_len);
                } else {
                    char out_num[64];
                    vibe_bench_format_shortest_f64(value, out_num);
                    int wrote = (int)strlen(out_num);
                    if (wrote <= 0) {
                        vibe_bench_builder_append_bytes(&builder, raw + start, tok_len);
                    } else {
                        vibe_bench_builder_append_bytes(&builder, out_num, (size_t)wrote);
                    }
                }
            }
            continue;
        }
        vibe_bench_builder_append_bytes(&builder, &ch, 1);
        i += 1;
    }
    return builder.data;
}

char *vibe_bench_json_repeat_array(const char *item, int64_t n) {
    if (n <= 0) {
        return vibe_bench_strdup_or_panic("[]");
    }
    const char *value = item == NULL ? "" : item;
    size_t item_len = strlen(value);
    size_t approx = 2 + (size_t)n * item_len + (size_t)(n - 1);
    vibe_bench_string_builder builder;
    vibe_bench_builder_init(&builder, approx + 16);
    vibe_bench_builder_append_bytes(&builder, "[", 1);
    for (int64_t i = 0; i < n; i++) {
        if (i > 0) {
            vibe_bench_builder_append_bytes(&builder, ",", 1);
        }
        vibe_bench_builder_append_bytes(&builder, value, item_len);
    }
    vibe_bench_builder_append_bytes(&builder, "]", 1);
    return builder.data;
}

// --- net primitives (bench) ---

int64_t vibe_bench_net_listen(const char *host, int64_t port) {
#ifdef _WIN32
    (void)host;
    (void)port;
    return 0;
#else
    const char *addr = (host == NULL || host[0] == '\0') ? "127.0.0.1" : host;
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) {
        return 0;
    }
    int opt = 1;
    (void)setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    struct sockaddr_in sa;
    memset(&sa, 0, sizeof(sa));
    sa.sin_family = AF_INET;
    sa.sin_port = htons((uint16_t)(port < 0 ? 0 : port));
    if (inet_pton(AF_INET, addr, &sa.sin_addr) != 1) {
        close(fd);
        return 0;
    }
    if (bind(fd, (struct sockaddr *)&sa, sizeof(sa)) != 0) {
        close(fd);
        return 0;
    }
    if (listen(fd, 128) != 0) {
        close(fd);
        return 0;
    }
    return (int64_t)fd;
#endif
}

int64_t vibe_bench_net_listener_port(int64_t listener_fd) {
#ifdef _WIN32
    (void)listener_fd;
    return 0;
#else
    int fd = (int)listener_fd;
    struct sockaddr_in sa;
    socklen_t len = (socklen_t)sizeof(sa);
    memset(&sa, 0, sizeof(sa));
    if (getsockname(fd, (struct sockaddr *)&sa, &len) != 0) {
        return 0;
    }
    return (int64_t)ntohs(sa.sin_port);
#endif
}

int64_t vibe_bench_net_accept(int64_t listener_fd) {
#ifdef _WIN32
    (void)listener_fd;
    return 0;
#else
    int fd = (int)listener_fd;
    int conn = accept(fd, NULL, NULL);
    if (conn < 0) {
        return 0;
    }
    return (int64_t)conn;
#endif
}

int64_t vibe_bench_net_connect(const char *host, int64_t port) {
#ifdef _WIN32
    (void)host;
    (void)port;
    return 0;
#else
    const char *addr = (host == NULL || host[0] == '\0') ? "127.0.0.1" : host;
    if (port <= 0 || port > 65535) {
        return 0;
    }
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) {
        return 0;
    }
    struct sockaddr_in sa;
    memset(&sa, 0, sizeof(sa));
    sa.sin_family = AF_INET;
    sa.sin_port = htons((uint16_t)port);
    if (inet_pton(AF_INET, addr, &sa.sin_addr) != 1) {
        close(fd);
        return 0;
    }
    if (connect(fd, (struct sockaddr *)&sa, sizeof(sa)) != 0) {
        close(fd);
        return 0;
    }
    return (int64_t)fd;
#endif
}

char *vibe_bench_net_read(int64_t fd_raw, int64_t max_bytes_raw) {
#ifdef _WIN32
    (void)fd_raw;
    (void)max_bytes_raw;
    return vibe_bench_strdup_or_panic("");
#else
    int fd = (int)fd_raw;
    int64_t max_bytes = max_bytes_raw;
    if (fd <= 0 || max_bytes <= 0) {
        return vibe_bench_strdup_or_panic("");
    }
    if (max_bytes > 4 * 1024 * 1024) {
        max_bytes = 4 * 1024 * 1024;
    }
    char *buffer = (char *)calloc((size_t)max_bytes + 1, sizeof(char));
    if (buffer == NULL) {
        vibe_panic("failed to allocate net.read buffer");
    }
    ssize_t n = recv(fd, buffer, (size_t)max_bytes, 0);
    if (n <= 0) {
        free(buffer);
        return vibe_bench_strdup_or_panic("");
    }
    buffer[(size_t)n] = '\0';
    return buffer;
#endif
}

int64_t vibe_bench_net_write(int64_t fd_raw, const char *data) {
#ifdef _WIN32
    (void)fd_raw;
    (void)data;
    return 0;
#else
    int fd = (int)fd_raw;
    const char *raw = data == NULL ? "" : data;
    size_t len = strlen(raw);
    if (fd <= 0 || len == 0) {
        return 0;
    }
    ssize_t n = send(fd, raw, len, 0);
    return n < 0 ? 0 : (int64_t)n;
#endif
}

int64_t vibe_bench_net_close(int64_t fd_raw) {
#ifdef _WIN32
    (void)fd_raw;
    return 0;
#else
    int fd = (int)fd_raw;
    if (fd <= 0) {
        return 0;
    }
    return close(fd) == 0 ? 1 : 0;
#endif
}

// --- HTTP server microbenchmark (bench) ---

static int64_t vibe_bench_http_extract_i64(const char *text) {
    if (text == NULL) {
        return 0;
    }
    const char *p = text;
    while (*p != '\0') {
        if (*p == '-' || (*p >= '0' && *p <= '9')) {
            char *end = NULL;
            long long v = strtoll(p, &end, 10);
            if (end != NULL && end != p) {
                return (int64_t)v;
            }
            return 0;
        }
        p++;
    }
    return 0;
}

static const char *vibe_bench_http_find_body(const char *req) {
    if (req == NULL) {
        return NULL;
    }
    const char *p = strstr(req, "\r\n\r\n");
    return p == NULL ? NULL : (p + 4);
}

static char *vibe_bench_http_read_message(int fd, int64_t max_bytes) {
    if (max_bytes <= 0) {
        return vibe_bench_strdup_or_panic("");
    }
    if (max_bytes > 4 * 1024 * 1024) {
        max_bytes = 4 * 1024 * 1024;
    }
    vibe_bench_string_builder builder;
    vibe_bench_builder_init(&builder, 4096);
    while ((int64_t)builder.len < max_bytes) {
        char tmp[4096];
        size_t remain = (size_t)(max_bytes - (int64_t)builder.len);
        size_t to_read = remain < sizeof(tmp) ? remain : sizeof(tmp);
        ssize_t n = recv(fd, tmp, to_read, 0);
        if (n <= 0) {
            break;
        }
        vibe_bench_builder_append_bytes(&builder, tmp, (size_t)n);
        const char *body = vibe_bench_http_find_body(builder.data);
        if (body != NULL) {
            if (vibe_bench_http_extract_i64(body) != 0 || strstr(body, "0") != NULL) {
                break;
            }
        }
    }
    return builder.data;
}

static void vibe_bench_http_handle_conn(int conn) {
    char *req = vibe_bench_http_read_message(conn, 16384);
    const char *body = vibe_bench_http_find_body(req);
    int64_t value = vibe_bench_http_extract_i64(body);
    free(req);

    char resp_body[32];
    snprintf(resp_body, sizeof(resp_body), "%lld", (long long)value);
    size_t resp_len = strlen(resp_body);

    char header[128];
    int header_len = snprintf(
        header,
        sizeof(header),
        "HTTP/1.1 200 OK\r\nContent-Length: %zu\r\n\r\n",
        resp_len
    );
    if (header_len < 0) {
        header_len = 0;
    }
    (void)vibe_bench_net_write((int64_t)conn, header);
    (void)vibe_bench_net_write((int64_t)conn, resp_body);
    (void)vibe_bench_net_close((int64_t)conn);
}

typedef struct vibe_bench_http_server_ctx {
    int listener;
    int64_t total;
} vibe_bench_http_server_ctx;

static void *vibe_bench_http_server_thread(void *arg) {
    vibe_bench_http_server_ctx *ctx = (vibe_bench_http_server_ctx *)arg;
    int handled = 0;
    while ((int64_t)handled < ctx->total) {
        int conn = (int)vibe_bench_net_accept((int64_t)ctx->listener);
        if (conn != 0) {
            vibe_bench_http_handle_conn(conn);
            handled++;
        }
    }
    (void)vibe_bench_net_close((int64_t)ctx->listener);
    return NULL;
}

typedef struct vibe_bench_http_client_ctx {
    int port;
    int64_t value;
    int64_t *out;
} vibe_bench_http_client_ctx;

static void *vibe_bench_http_client_thread(void *arg) {
    vibe_bench_http_client_ctx *ctx = (vibe_bench_http_client_ctx *)arg;
    int64_t value = ctx->value;
    char body[64];
    snprintf(body, sizeof(body), "{\"value\":%lld}", (long long)value);
    size_t body_len = strlen(body);

    char req[256];
    snprintf(
        req,
        sizeof(req),
        "POST /api HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: %zu\r\n\r\n%s",
        body_len,
        body
    );

    int64_t conn = 0;
    for (int tries = 0; tries < 200; tries++) {
        conn = vibe_bench_net_connect("127.0.0.1", (int64_t)ctx->port);
        if (conn != 0) {
            break;
        }
    }
    if (conn == 0) {
        *ctx->out = 0;
        return NULL;
    }
    (void)vibe_bench_net_write(conn, req);
    char *resp = vibe_bench_http_read_message((int)conn, 16384);
    (void)vibe_bench_net_close(conn);
    const char *body_ptr = vibe_bench_http_find_body(resp);
    int64_t parsed = vibe_bench_http_extract_i64(body_ptr);
    free(resp);
    *ctx->out = parsed;
    return NULL;
}

int64_t vibe_bench_http_server_bench(int64_t n) {
#ifdef _WIN32
    (void)n;
    return 0;
#else
    if (n <= 0) {
        return 0;
    }
    int64_t listener_raw = vibe_bench_net_listen("127.0.0.1", 0);
    if (listener_raw == 0) {
        return 0;
    }
    int listener = (int)listener_raw;
    int port = (int)vibe_bench_net_listener_port(listener_raw);
    if (port <= 0) {
        (void)vibe_bench_net_close(listener_raw);
        return 0;
    }

    vibe_bench_http_server_ctx server_ctx = {
        .listener = listener,
        .total = n,
    };
    pthread_t server_thread;
    if (pthread_create(&server_thread, NULL, vibe_bench_http_server_thread, &server_ctx) != 0) {
        (void)vibe_bench_net_close(listener_raw);
        return 0;
    }

    pthread_t *threads = (pthread_t *)calloc((size_t)n, sizeof(pthread_t));
    vibe_bench_http_client_ctx *ctxs =
        (vibe_bench_http_client_ctx *)calloc((size_t)n, sizeof(vibe_bench_http_client_ctx));
    int64_t *results = (int64_t *)calloc((size_t)n, sizeof(int64_t));
    if (threads == NULL || ctxs == NULL || results == NULL) {
        vibe_panic("failed to allocate http-server bench resources");
    }
    for (int64_t i = 0; i < n; i++) {
        ctxs[i].port = port;
        ctxs[i].value = i + 1;
        ctxs[i].out = &results[i];
        (void)pthread_create(&threads[i], NULL, vibe_bench_http_client_thread, &ctxs[i]);
    }
    for (int64_t i = 0; i < n; i++) {
        (void)pthread_join(threads[i], NULL);
    }
    (void)pthread_join(server_thread, NULL);

    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) {
        sum += results[i];
    }
    free(threads);
    free(ctxs);
    free(results);
    return sum;
#endif
}

// --- secp256k1 (bench) ---

typedef struct vibe_bench_fe256 {
    uint64_t v[4]; // little-endian limbs
} vibe_bench_fe256;

static const uint64_t VIBE_BENCH_SECP_P[4] = {
    0xFFFFFFFEFFFFFC2Full,
    0xFFFFFFFFFFFFFFFFull,
    0xFFFFFFFFFFFFFFFFull,
    0xFFFFFFFFFFFFFFFFull,
};

static int vibe_bench_fe_ge_p(const vibe_bench_fe256 *a) {
    for (int i = 3; i >= 0; i--) {
        if (a->v[i] > VIBE_BENCH_SECP_P[i]) return 1;
        if (a->v[i] < VIBE_BENCH_SECP_P[i]) return 0;
    }
    return 1;
}

static void vibe_bench_fe_sub_p(vibe_bench_fe256 *a) {
    __uint128_t borrow = 0;
    for (int i = 0; i < 4; i++) {
        __uint128_t av = (__uint128_t)a->v[i];
        __uint128_t bv = (__uint128_t)VIBE_BENCH_SECP_P[i] + borrow;
        __uint128_t r = av - bv;
        a->v[i] = (uint64_t)r;
        borrow = av < bv ? 1u : 0u;
    }
}

static void vibe_bench_fe_add(vibe_bench_fe256 *out, const vibe_bench_fe256 *a, const vibe_bench_fe256 *b) {
    __uint128_t carry = 0;
    for (int i = 0; i < 4; i++) {
        __uint128_t s = (__uint128_t)a->v[i] + b->v[i] + carry;
        out->v[i] = (uint64_t)s;
        carry = s >> 64;
    }
    if (carry || vibe_bench_fe_ge_p(out)) {
        vibe_bench_fe_sub_p(out);
    }
}

static void vibe_bench_fe_sub(vibe_bench_fe256 *out, const vibe_bench_fe256 *a, const vibe_bench_fe256 *b) {
    __uint128_t borrow = 0;
    for (int i = 0; i < 4; i++) {
        __uint128_t av = (__uint128_t)a->v[i];
        __uint128_t bv = (__uint128_t)b->v[i] + borrow;
        __uint128_t r = av - bv;
        out->v[i] = (uint64_t)r;
        borrow = av < bv ? 1u : 0u;
    }
    if (borrow) {
        __uint128_t carry = 0;
        for (int i = 0; i < 4; i++) {
            __uint128_t s = (__uint128_t)out->v[i] + VIBE_BENCH_SECP_P[i] + carry;
            out->v[i] = (uint64_t)s;
            carry = s >> 64;
        }
    }
}

static void vibe_bench_fe_mul_small_add_shift32(vibe_bench_fe256 *io, uint64_t hi) {
    uint64_t high = hi;
    while (high) {
        __uint128_t acc0 =
            (__uint128_t)io->v[0] + ((__uint128_t)high << 32u) + (__uint128_t)high * 977u;
        io->v[0] = (uint64_t)acc0;
        __uint128_t carry = acc0 >> 64;

        __uint128_t acc1 = (__uint128_t)io->v[1] + (high >> 32u) +
                           (uint64_t)(((__uint128_t)high * 977u) >> 64) + carry;
        io->v[1] = (uint64_t)acc1;
        carry = acc1 >> 64;

        __uint128_t acc2 = (__uint128_t)io->v[2] + carry;
        io->v[2] = (uint64_t)acc2;
        carry = acc2 >> 64;

        __uint128_t acc3 = (__uint128_t)io->v[3] + carry;
        io->v[3] = (uint64_t)acc3;
        carry = acc3 >> 64;

        high = (uint64_t)carry;
    }
    if (vibe_bench_fe_ge_p(io)) {
        vibe_bench_fe_sub_p(io);
    }
}

static void vibe_bench_fe_reduce512(vibe_bench_fe256 *out, const uint64_t in[8]) {
    uint64_t lo[4] = {in[0], in[1], in[2], in[3]};
    uint64_t hi[4] = {in[4], in[5], in[6], in[7]};

    uint64_t shifted[5];
    shifted[0] = hi[0] << 32u;
    shifted[1] = (hi[1] << 32u) | (hi[0] >> 32u);
    shifted[2] = (hi[2] << 32u) | (hi[1] >> 32u);
    shifted[3] = (hi[3] << 32u) | (hi[2] >> 32u);
    shifted[4] = hi[3] >> 32u;

    uint64_t mul977[5];
    __uint128_t carry = 0;
    for (int i = 0; i < 4; i++) {
        __uint128_t prod = (__uint128_t)hi[i] * 977u + carry;
        mul977[i] = (uint64_t)prod;
        carry = prod >> 64;
    }
    mul977[4] = (uint64_t)carry;

    uint64_t t[5];
    carry = 0;
    for (int i = 0; i < 4; i++) {
        __uint128_t s = (__uint128_t)lo[i] + shifted[i] + mul977[i] + carry;
        t[i] = (uint64_t)s;
        carry = s >> 64;
    }
    __uint128_t s4 = (__uint128_t)shifted[4] + mul977[4] + carry;
    t[4] = (uint64_t)s4;

    out->v[0] = t[0];
    out->v[1] = t[1];
    out->v[2] = t[2];
    out->v[3] = t[3];
    uint64_t high = t[4];
    if (high) {
        vibe_bench_fe_mul_small_add_shift32(out, high);
    } else if (vibe_bench_fe_ge_p(out)) {
        vibe_bench_fe_sub_p(out);
    }
}

static void vibe_bench_fe_mul(vibe_bench_fe256 *out, const vibe_bench_fe256 *a, const vibe_bench_fe256 *b) {
    uint64_t prod[8] = {0};
    for (int i = 0; i < 4; i++) {
        __uint128_t carry = 0;
        for (int j = 0; j < 4; j++) {
            __uint128_t cur = (__uint128_t)a->v[i] * b->v[j] + prod[i + j] + carry;
            prod[i + j] = (uint64_t)cur;
            carry = cur >> 64;
        }
        int k = i + 4;
        while (carry && k < 8) {
            __uint128_t cur = (__uint128_t)prod[k] + (uint64_t)carry;
            prod[k] = (uint64_t)cur;
            carry = cur >> 64;
            k += 1;
        }
    }
    vibe_bench_fe_reduce512(out, prod);
}

static void vibe_bench_fe_sqr(vibe_bench_fe256 *out, const vibe_bench_fe256 *a) {
    vibe_bench_fe_mul(out, a, a);
}

static int vibe_bench_fe_is_zero(const vibe_bench_fe256 *a) {
    return (a->v[0] | a->v[1] | a->v[2] | a->v[3]) == 0;
}

static void vibe_bench_fe_copy(vibe_bench_fe256 *out, const vibe_bench_fe256 *a) {
    out->v[0] = a->v[0];
    out->v[1] = a->v[1];
    out->v[2] = a->v[2];
    out->v[3] = a->v[3];
}

static void vibe_bench_fe_from_u64(vibe_bench_fe256 *out, uint64_t x) {
    out->v[0] = x;
    out->v[1] = 0;
    out->v[2] = 0;
    out->v[3] = 0;
}

static void vibe_bench_fe_pow_pminus2(vibe_bench_fe256 *out, const vibe_bench_fe256 *a) {
    const uint64_t exp[4] = {
        0xFFFFFFFEFFFFFC2Dull,
        0xFFFFFFFFFFFFFFFFull,
        0xFFFFFFFFFFFFFFFFull,
        0xFFFFFFFFFFFFFFFFull,
    };
    vibe_bench_fe256 result;
    vibe_bench_fe_from_u64(&result, 1);
    vibe_bench_fe256 base;
    vibe_bench_fe_copy(&base, a);
    for (int limb = 3; limb >= 0; limb--) {
        for (int bit = 63; bit >= 0; bit--) {
            vibe_bench_fe_sqr(&result, &result);
            if ((exp[limb] >> bit) & 1ull) {
                vibe_bench_fe_mul(&result, &result, &base);
            }
        }
    }
    vibe_bench_fe_copy(out, &result);
}

typedef struct vibe_bench_jacobian {
    vibe_bench_fe256 x;
    vibe_bench_fe256 y;
    vibe_bench_fe256 z;
} vibe_bench_jacobian;

static void vibe_bench_jacobian_zero(vibe_bench_jacobian *p) {
    vibe_bench_fe_from_u64(&p->x, 0);
    vibe_bench_fe_from_u64(&p->y, 1);
    vibe_bench_fe_from_u64(&p->z, 0);
}

static int vibe_bench_jacobian_is_zero(const vibe_bench_jacobian *p) {
    return vibe_bench_fe_is_zero(&p->z);
}

static void vibe_bench_jacobian_double(vibe_bench_jacobian *out, const vibe_bench_jacobian *p) {
    if (vibe_bench_jacobian_is_zero(p)) {
        *out = *p;
        return;
    }
    vibe_bench_fe256 a, b, c, d, e, f, tmp;
    vibe_bench_fe_sqr(&a, &p->x);
    vibe_bench_fe_sqr(&b, &p->y);
    vibe_bench_fe_sqr(&c, &b);

    vibe_bench_fe_add(&tmp, &p->x, &b);
    vibe_bench_fe_sqr(&tmp, &tmp);
    vibe_bench_fe_sub(&tmp, &tmp, &a);
    vibe_bench_fe_sub(&tmp, &tmp, &c);
    vibe_bench_fe_add(&d, &tmp, &tmp);

    vibe_bench_fe_add(&e, &a, &a);
    vibe_bench_fe_add(&e, &e, &a);
    vibe_bench_fe_sqr(&f, &e);

    vibe_bench_fe_add(&tmp, &d, &d);
    vibe_bench_fe_sub(&out->x, &f, &tmp);

    vibe_bench_fe_sub(&tmp, &d, &out->x);
    vibe_bench_fe_mul(&tmp, &e, &tmp);
    vibe_bench_fe_add(&c, &c, &c);
    vibe_bench_fe_add(&c, &c, &c);
    vibe_bench_fe_add(&c, &c, &c);
    vibe_bench_fe_sub(&out->y, &tmp, &c);

    vibe_bench_fe_mul(&tmp, &p->y, &p->z);
    vibe_bench_fe_add(&out->z, &tmp, &tmp);
}

static void vibe_bench_jacobian_add(
    vibe_bench_jacobian *out,
    const vibe_bench_jacobian *p,
    const vibe_bench_jacobian *q
) {
    if (vibe_bench_jacobian_is_zero(q)) {
        *out = *p;
        return;
    }
    if (vibe_bench_jacobian_is_zero(p)) {
        *out = *q;
        return;
    }
    vibe_bench_fe256 z1z1, z2z2, u1, u2, s1, s2, h, r, hh, hhh, v, tmp, x3, y3, z3;
    vibe_bench_fe_sqr(&z1z1, &p->z);
    vibe_bench_fe_sqr(&z2z2, &q->z);
    vibe_bench_fe_mul(&u1, &p->x, &z2z2);
    vibe_bench_fe_mul(&u2, &q->x, &z1z1);
    vibe_bench_fe_mul(&tmp, &q->z, &z2z2);
    vibe_bench_fe_mul(&s1, &p->y, &tmp);
    vibe_bench_fe_mul(&tmp, &p->z, &z1z1);
    vibe_bench_fe_mul(&s2, &q->y, &tmp);
    vibe_bench_fe_sub(&h, &u2, &u1);
    vibe_bench_fe_sub(&r, &s2, &s1);
    if (vibe_bench_fe_is_zero(&h)) {
        if (vibe_bench_fe_is_zero(&r)) {
            vibe_bench_jacobian_double(out, p);
        } else {
            vibe_bench_jacobian_zero(out);
        }
        return;
    }
    vibe_bench_fe_sqr(&hh, &h);
    vibe_bench_fe_mul(&hhh, &h, &hh);
    vibe_bench_fe_mul(&v, &u1, &hh);
    vibe_bench_fe_sqr(&tmp, &r);
    vibe_bench_fe_add(&x3, &v, &v);
    vibe_bench_fe_sub(&x3, &tmp, &x3);
    vibe_bench_fe_sub(&x3, &x3, &hhh);
    vibe_bench_fe_sub(&tmp, &v, &x3);
    vibe_bench_fe_mul(&tmp, &r, &tmp);
    vibe_bench_fe_mul(&y3, &s1, &hhh);
    vibe_bench_fe_sub(&y3, &tmp, &y3);
    vibe_bench_fe_mul(&tmp, &p->z, &q->z);
    vibe_bench_fe_mul(&z3, &tmp, &h);
    out->x = x3;
    out->y = y3;
    out->z = z3;
}

static void vibe_bench_jacobian_mul_scalar(
    vibe_bench_jacobian *out,
    const vibe_bench_jacobian *p,
    const uint64_t scalar[4]
) {
    vibe_bench_jacobian acc;
    vibe_bench_jacobian_zero(&acc);
    vibe_bench_jacobian d = *p;
    for (int limb = 0; limb < 4; limb++) {
        uint64_t w = scalar[limb];
        for (int bit = 0; bit < 64; bit++) {
            if (w & 1ull) {
                vibe_bench_jacobian tmp;
                vibe_bench_jacobian_add(&tmp, &acc, &d);
                acc = tmp;
            }
            vibe_bench_jacobian tmpd;
            vibe_bench_jacobian_double(&tmpd, &d);
            d = tmpd;
            w >>= 1;
        }
    }
    *out = acc;
}

static void vibe_bench_jacobian_to_affine(
    vibe_bench_fe256 *x,
    vibe_bench_fe256 *y,
    const vibe_bench_jacobian *p
) {
    if (vibe_bench_jacobian_is_zero(p)) {
        vibe_bench_fe_from_u64(x, 0);
        vibe_bench_fe_from_u64(y, 0);
        return;
    }
    vibe_bench_fe256 invz, invz2, invz3;
    vibe_bench_fe_pow_pminus2(&invz, &p->z);
    vibe_bench_fe_sqr(&invz2, &invz);
    vibe_bench_fe_mul(&invz3, &invz2, &invz);
    vibe_bench_fe_mul(x, &p->x, &invz2);
    vibe_bench_fe_mul(y, &p->y, &invz3);
}

static void vibe_bench_fe_to_hex(char out[65], const vibe_bench_fe256 *a) {
    static const char hex[] = "0123456789abcdef";
    unsigned char bytes[32];
    for (int i = 0; i < 4; i++) {
        uint64_t limb = a->v[i];
        for (int j = 0; j < 8; j++) {
            bytes[31 - (i * 8 + j)] = (unsigned char)((limb >> (8u * (uint32_t)j)) & 0xffu);
        }
    }
    for (int i = 0; i < 32; i++) {
        out[i * 2] = hex[(bytes[i] >> 4) & 0x0f];
        out[i * 2 + 1] = hex[bytes[i] & 0x0f];
    }
    out[64] = '\0';
}

char *vibe_bench_secp256k1(int64_t n) {
    if (n <= 0) {
        return vibe_bench_strdup_or_panic("");
    }
    const uint64_t priv[4] = {
        0xd9132d70b5bc7693ull,
        0xb64592e3fe0ab0daull,
        0x4fca3ef970ff4d38ull,
        0x2dee927079283c3cull,
    };
    vibe_bench_jacobian p;
    p.x.v[0] = 0x59f2815b16f81798ull;
    p.x.v[1] = 0x029bfcdb2dce28d9ull;
    p.x.v[2] = 0x55a06295ce870b07ull;
    p.x.v[3] = 0x79be667ef9dcbbacull;
    p.y.v[0] = 0x9c47d08ffb10d4b8ull;
    p.y.v[1] = 0xfd17b448a6855419ull;
    p.y.v[2] = 0x5da4fbfc0e1108a8ull;
    p.y.v[3] = 0x483ada7726a3c465ull;
    vibe_bench_fe_from_u64(&p.z, 1);

    for (int64_t i = 0; i < n; i++) {
        vibe_bench_jacobian next;
        vibe_bench_jacobian_mul_scalar(&next, &p, priv);
        p = next;
    }

    vibe_bench_fe256 ax, ay;
    vibe_bench_jacobian_to_affine(&ax, &ay, &p);
    char hx[65];
    char hy[65];
    vibe_bench_fe_to_hex(hx, &ax);
    vibe_bench_fe_to_hex(hy, &ay);

    size_t out_len = 64 + 1 + 64;
    char *out = (char *)calloc(out_len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate secp256k1 output");
    }
    memcpy(out, hx, 64);
    out[64] = ',';
    memcpy(out + 65, hy, 64);
    out[out_len] = '\0';
    return out;
}

// --- edigits (bench) ---

#define VIBE_BENCH_BIG_BASE 1000000000u

typedef struct vibe_bench_big {
    uint32_t *d; // little-endian base 1e9 limbs
    size_t len;
    size_t cap;
} vibe_bench_big;

static void vibe_bench_big_trim(vibe_bench_big *a) {
    while (a->len > 0 && a->d[a->len - 1] == 0) {
        a->len--;
    }
}

static void vibe_bench_big_reserve(vibe_bench_big *a, size_t cap) {
    if (cap <= a->cap) {
        return;
    }
    size_t new_cap = a->cap == 0 ? 4 : a->cap;
    while (new_cap < cap) {
        new_cap *= 2;
    }
    uint32_t *next = (uint32_t *)realloc(a->d, new_cap * sizeof(uint32_t));
    if (next == NULL) {
        vibe_panic("failed to allocate bigint");
    }
    a->d = next;
    a->cap = new_cap;
}

static void vibe_bench_big_init_u32(vibe_bench_big *a, uint32_t v) {
    a->d = NULL;
    a->len = 0;
    a->cap = 0;
    if (v == 0) {
        return;
    }
    vibe_bench_big_reserve(a, 1);
    a->d[0] = v;
    a->len = 1;
}

static void vibe_bench_big_copy(vibe_bench_big *out, const vibe_bench_big *a) {
    out->d = NULL;
    out->len = 0;
    out->cap = 0;
    if (a->len == 0) {
        return;
    }
    vibe_bench_big_reserve(out, a->len);
    memcpy(out->d, a->d, a->len * sizeof(uint32_t));
    out->len = a->len;
}

static void vibe_bench_big_free(vibe_bench_big *a) {
    free(a->d);
    a->d = NULL;
    a->len = 0;
    a->cap = 0;
}

static int vibe_bench_big_cmp(const vibe_bench_big *a, const vibe_bench_big *b) {
    if (a->len != b->len) {
        return a->len < b->len ? -1 : 1;
    }
    for (size_t i = a->len; i > 0; i--) {
        uint32_t av = a->d[i - 1];
        uint32_t bv = b->d[i - 1];
        if (av != bv) {
            return av < bv ? -1 : 1;
        }
    }
    return 0;
}

static void vibe_bench_big_add(vibe_bench_big *out, const vibe_bench_big *a, const vibe_bench_big *b) {
    size_t n = a->len > b->len ? a->len : b->len;
    vibe_bench_big_reserve(out, n + 1);
    uint64_t carry = 0;
    for (size_t i = 0; i < n; i++) {
        uint64_t av = i < a->len ? a->d[i] : 0;
        uint64_t bv = i < b->len ? b->d[i] : 0;
        uint64_t s = av + bv + carry;
        out->d[i] = (uint32_t)(s % VIBE_BENCH_BIG_BASE);
        carry = s / VIBE_BENCH_BIG_BASE;
    }
    if (carry) {
        out->d[n] = (uint32_t)carry;
        out->len = n + 1;
    } else {
        out->len = n;
    }
    vibe_bench_big_trim(out);
}

static void vibe_bench_big_add_inplace(vibe_bench_big *a, const vibe_bench_big *b) {
    vibe_bench_big tmp = {0};
    vibe_bench_big_add(&tmp, a, b);
    vibe_bench_big_free(a);
    *a = tmp;
}

static void vibe_bench_big_sub_inplace(vibe_bench_big *a, const vibe_bench_big *b) {
    uint64_t borrow = 0;
    for (size_t i = 0; i < a->len; i++) {
        uint64_t av = a->d[i];
        uint64_t bv = (i < b->len ? b->d[i] : 0) + borrow;
        if (av < bv) {
            a->d[i] = (uint32_t)(VIBE_BENCH_BIG_BASE + av - bv);
            borrow = 1;
        } else {
            a->d[i] = (uint32_t)(av - bv);
            borrow = 0;
        }
    }
    vibe_bench_big_trim(a);
}

static void vibe_bench_big_mul_small_inplace(vibe_bench_big *a, uint32_t m) {
    if (a->len == 0 || m == 1) {
        return;
    }
    if (m == 0) {
        a->len = 0;
        return;
    }
    vibe_bench_big_reserve(a, a->len + 1);
    uint64_t carry = 0;
    for (size_t i = 0; i < a->len; i++) {
        uint64_t cur = (uint64_t)a->d[i] * m + carry;
        a->d[i] = (uint32_t)(cur % VIBE_BENCH_BIG_BASE);
        carry = cur / VIBE_BENCH_BIG_BASE;
    }
    if (carry) {
        a->d[a->len++] = (uint32_t)carry;
    }
    vibe_bench_big_trim(a);
}

static uint32_t vibe_bench_big_div_small_inplace(vibe_bench_big *a, uint32_t v) {
    uint64_t rem = 0;
    for (size_t i = a->len; i > 0; i--) {
        uint64_t cur = a->d[i - 1] + rem * VIBE_BENCH_BIG_BASE;
        a->d[i - 1] = (uint32_t)(cur / v);
        rem = cur % v;
    }
    vibe_bench_big_trim(a);
    return (uint32_t)rem;
}

static void vibe_bench_big_mul(vibe_bench_big *out, const vibe_bench_big *a, const vibe_bench_big *b) {
    if (a->len == 0 || b->len == 0) {
        out->len = 0;
        return;
    }
    vibe_bench_big_reserve(out, a->len + b->len);
    memset(out->d, 0, (a->len + b->len) * sizeof(uint32_t));
    for (size_t i = 0; i < a->len; i++) {
        uint64_t carry = 0;
        for (size_t j = 0; j < b->len || carry; j++) {
            uint64_t cur = out->d[i + j] +
                           (uint64_t)a->d[i] * (j < b->len ? b->d[j] : 0) + carry;
            out->d[i + j] = (uint32_t)(cur % VIBE_BENCH_BIG_BASE);
            carry = cur / VIBE_BENCH_BIG_BASE;
        }
    }
    out->len = a->len + b->len;
    vibe_bench_big_trim(out);
}

static void vibe_bench_big_mul_small(vibe_bench_big *out, const vibe_bench_big *a, uint32_t m) {
    vibe_bench_big_copy(out, a);
    vibe_bench_big_mul_small_inplace(out, m);
}

static void vibe_bench_big_shift_base_add(vibe_bench_big *r, uint32_t digit) {
    vibe_bench_big_reserve(r, r->len + 1);
    if (r->len > 0) {
        memmove(r->d + 1, r->d, r->len * sizeof(uint32_t));
    }
    r->d[0] = digit;
    r->len += 1;
    vibe_bench_big_trim(r);
}

static void vibe_bench_big_div(vibe_bench_big *q, const vibe_bench_big *a1, const vibe_bench_big *b1) {
    if (b1->len == 0) {
        vibe_panic("division by zero bigint");
    }
    if (a1->len == 0) {
        q->len = 0;
        return;
    }
    if (vibe_bench_big_cmp(a1, b1) < 0) {
        q->len = 0;
        return;
    }
    uint32_t b_msd = b1->d[b1->len - 1];
    uint32_t norm = (uint32_t)(VIBE_BENCH_BIG_BASE / ((uint64_t)b_msd + 1));

    vibe_bench_big a = {0};
    vibe_bench_big b = {0};
    vibe_bench_big_copy(&a, a1);
    vibe_bench_big_copy(&b, b1);
    if (norm != 1) {
        vibe_bench_big_mul_small_inplace(&a, norm);
        vibe_bench_big_mul_small_inplace(&b, norm);
    }

    vibe_bench_big r = {0};
    q->len = 0;
    vibe_bench_big_reserve(q, a.len);
    memset(q->d, 0, a.len * sizeof(uint32_t));
    q->len = a.len;

    for (size_t i = a.len; i > 0; i--) {
        vibe_bench_big_shift_base_add(&r, a.d[i - 1]);
        uint32_t s1 = r.len <= b.len ? 0 : r.d[b.len];
        uint32_t s2 = r.len <= b.len - 1 ? 0 : r.d[b.len - 1];
        uint64_t d_est = ((uint64_t)s1 * VIBE_BENCH_BIG_BASE + s2) / b.d[b.len - 1];
        if (d_est >= VIBE_BENCH_BIG_BASE) {
            d_est = VIBE_BENCH_BIG_BASE - 1;
        }

        vibe_bench_big bd = {0};
        vibe_bench_big_mul_small(&bd, &b, (uint32_t)d_est);
        while (vibe_bench_big_cmp(&r, &bd) < 0) {
            vibe_bench_big_free(&bd);
            d_est -= 1;
            vibe_bench_big_mul_small(&bd, &b, (uint32_t)d_est);
        }
        vibe_bench_big_sub_inplace(&r, &bd);
        vibe_bench_big_free(&bd);
        q->d[i - 1] = (uint32_t)d_est;
    }
    vibe_bench_big_trim(q);
    if (norm != 1) {
        (void)vibe_bench_big_div_small_inplace(&r, norm);
    }
    vibe_bench_big_free(&a);
    vibe_bench_big_free(&b);
    vibe_bench_big_free(&r);
}

static char *vibe_bench_big_to_dec_str(const vibe_bench_big *a) {
    if (a->len == 0) {
        return vibe_bench_strdup_or_panic("0");
    }
    size_t approx = a->len * 9 + 1;
    char *out = (char *)calloc(approx + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate bigint string");
    }
    char *p = out;
    int wrote = snprintf(p, approx + 1, "%u", a->d[a->len - 1]);
    if (wrote < 0) {
        out[0] = '\0';
        return out;
    }
    p += wrote;
    for (size_t i = a->len - 1; i > 0; i--) {
        wrote = snprintf(p, approx + 1 - (size_t)(p - out), "%09u", a->d[i - 1]);
        if (wrote < 0) {
            break;
        }
        p += wrote;
    }
    return out;
}

typedef struct vibe_bench_pq {
    vibe_bench_big p;
    vibe_bench_big q;
} vibe_bench_pq;

static vibe_bench_pq vibe_bench_sum_terms(uint32_t a, uint32_t b) {
    if (b == a + 1) {
        vibe_bench_pq base = {0};
        vibe_bench_big_init_u32(&base.p, 1);
        vibe_bench_big_init_u32(&base.q, b);
        return base;
    }
    uint32_t mid = (a + b) / 2;
    vibe_bench_pq left = vibe_bench_sum_terms(a, mid);
    vibe_bench_pq right = vibe_bench_sum_terms(mid, b);

    vibe_bench_big p_left_q_right = {0};
    vibe_bench_big_mul(&p_left_q_right, &left.p, &right.q);
    vibe_bench_big_add_inplace(&p_left_q_right, &right.p);

    vibe_bench_big q_left_q_right = {0};
    vibe_bench_big_mul(&q_left_q_right, &left.q, &right.q);

    vibe_bench_big_free(&left.p);
    vibe_bench_big_free(&left.q);
    vibe_bench_big_free(&right.p);
    vibe_bench_big_free(&right.q);

    vibe_bench_pq out = {0};
    out.p = p_left_q_right;
    out.q = q_left_q_right;
    return out;
}

static uint32_t vibe_bench_edigits_find_k(int64_t n) {
    int64_t target = n + 50;
    uint32_t a = 0;
    uint32_t b = 1;
    while (1) {
        double k = (double)b;
        if (k > 0) {
            double ln_k_fact =
                k * (log(k) - 1.0) + 0.5 * log(2.0 * 3.14159265358979323846 * k);
            double log10_k_fact = ln_k_fact / log(10.0);
            if (log10_k_fact >= (double)target) {
                break;
            }
        }
        a = b;
        b *= 2;
    }
    while (b - a > 1) {
        uint32_t m = a + (b - a) / 2;
        double k = (double)m;
        double ln_k_fact =
            k * (log(k) - 1.0) + 0.5 * log(2.0 * 3.14159265358979323846 * k);
        double log10_k_fact = ln_k_fact / log(10.0);
        if (log10_k_fact >= (double)target) {
            b = m;
        } else {
            a = m;
        }
    }
    return b;
}

static char *vibe_bench_edigits_calculate(int64_t n) {
    if (n <= 0) {
        return vibe_bench_strdup_or_panic("");
    }
    uint32_t k = vibe_bench_edigits_find_k(n);
    vibe_bench_pq pq = vibe_bench_sum_terms(0, k - 1);
    vibe_bench_big_add_inplace(&pq.p, &pq.q);

    int64_t exp = n - 1;
    uint32_t shift = (uint32_t)(exp / 9);
    uint32_t rem = (uint32_t)(exp % 9);
    uint32_t pow10 = 1;
    for (uint32_t i = 0; i < rem; i++) {
        pow10 *= 10u;
    }
    vibe_bench_big_mul_small_inplace(&pq.p, pow10);
    if (shift > 0 && pq.p.len > 0) {
        vibe_bench_big_reserve(&pq.p, pq.p.len + shift);
        memmove(pq.p.d + shift, pq.p.d, pq.p.len * sizeof(uint32_t));
        memset(pq.p.d, 0, shift * sizeof(uint32_t));
        pq.p.len += shift;
    }
    vibe_bench_big_trim(&pq.p);

    vibe_bench_big answer = {0};
    vibe_bench_big_div(&answer, &pq.p, &pq.q);
    char *s = vibe_bench_big_to_dec_str(&answer);

    vibe_bench_big_free(&pq.p);
    vibe_bench_big_free(&pq.q);
    vibe_bench_big_free(&answer);

    size_t len = strlen(s);
    if ((int64_t)len >= n) {
        return s;
    }
    size_t pad = (size_t)(n - (int64_t)len);
    char *out = (char *)calloc((size_t)n + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate edigits string");
    }
    memset(out, '0', pad);
    memcpy(out + pad, s, len);
    out[n] = '\0';
    free(s);
    return out;
}

char *vibe_bench_edigits(int64_t n) {
    if (n <= 0) {
        n = 27;
    }
    char *digits = vibe_bench_edigits_calculate(n);
    vibe_bench_string_builder builder;
    vibe_bench_builder_init(&builder, (size_t)n + 64);
    for (int64_t i = 0; i < n; i += 10) {
        int64_t count = i + 10 <= n ? i + 10 : n;
        char line[16];
        memset(line, ' ', 10);
        int64_t take = count - i;
        memcpy(line, digits + i, (size_t)take);
        line[10] = '\0';
        char suffix[32];
        int suffix_len = 0;
        if (count < n) {
            suffix_len = snprintf(suffix, sizeof(suffix), "\t:%lld\n", (long long)count);
        } else {
            suffix_len = snprintf(suffix, sizeof(suffix), "\t:%lld", (long long)count);
        }
        vibe_bench_builder_append_bytes(&builder, line, 10);
        vibe_bench_builder_append_bytes(&builder, suffix, (size_t)(suffix_len > 0 ? suffix_len : 0));
    }
    free(digits);
    return builder.data;
}

