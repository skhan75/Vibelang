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

void vibe_sleep_ms(int64_t ms) {
    if (ms <= 0) {
        return;
    }
    usleep((useconds_t)(ms * 1000));
}
