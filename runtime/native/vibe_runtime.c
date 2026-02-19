#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <pthread.h>
#include <time.h>
#include <unistd.h>

typedef struct vibe_chan_i64 {
    pthread_mutex_t mu;
    pthread_cond_t can_send;
    pthread_cond_t can_recv;
    int64_t *buffer;
    int64_t capacity;
    int64_t head;
    int64_t tail;
    int64_t size;
    int closed;
} vibe_chan_i64;

typedef struct vibe_spawn0_ctx {
    int64_t (*fn)(void);
} vibe_spawn0_ctx;

typedef struct vibe_spawn1_i64_ctx {
    int64_t (*fn)(int64_t);
    int64_t arg0;
} vibe_spawn1_i64_ctx;

static pthread_mutex_t vibe_select_cursor_mu = PTHREAD_MUTEX_INITIALIZER;
static uint64_t vibe_select_cursor = 0;

void vibe_println(const char *s) {
    if (s == NULL) {
        puts("(null)");
        return;
    }
    puts(s);
}

void vibe_panic(const char *s) {
    if (s == NULL) {
        fputs("panic: (null)\n", stderr);
    } else {
        fputs("panic: ", stderr);
        fputs(s, stderr);
        fputs("\n", stderr);
    }
    abort();
}

void vibe_exit(int code) {
    exit(code);
}

void *vibe_chan_new_i64(int64_t capacity) {
    if (capacity <= 0) {
        capacity = 1;
    }
    vibe_chan_i64 *ch = (vibe_chan_i64 *)calloc(1, sizeof(vibe_chan_i64));
    if (ch == NULL) {
        vibe_panic("failed to allocate channel");
    }
    ch->buffer = (int64_t *)calloc((size_t)capacity, sizeof(int64_t));
    if (ch->buffer == NULL) {
        free(ch);
        vibe_panic("failed to allocate channel buffer");
    }
    ch->capacity = capacity;
    ch->head = 0;
    ch->tail = 0;
    ch->size = 0;
    ch->closed = 0;
    pthread_mutex_init(&ch->mu, NULL);
    pthread_cond_init(&ch->can_send, NULL);
    pthread_cond_init(&ch->can_recv, NULL);
    return (void *)ch;
}

int64_t vibe_chan_send_i64(void *handle, int64_t value) {
    vibe_chan_i64 *ch = (vibe_chan_i64 *)handle;
    if (ch == NULL) {
        return 1;
    }
    pthread_mutex_lock(&ch->mu);
    while (!ch->closed && ch->size >= ch->capacity) {
        pthread_cond_wait(&ch->can_send, &ch->mu);
    }
    if (ch->closed) {
        pthread_mutex_unlock(&ch->mu);
        return 1;
    }
    ch->buffer[ch->tail] = value;
    ch->tail = (ch->tail + 1) % ch->capacity;
    ch->size += 1;
    pthread_cond_signal(&ch->can_recv);
    pthread_mutex_unlock(&ch->mu);
    return 0;
}

int64_t vibe_chan_recv_i64(void *handle) {
    vibe_chan_i64 *ch = (vibe_chan_i64 *)handle;
    if (ch == NULL) {
        return 0;
    }
    pthread_mutex_lock(&ch->mu);
    while (!ch->closed && ch->size == 0) {
        pthread_cond_wait(&ch->can_recv, &ch->mu);
    }
    if (ch->size == 0) {
        pthread_mutex_unlock(&ch->mu);
        return 0;
    }
    int64_t value = ch->buffer[ch->head];
    ch->head = (ch->head + 1) % ch->capacity;
    ch->size -= 1;
    pthread_cond_signal(&ch->can_send);
    pthread_mutex_unlock(&ch->mu);
    return value;
}

int64_t vibe_chan_try_recv_i64(void *handle, int64_t *out_value) {
    vibe_chan_i64 *ch = (vibe_chan_i64 *)handle;
    if (ch == NULL) {
        return 2;
    }
    pthread_mutex_lock(&ch->mu);
    if (ch->size > 0) {
        int64_t value = ch->buffer[ch->head];
        ch->head = (ch->head + 1) % ch->capacity;
        ch->size -= 1;
        pthread_cond_signal(&ch->can_send);
        pthread_mutex_unlock(&ch->mu);
        if (out_value != NULL) {
            *out_value = value;
        }
        return 1;
    }
    int64_t closed = ch->closed ? 1 : 0;
    pthread_mutex_unlock(&ch->mu);
    return closed ? 2 : 0;
}

int64_t vibe_chan_has_data_i64(void *handle) {
    vibe_chan_i64 *ch = (vibe_chan_i64 *)handle;
    if (ch == NULL) {
        return 0;
    }
    pthread_mutex_lock(&ch->mu);
    int64_t ready = ch->size > 0 ? 1 : 0;
    pthread_mutex_unlock(&ch->mu);
    return ready;
}

void vibe_chan_close_i64(void *handle) {
    vibe_chan_i64 *ch = (vibe_chan_i64 *)handle;
    if (ch == NULL) {
        return;
    }
    pthread_mutex_lock(&ch->mu);
    ch->closed = 1;
    pthread_cond_broadcast(&ch->can_send);
    pthread_cond_broadcast(&ch->can_recv);
    pthread_mutex_unlock(&ch->mu);
}

int64_t vibe_chan_is_closed_i64(void *handle) {
    vibe_chan_i64 *ch = (vibe_chan_i64 *)handle;
    if (ch == NULL) {
        return 1;
    }
    pthread_mutex_lock(&ch->mu);
    int64_t result = ch->closed ? 1 : 0;
    pthread_mutex_unlock(&ch->mu);
    return result;
}

static void *vibe_spawn0_entry(void *opaque) {
    vibe_spawn0_ctx *ctx = (vibe_spawn0_ctx *)opaque;
    if (ctx != NULL && ctx->fn != NULL) {
        ctx->fn();
    }
    free(ctx);
    return NULL;
}

static void *vibe_spawn1_i64_entry(void *opaque) {
    vibe_spawn1_i64_ctx *ctx = (vibe_spawn1_i64_ctx *)opaque;
    if (ctx != NULL && ctx->fn != NULL) {
        ctx->fn(ctx->arg0);
    }
    free(ctx);
    return NULL;
}

int64_t vibe_spawn0(void *fn_ptr) {
    if (fn_ptr == NULL) {
        return 1;
    }
    vibe_spawn0_ctx *ctx = (vibe_spawn0_ctx *)calloc(1, sizeof(vibe_spawn0_ctx));
    if (ctx == NULL) {
        return 1;
    }
    ctx->fn = (int64_t (*)(void))fn_ptr;
    pthread_t tid;
    int rc = pthread_create(&tid, NULL, vibe_spawn0_entry, ctx);
    if (rc != 0) {
        free(ctx);
        return 1;
    }
    pthread_detach(tid);
    return 0;
}

int64_t vibe_spawn1_i64(void *fn_ptr, int64_t arg0) {
    if (fn_ptr == NULL) {
        return 1;
    }
    vibe_spawn1_i64_ctx *ctx = (vibe_spawn1_i64_ctx *)calloc(1, sizeof(vibe_spawn1_i64_ctx));
    if (ctx == NULL) {
        return 1;
    }
    ctx->fn = (int64_t (*)(int64_t))fn_ptr;
    ctx->arg0 = arg0;
    pthread_t tid;
    int rc = pthread_create(&tid, NULL, vibe_spawn1_i64_entry, ctx);
    if (rc != 0) {
        free(ctx);
        return 1;
    }
    pthread_detach(tid);
    return 0;
}

int64_t vibe_select_next_cursor(int64_t case_count) {
    if (case_count <= 0) {
        return 0;
    }
    pthread_mutex_lock(&vibe_select_cursor_mu);
    uint64_t current = vibe_select_cursor % (uint64_t)case_count;
    vibe_select_cursor += 1;
    pthread_mutex_unlock(&vibe_select_cursor_mu);
    return (int64_t)current;
}

void vibe_sleep_ms(int64_t ms) {
    if (ms <= 0) {
        return;
    }
    struct timespec req;
    req.tv_sec = (time_t)(ms / 1000);
    req.tv_nsec = (long)((ms % 1000) * 1000000);
    nanosleep(&req, NULL);
}
