#define _POSIX_C_SOURCE 200809L
#if defined(__linux__)
#define _DEFAULT_SOURCE 1
#endif
#if defined(__APPLE__)
#include <sys/random.h>
#include <crt_externs.h>
#endif
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <strings.h>
#include <ctype.h>
#include <errno.h>
#include <pthread.h>
#include <math.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>
#ifdef _WIN32
#include <direct.h>
#include <windows.h>
#include <bcrypt.h>
/*
 * NT_SUCCESS lives in <ntdef.h>/<winternl.h>, which <windows.h> does not pull
 * in under MinGW, so `BCryptGenRandom`'s NTSTATUS could be obtained but not
 * tested: the Windows install smoke failed with
 * `error: implicit declaration of function 'NT_SUCCESS'`, which meant the
 * packaged Windows compiler could not build any program at all, hello world
 * included. Defining it here rather than including <winternl.h> keeps the
 * internal-API surface out of the runtime; the test is a documented one-liner.
 */
#ifndef NT_SUCCESS
#define NT_SUCCESS(Status) (((NTSTATUS)(Status)) >= 0)
#endif
#else
#include <regex.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/time.h>
#endif

typedef struct vibe_chan_i64 {
    uint64_t magic;
    pthread_mutex_t mu;
    pthread_cond_t can_send;
    pthread_cond_t can_recv;
    int64_t *buffer;
    int64_t capacity;
    int64_t head;
    int64_t tail;
    int64_t size;
    int closed;
    int64_t send_fast_path_hits;
    int64_t recv_fast_path_hits;
    int64_t send_slow_path_hits;
    int64_t recv_slow_path_hits;
    int64_t send_wait_count;
    int64_t recv_wait_count;
    int64_t contention_count;
} vibe_chan_i64;

typedef struct vibe_spawn0_ctx {
    int64_t (*fn)(void);
} vibe_spawn0_ctx;

typedef struct vibe_spawn1_i64_ctx {
    int64_t (*fn)(int64_t);
    int64_t arg0;
} vibe_spawn1_i64_ctx;

enum {
    VIBE_CONTAINER_LIST_I64 = 1,
    VIBE_CONTAINER_MAP_I64_I64 = 2,
    VIBE_CONTAINER_MAP_STR_I64 = 3,
    VIBE_CONTAINER_MAP_STR_STR = 4,
    VIBE_CONTAINER_BYTES = 5,
};

typedef struct vibe_list_i64 {
    int64_t tag;
    int64_t len;
    int64_t cap;
    int64_t *items;
} vibe_list_i64;

typedef struct vibe_map_i64_entry {
    int64_t key;
    int64_t value;
} vibe_map_i64_entry;

typedef struct vibe_map_i64_i64 {
    int64_t tag;
    int64_t len;
    int64_t cap;
    vibe_map_i64_entry *entries;
    int64_t hash_cap;
    int64_t hash_len;
    int64_t *hash_slots;
    int64_t collision_count;
    int64_t resize_count;
    int64_t probe_steps;
} vibe_map_i64_i64;

typedef struct vibe_map_str_entry {
    char *key;
    int64_t value;
} vibe_map_str_entry;

typedef struct vibe_map_str_i64 {
    int64_t tag;
    int64_t len;
    int64_t cap;
    vibe_map_str_entry *entries;
    int64_t hash_cap;
    int64_t hash_len;
    int64_t *hash_slots;
    int64_t collision_count;
    int64_t resize_count;
    int64_t probe_steps;
} vibe_map_str_i64;

typedef struct vibe_map_str_str_entry {
    char *key;
    char *value;
    int64_t hash;
} vibe_map_str_str_entry;

typedef struct vibe_map_str_str {
    int64_t tag;
    vibe_map_str_str_entry *entries;
    int64_t count;
    int64_t capacity;
    int64_t *hash_buckets;
    int64_t hash_capacity;
} vibe_map_str_str;

static pthread_mutex_t vibe_select_cursor_mu = PTHREAD_MUTEX_INITIALIZER;
static uint64_t vibe_select_cursor = 0;
static int64_t vibe_json_parse_calls = 0;
static int64_t vibe_json_stringify_calls = 0;
static int64_t vibe_json_minify_calls = 0;
static int64_t vibe_json_canonical_calls = 0;
static int64_t vibe_json_validate_calls = 0;
static int64_t vibe_json_allocations = 0;

static void vibe_counter_inc(int64_t *counter) {
    __sync_fetch_and_add(counter, 1);
}

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

/*
 * Live-handle registry.
 *
 * The str_builder, chan, and net/ws stdlib APIs hand out opaque Int
 * handles (heap pointers or socket fds), so generated code can pass any
 * integer back in. Every handle-consuming native first checks the value
 * against its family's live-handle set and panics with a clear message
 * instead of dereferencing a forged, stale, or arithmetically-derived
 * handle (or touching an unrelated file descriptor). Pointer-backed
 * structs additionally carry a magic word (same idea as the container
 * tags below) that is cleared when the handle is freed.
 */
#define VIBE_HANDLE_TOMBSTONE ((uintptr_t)-1)

typedef struct vibe_handle_set {
    pthread_mutex_t mu;
    uintptr_t *slots; /* open addressing; 0 = empty, VIBE_HANDLE_TOMBSTONE = deleted */
    size_t cap;       /* power of two; 0 until first insert */
    size_t len;       /* live keys */
    size_t used;      /* live keys + tombstones */
} vibe_handle_set;

static vibe_handle_set vibe_str_builder_live = {PTHREAD_MUTEX_INITIALIZER, NULL, 0, 0, 0};
static vibe_handle_set vibe_chan_live = {PTHREAD_MUTEX_INITIALIZER, NULL, 0, 0, 0};
static vibe_handle_set vibe_net_live = {PTHREAD_MUTEX_INITIALIZER, NULL, 0, 0, 0};

static size_t vibe_handle_slot_start(uintptr_t key, size_t cap) {
    uint64_t h = (uint64_t)key * UINT64_C(0x9E3779B97F4A7C15);
    return (size_t)(h & (uint64_t)(cap - 1));
}

/* set->mu must be held. Keeps at least half of the slots empty so probe
 * loops always terminate. */
static void vibe_handle_set_grow(vibe_handle_set *set) {
    size_t new_cap = set->cap == 0 ? 64 : set->cap * 2;
    uintptr_t *new_slots = (uintptr_t *)calloc(new_cap, sizeof(uintptr_t));
    if (new_slots == NULL) {
        vibe_panic("failed to allocate handle registry");
    }
    for (size_t i = 0; i < set->cap; i++) {
        uintptr_t key = set->slots[i];
        if (key == 0 || key == VIBE_HANDLE_TOMBSTONE) {
            continue;
        }
        size_t j = vibe_handle_slot_start(key, new_cap);
        while (new_slots[j] != 0) {
            j = (j + 1) & (new_cap - 1);
        }
        new_slots[j] = key;
    }
    free(set->slots);
    set->slots = new_slots;
    set->cap = new_cap;
    set->used = set->len;
}

/* Idempotent: registering an already-live key (an OS fd number being
 * handed out again) is a no-op. */
static void vibe_handle_register(vibe_handle_set *set, uintptr_t key) {
    if (key == 0 || key == VIBE_HANDLE_TOMBSTONE) {
        vibe_panic("handle registry rejects sentinel value");
    }
    pthread_mutex_lock(&set->mu);
    if (set->cap == 0 || (set->used + 1) * 2 > set->cap) {
        vibe_handle_set_grow(set);
    }
    size_t insert_at = (size_t)-1;
    size_t i = vibe_handle_slot_start(key, set->cap);
    while (set->slots[i] != 0) {
        if (set->slots[i] == key) {
            pthread_mutex_unlock(&set->mu);
            return;
        }
        if (set->slots[i] == VIBE_HANDLE_TOMBSTONE && insert_at == (size_t)-1) {
            insert_at = i;
        }
        i = (i + 1) & (set->cap - 1);
    }
    if (insert_at == (size_t)-1) {
        insert_at = i;
        set->used += 1;
    }
    set->slots[insert_at] = key;
    set->len += 1;
    pthread_mutex_unlock(&set->mu);
}

static int vibe_handle_is_live(vibe_handle_set *set, uintptr_t key) {
    if (key == 0 || key == VIBE_HANDLE_TOMBSTONE) {
        return 0;
    }
    pthread_mutex_lock(&set->mu);
    int found = 0;
    if (set->cap > 0) {
        size_t i = vibe_handle_slot_start(key, set->cap);
        while (set->slots[i] != 0) {
            if (set->slots[i] == key) {
                found = 1;
                break;
            }
            i = (i + 1) & (set->cap - 1);
        }
    }
    pthread_mutex_unlock(&set->mu);
    return found;
}

static void vibe_handle_unregister(vibe_handle_set *set, uintptr_t key) {
    if (key == 0 || key == VIBE_HANDLE_TOMBSTONE) {
        return;
    }
    pthread_mutex_lock(&set->mu);
    if (set->cap > 0) {
        size_t i = vibe_handle_slot_start(key, set->cap);
        while (set->slots[i] != 0) {
            if (set->slots[i] == key) {
                set->slots[i] = VIBE_HANDLE_TOMBSTONE;
                set->len -= 1;
                break;
            }
            i = (i + 1) & (set->cap - 1);
        }
    }
    pthread_mutex_unlock(&set->mu);
}

static void vibe_handle_check(vibe_handle_set *set, uintptr_t key, const char *message) {
    if (!vibe_handle_is_live(set, key)) {
        vibe_panic(message);
    }
}

/* Heap records: fixed 8-byte slots, slot_count slots. Returns aligned pointer. */
void *vibe_record_alloc(int64_t slot_count) {
    if (slot_count <= 0) {
        vibe_panic("vibe_record_alloc: invalid slot_count");
    }
    size_t size = (size_t)slot_count * 8;
    void *ptr = calloc(1, size);
    if (ptr == NULL) {
        vibe_panic("vibe_record_alloc: allocation failed");
    }
    return ptr;
}

static char *vibe_strdup_or_panic(const char *src) {
    if (src == NULL) {
        char *empty = (char *)calloc(1, sizeof(char));
        if (empty == NULL) {
            vibe_panic("failed to allocate empty string");
        }
        empty[0] = '\0';
        return empty;
    }
    size_t len = strlen(src);
    char *copy = (char *)calloc(len + 1, sizeof(char));
    if (copy == NULL) {
        vibe_panic("failed to allocate string copy");
    }
    memcpy(copy, src, len);
    copy[len] = '\0';
    return copy;
}

typedef struct vibe_string_builder {
    char *data;
    size_t len;
    size_t cap;
} vibe_string_builder;

static void vibe_builder_init(vibe_string_builder *builder, size_t initial_cap) {
    if (builder == NULL) {
        vibe_panic("string builder is null");
    }
    size_t cap = initial_cap < 64 ? 64 : initial_cap;
    builder->data = (char *)calloc(cap, sizeof(char));
    if (builder->data == NULL) {
        vibe_panic("failed to allocate string builder");
    }
    builder->len = 0;
    builder->cap = cap;
    builder->data[0] = '\0';
}

static void vibe_builder_reserve(vibe_string_builder *builder, size_t extra_len) {
    if (builder == NULL) {
        vibe_panic("string builder is null");
    }
    size_t needed = builder->len + extra_len + 1;
    if (needed <= builder->cap) {
        return;
    }
    size_t next_cap = builder->cap;
    while (next_cap < needed) {
        next_cap *= 2;
    }
    char *next = (char *)realloc(builder->data, next_cap * sizeof(char));
    if (next == NULL) {
        vibe_panic("failed to grow string builder");
    }
    builder->data = next;
    builder->cap = next_cap;
}

static void vibe_builder_append_bytes(vibe_string_builder *builder, const char *bytes, size_t len) {
    if (builder == NULL) {
        vibe_panic("string builder is null");
    }
    if (len == 0) {
        return;
    }
    if (bytes == NULL) {
        vibe_panic("attempted to append null bytes into string builder");
    }
    vibe_builder_reserve(builder, len);
    memcpy(builder->data + builder->len, bytes, len);
    builder->len += len;
    builder->data[builder->len] = '\0';
}

static int64_t vibe_utf8_is_boundary(const char *text, int64_t index, int64_t len) {
    if (index < 0 || index > len) {
        return 0;
    }
    if (index == 0 || index == len) {
        return 1;
    }
    unsigned char ch = (unsigned char)text[index];
    return (ch & 0xC0u) != 0x80u;
}

static void vibe_list_ensure_capacity(vibe_list_i64 *list, int64_t min_cap) {
    if (list == NULL) {
        vibe_panic("list handle is null");
    }
    if (list->cap >= min_cap) {
        return;
    }
    int64_t next_cap = list->cap <= 0 ? 4 : list->cap;
    while (next_cap < min_cap) {
        next_cap *= 2;
    }
    int64_t *next_items = (int64_t *)calloc((size_t)next_cap, sizeof(int64_t));
    if (next_items == NULL) {
        vibe_panic("failed to grow list buffer");
    }
    if (list->items != NULL && list->len > 0) {
        memcpy(next_items, list->items, (size_t)list->len * sizeof(int64_t));
    }
    free(list->items);
    list->items = next_items;
    list->cap = next_cap;
}

void *vibe_list_new_i64(int64_t capacity) {
    if (capacity < 0) {
        capacity = 0;
    }
    vibe_list_i64 *list = (vibe_list_i64 *)calloc(1, sizeof(vibe_list_i64));
    if (list == NULL) {
        vibe_panic("failed to allocate list");
    }
    list->tag = VIBE_CONTAINER_LIST_I64;
    list->len = 0;
    list->cap = 0;
    list->items = NULL;
    if (capacity > 0) {
        vibe_list_ensure_capacity(list, capacity);
    }
    return (void *)list;
}

int64_t vibe_list_append_i64(void *handle, int64_t value) {
    vibe_list_i64 *list = (vibe_list_i64 *)handle;
    if (list == NULL || list->tag != VIBE_CONTAINER_LIST_I64) {
        vibe_panic("list append called on non-list handle");
    }
    vibe_list_ensure_capacity(list, list->len + 1);
    list->items[list->len] = value;
    list->len += 1;
    return list->len;
}

int64_t vibe_list_get_i64(void *handle, int64_t index) {
    vibe_list_i64 *list = (vibe_list_i64 *)handle;
    if (list == NULL || list->tag != VIBE_CONTAINER_LIST_I64) {
        vibe_panic("list get called on non-list handle");
    }
    if (index < 0 || index >= list->len) {
        vibe_panic("list index out of bounds");
    }
    return list->items[index];
}

int64_t vibe_list_set_i64(void *handle, int64_t index, int64_t value) {
    vibe_list_i64 *list = (vibe_list_i64 *)handle;
    if (list == NULL || list->tag != VIBE_CONTAINER_LIST_I64) {
        vibe_panic("list set called on non-list handle");
    }
    if (index < 0 || index >= list->len) {
        vibe_panic("list index out of bounds");
    }
    list->items[index] = value;
    return 0;
}

int64_t vibe_list_len_i64(void *handle) {
    vibe_list_i64 *list = (vibe_list_i64 *)handle;
    if (list == NULL || list->tag != VIBE_CONTAINER_LIST_I64) {
        vibe_panic("list len called on non-list handle");
    }
    return list->len;
}

static int vibe_list_sort_desc_i64_cmp(const void *lhs, const void *rhs) {
    int64_t left = *(const int64_t *)lhs;
    int64_t right = *(const int64_t *)rhs;
    if (left < right) {
        return 1;
    }
    if (left > right) {
        return -1;
    }
    return 0;
}

void *vibe_list_sort_desc_i64(void *handle) {
    vibe_list_i64 *list = (vibe_list_i64 *)handle;
    if (list == NULL || list->tag != VIBE_CONTAINER_LIST_I64) {
        vibe_panic("list sort_desc called on non-list handle");
    }
    vibe_list_i64 *out = (vibe_list_i64 *)vibe_list_new_i64(list->len);
    out->len = list->len;
    if (list->len > 0) {
        memcpy(out->items, list->items, (size_t)list->len * sizeof(int64_t));
        qsort(
            out->items,
            (size_t)out->len,
            sizeof(int64_t),
            vibe_list_sort_desc_i64_cmp
        );
    }
    return (void *)out;
}

void *vibe_list_take_i64(void *handle, int64_t count) {
    vibe_list_i64 *list = (vibe_list_i64 *)handle;
    if (list == NULL || list->tag != VIBE_CONTAINER_LIST_I64) {
        vibe_panic("list take called on non-list handle");
    }
    if (count < 0) {
        count = 0;
    }
    if (count > list->len) {
        count = list->len;
    }
    vibe_list_i64 *out = (vibe_list_i64 *)vibe_list_new_i64(count);
    out->len = count;
    if (count > 0) {
        memcpy(out->items, list->items, (size_t)count * sizeof(int64_t));
    }
    return (void *)out;
}

static uint64_t vibe_hash_u64(uint64_t value) {
    value ^= value >> 33;
    value *= 0xff51afd7ed558ccdull;
    value ^= value >> 33;
    value *= 0xc4ceb9fe1a85ec53ull;
    value ^= value >> 33;
    return value;
}

static uint64_t vibe_hash_i64_key(int64_t key) {
    return vibe_hash_u64((uint64_t)key ^ 0x9e3779b97f4a7c15ull);
}

static uint64_t vibe_hash_cstr_key(const char *key) {
    const unsigned char *bytes = (const unsigned char *)(key == NULL ? "" : key);
    uint64_t hash = 1469598103934665603ull;
    while (*bytes != '\0') {
        hash ^= (uint64_t)(*bytes);
        hash *= 1099511628211ull;
        bytes += 1;
    }
    return hash;
}

static int64_t vibe_next_hash_capacity(int64_t min_cap) {
    int64_t cap = 16;
    while (cap < min_cap) {
        cap *= 2;
    }
    return cap;
}

static void vibe_map_i64_hash_ensure(vibe_map_i64_i64 *map, int64_t min_cap);
static void vibe_map_str_hash_ensure(vibe_map_str_i64 *map, int64_t min_cap);
static void vibe_map_str_str_hash_ensure(vibe_map_str_str *map, int64_t min_cap);

static int64_t vibe_map_i64_find_slot(
    vibe_map_i64_i64 *map,
    int64_t key,
    int64_t *found,
    int64_t *probe_count
) {
    if (map->hash_cap <= 0 || map->hash_slots == NULL) {
        *found = 0;
        if (probe_count != NULL) {
            *probe_count = 0;
        }
        return -1;
    }
    uint64_t hash = vibe_hash_i64_key(key);
    int64_t mask = map->hash_cap - 1;
    int64_t slot = (int64_t)(hash & (uint64_t)mask);
    int64_t first_tombstone = -1;
    int64_t probes = 0;
    while (1) {
        probes += 1;
        int64_t marker = map->hash_slots[slot];
        if (marker == 0) {
            *found = 0;
            if (probe_count != NULL) {
                *probe_count = probes;
            }
            return first_tombstone >= 0 ? first_tombstone : slot;
        }
        if (marker < 0) {
            if (first_tombstone < 0) {
                first_tombstone = slot;
            }
        } else {
            int64_t entry_index = marker - 1;
            if (entry_index >= 0 && entry_index < map->len && map->entries[entry_index].key == key) {
                *found = 1;
                if (probe_count != NULL) {
                    *probe_count = probes;
                }
                return slot;
            }
        }
        slot = (slot + 1) & mask;
    }
}

static int64_t vibe_map_str_find_slot(
    vibe_map_str_i64 *map,
    const char *key,
    int64_t *found,
    int64_t *probe_count
) {
    const char *safe_key = key == NULL ? "" : key;
    if (map->hash_cap <= 0 || map->hash_slots == NULL) {
        *found = 0;
        if (probe_count != NULL) {
            *probe_count = 0;
        }
        return -1;
    }
    uint64_t hash = vibe_hash_cstr_key(safe_key);
    int64_t mask = map->hash_cap - 1;
    int64_t slot = (int64_t)(hash & (uint64_t)mask);
    int64_t first_tombstone = -1;
    int64_t probes = 0;
    while (1) {
        probes += 1;
        int64_t marker = map->hash_slots[slot];
        if (marker == 0) {
            *found = 0;
            if (probe_count != NULL) {
                *probe_count = probes;
            }
            return first_tombstone >= 0 ? first_tombstone : slot;
        }
        if (marker < 0) {
            if (first_tombstone < 0) {
                first_tombstone = slot;
            }
        } else {
            int64_t entry_index = marker - 1;
            if (entry_index >= 0
                && entry_index < map->len
                && strcmp(map->entries[entry_index].key, safe_key) == 0) {
                *found = 1;
                if (probe_count != NULL) {
                    *probe_count = probes;
                }
                return slot;
            }
        }
        slot = (slot + 1) & mask;
    }
}

static void vibe_map_i64_hash_rebuild(vibe_map_i64_i64 *map, int64_t new_cap) {
    int64_t cap = vibe_next_hash_capacity(new_cap);
    int64_t *slots = (int64_t *)calloc((size_t)cap, sizeof(int64_t));
    if (slots == NULL) {
        vibe_panic("failed to allocate i64 map hash slots");
    }
    free(map->hash_slots);
    map->hash_slots = slots;
    map->hash_cap = cap;
    map->hash_len = 0;
    map->resize_count += 1;
    for (int64_t i = 0; i < map->len; i++) {
        int64_t found = 0;
        int64_t probes = 0;
        int64_t slot = vibe_map_i64_find_slot(map, map->entries[i].key, &found, &probes);
        if (slot < 0) {
            vibe_panic("failed to rebuild i64 map hash slots");
        }
        map->hash_slots[slot] = i + 1;
        map->hash_len += 1;
        map->probe_steps += probes;
    }
}

static void vibe_map_str_hash_rebuild(vibe_map_str_i64 *map, int64_t new_cap) {
    int64_t cap = vibe_next_hash_capacity(new_cap);
    int64_t *slots = (int64_t *)calloc((size_t)cap, sizeof(int64_t));
    if (slots == NULL) {
        vibe_panic("failed to allocate string map hash slots");
    }
    free(map->hash_slots);
    map->hash_slots = slots;
    map->hash_cap = cap;
    map->hash_len = 0;
    map->resize_count += 1;
    for (int64_t i = 0; i < map->len; i++) {
        int64_t found = 0;
        int64_t probes = 0;
        int64_t slot = vibe_map_str_find_slot(map, map->entries[i].key, &found, &probes);
        if (slot < 0) {
            vibe_panic("failed to rebuild string map hash slots");
        }
        map->hash_slots[slot] = i + 1;
        map->hash_len += 1;
        map->probe_steps += probes;
    }
}

static void vibe_map_i64_hash_ensure(vibe_map_i64_i64 *map, int64_t min_cap) {
    if (map->hash_cap < min_cap || map->hash_slots == NULL) {
        vibe_map_i64_hash_rebuild(map, min_cap);
    }
}

static void vibe_map_str_hash_ensure(vibe_map_str_i64 *map, int64_t min_cap) {
    if (map->hash_cap < min_cap || map->hash_slots == NULL) {
        vibe_map_str_hash_rebuild(map, min_cap);
    }
}

static void vibe_map_i64_maybe_grow_hash(vibe_map_i64_i64 *map) {
    if (map->hash_cap <= 0) {
        vibe_map_i64_hash_ensure(map, 16);
        return;
    }
    if ((map->hash_len + 1) * 10 >= map->hash_cap * 7) {
        vibe_map_i64_hash_rebuild(map, map->hash_cap * 2);
    }
}

static void vibe_map_str_maybe_grow_hash(vibe_map_str_i64 *map) {
    if (map->hash_cap <= 0) {
        vibe_map_str_hash_ensure(map, 16);
        return;
    }
    if ((map->hash_len + 1) * 10 >= map->hash_cap * 7) {
        vibe_map_str_hash_rebuild(map, map->hash_cap * 2);
    }
}

static void vibe_map_i64_ensure_capacity(vibe_map_i64_i64 *map, int64_t min_cap) {
    if (map == NULL) {
        vibe_panic("map handle is null");
    }
    if (map->cap >= min_cap) {
        return;
    }
    int64_t next_cap = map->cap <= 0 ? 8 : map->cap;
    while (next_cap < min_cap) {
        next_cap *= 2;
    }
    vibe_map_i64_entry *next_entries =
        (vibe_map_i64_entry *)calloc((size_t)next_cap, sizeof(vibe_map_i64_entry));
    if (next_entries == NULL) {
        vibe_panic("failed to grow i64 map");
    }
    if (map->entries != NULL && map->len > 0) {
        memcpy(next_entries, map->entries, (size_t)map->len * sizeof(vibe_map_i64_entry));
    }
    free(map->entries);
    map->entries = next_entries;
    map->cap = next_cap;
}

void *vibe_map_new_i64_i64(void) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)calloc(1, sizeof(vibe_map_i64_i64));
    if (map == NULL) {
        vibe_panic("failed to allocate i64 map");
    }
    map->tag = VIBE_CONTAINER_MAP_I64_I64;
    map->len = 0;
    map->cap = 0;
    map->entries = NULL;
    return (void *)map;
}

int64_t vibe_map_set_i64_i64(void *handle, int64_t key, int64_t value) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map.set(i64, i64) called on non-map handle");
    }
    vibe_map_i64_hash_ensure(map, 16);
    vibe_map_i64_maybe_grow_hash(map);
    int64_t found = 0;
    int64_t probes = 0;
    int64_t slot = vibe_map_i64_find_slot(map, key, &found, &probes);
    if (slot < 0) {
        vibe_panic("failed to find i64 map slot");
    }
    map->probe_steps += probes;
    if (found) {
        int64_t entry_index = map->hash_slots[slot] - 1;
        map->entries[entry_index].value = value;
        return 0;
    }
    if (probes > 1) {
        map->collision_count += 1;
    }
    vibe_map_i64_ensure_capacity(map, map->len + 1);
    int64_t entry_index = map->len;
    map->entries[entry_index].key = key;
    map->entries[entry_index].value = value;
    map->hash_slots[slot] = entry_index + 1;
    map->len += 1;
    map->hash_len += 1;
    return 0;
}

int64_t vibe_map_get_i64_i64(void *handle, int64_t key) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map.get(i64) called on non-map handle");
    }
    if (map->hash_cap <= 0 || map->hash_slots == NULL || map->len == 0) {
        return 0;
    }
    int64_t found = 0;
    int64_t probes = 0;
    int64_t slot = vibe_map_i64_find_slot(map, key, &found, &probes);
    map->probe_steps += probes;
    if (found && slot >= 0) {
        int64_t entry_index = map->hash_slots[slot] - 1;
        return map->entries[entry_index].value;
    }
    return 0;
}

int64_t vibe_map_contains_i64_i64(void *handle, int64_t key) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map.contains(i64) called on non-map handle");
    }
    if (map->hash_cap <= 0 || map->hash_slots == NULL || map->len == 0) {
        return 0;
    }
    int64_t found = 0;
    int64_t probes = 0;
    vibe_map_i64_find_slot(map, key, &found, &probes);
    map->probe_steps += probes;
    return found ? 1 : 0;
}

int64_t vibe_map_remove_i64_i64(void *handle, int64_t key) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map.remove(i64) called on non-map handle");
    }
    if (map->hash_cap <= 0 || map->hash_slots == NULL || map->len == 0) {
        return 0;
    }
    int64_t found = 0;
    int64_t probes = 0;
    int64_t slot = vibe_map_i64_find_slot(map, key, &found, &probes);
    map->probe_steps += probes;
    if (!found || slot < 0) {
        return 0;
    }
    int64_t removed_entry = map->hash_slots[slot] - 1;
    for (int64_t i = removed_entry + 1; i < map->len; i++) {
        map->entries[i - 1] = map->entries[i];
    }
    map->len -= 1;
    if (map->hash_len > 0) {
        map->hash_len -= 1;
    }
    if (map->hash_cap > 16 && map->hash_len * 4 < map->hash_cap) {
        vibe_map_i64_hash_rebuild(map, map->hash_cap / 2);
    } else {
        vibe_map_i64_hash_rebuild(map, map->hash_cap);
    }
    return 1;
}

int64_t vibe_map_len_i64_i64(void *handle) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map.len called on non-map handle");
    }
    return map->len;
}

int64_t vibe_map_collision_count_i64_i64(void *handle) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map collision_count called on non-map handle");
    }
    return map->collision_count;
}

int64_t vibe_map_resize_count_i64_i64(void *handle) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map resize_count called on non-map handle");
    }
    return map->resize_count;
}

int64_t vibe_map_probe_steps_i64_i64(void *handle) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map probe_steps called on non-map handle");
    }
    return map->probe_steps;
}

static void vibe_map_str_ensure_capacity(vibe_map_str_i64 *map, int64_t min_cap) {
    if (map == NULL) {
        vibe_panic("string map handle is null");
    }
    if (map->cap >= min_cap) {
        return;
    }
    int64_t next_cap = map->cap <= 0 ? 8 : map->cap;
    while (next_cap < min_cap) {
        next_cap *= 2;
    }
    vibe_map_str_entry *next_entries =
        (vibe_map_str_entry *)calloc((size_t)next_cap, sizeof(vibe_map_str_entry));
    if (next_entries == NULL) {
        vibe_panic("failed to grow string map");
    }
    if (map->entries != NULL && map->len > 0) {
        memcpy(next_entries, map->entries, (size_t)map->len * sizeof(vibe_map_str_entry));
    }
    free(map->entries);
    map->entries = next_entries;
    map->cap = next_cap;
}

void *vibe_map_new_str_i64(void) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)calloc(1, sizeof(vibe_map_str_i64));
    if (map == NULL) {
        vibe_panic("failed to allocate string map");
    }
    map->tag = VIBE_CONTAINER_MAP_STR_I64;
    map->len = 0;
    map->cap = 0;
    map->entries = NULL;
    return (void *)map;
}

int64_t vibe_map_set_str_i64(void *handle, const char *key, int64_t value) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map.set(Str, i64) called on non-map handle");
    }
    const char *safe_key = key == NULL ? "" : key;
    vibe_map_str_hash_ensure(map, 16);
    vibe_map_str_maybe_grow_hash(map);
    int64_t found = 0;
    int64_t probes = 0;
    int64_t slot = vibe_map_str_find_slot(map, safe_key, &found, &probes);
    if (slot < 0) {
        vibe_panic("failed to find string map slot");
    }
    map->probe_steps += probes;
    if (found) {
        int64_t entry_index = map->hash_slots[slot] - 1;
        map->entries[entry_index].value = value;
        return 0;
    }
    if (probes > 1) {
        map->collision_count += 1;
    }
    vibe_map_str_ensure_capacity(map, map->len + 1);
    int64_t entry_index = map->len;
    map->entries[entry_index].key = vibe_strdup_or_panic(safe_key);
    map->entries[entry_index].value = value;
    map->hash_slots[slot] = entry_index + 1;
    map->len += 1;
    map->hash_len += 1;
    return 0;
}

int64_t vibe_map_get_str_i64(void *handle, const char *key) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map.get(Str) called on non-map handle");
    }
    if (map->hash_cap <= 0 || map->hash_slots == NULL || map->len == 0) {
        return 0;
    }
    int64_t found = 0;
    int64_t probes = 0;
    int64_t slot = vibe_map_str_find_slot(map, key, &found, &probes);
    map->probe_steps += probes;
    if (found && slot >= 0) {
        int64_t entry_index = map->hash_slots[slot] - 1;
        return map->entries[entry_index].value;
    }
    return 0;
}

int64_t vibe_map_contains_str_i64(void *handle, const char *key) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map.contains(Str) called on non-map handle");
    }
    if (map->hash_cap <= 0 || map->hash_slots == NULL || map->len == 0) {
        return 0;
    }
    int64_t found = 0;
    int64_t probes = 0;
    vibe_map_str_find_slot(map, key, &found, &probes);
    map->probe_steps += probes;
    return found ? 1 : 0;
}

int64_t vibe_map_remove_str_i64(void *handle, const char *key) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map.remove(Str) called on non-map handle");
    }
    if (map->hash_cap <= 0 || map->hash_slots == NULL || map->len == 0) {
        return 0;
    }
    int64_t found = 0;
    int64_t probes = 0;
    int64_t slot = vibe_map_str_find_slot(map, key, &found, &probes);
    map->probe_steps += probes;
    if (!found || slot < 0) {
        return 0;
    }
    int64_t removed_entry = map->hash_slots[slot] - 1;
    free(map->entries[removed_entry].key);
    for (int64_t i = removed_entry + 1; i < map->len; i++) {
        map->entries[i - 1] = map->entries[i];
    }
    map->len -= 1;
    if (map->hash_len > 0) {
        map->hash_len -= 1;
    }
    if (map->hash_cap > 16 && map->hash_len * 4 < map->hash_cap) {
        vibe_map_str_hash_rebuild(map, map->hash_cap / 2);
    } else {
        vibe_map_str_hash_rebuild(map, map->hash_cap);
    }
    return 1;
}

int64_t vibe_map_len_str_i64(void *handle) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map.len called on non-map handle");
    }
    return map->len;
}

int64_t vibe_map_collision_count_str_i64(void *handle) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map collision_count called on non-map handle");
    }
    return map->collision_count;
}

int64_t vibe_map_resize_count_str_i64(void *handle) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map resize_count called on non-map handle");
    }
    return map->resize_count;
}

int64_t vibe_map_probe_steps_str_i64(void *handle) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map probe_steps called on non-map handle");
    }
    return map->probe_steps;
}

/* ── Map<Str, Str> hash helpers ─────────────────────────────────────── */

static int64_t vibe_map_str_str_find_slot(
    vibe_map_str_str *map,
    const char *key,
    int64_t *found,
    int64_t *probe_count
) {
    const char *safe_key = key == NULL ? "" : key;
    if (map->hash_capacity <= 0 || map->hash_buckets == NULL) {
        *found = 0;
        if (probe_count != NULL) {
            *probe_count = 0;
        }
        return -1;
    }
    uint64_t hash = vibe_hash_cstr_key(safe_key);
    int64_t mask = map->hash_capacity - 1;
    int64_t slot = (int64_t)(hash & (uint64_t)mask);
    int64_t first_tombstone = -1;
    int64_t probes = 0;
    while (1) {
        probes += 1;
        int64_t marker = map->hash_buckets[slot];
        if (marker == 0) {
            *found = 0;
            if (probe_count != NULL) {
                *probe_count = probes;
            }
            return first_tombstone >= 0 ? first_tombstone : slot;
        }
        if (marker < 0) {
            if (first_tombstone < 0) {
                first_tombstone = slot;
            }
        } else {
            int64_t entry_index = marker - 1;
            if (entry_index >= 0
                && entry_index < map->count
                && strcmp(map->entries[entry_index].key, safe_key) == 0) {
                *found = 1;
                if (probe_count != NULL) {
                    *probe_count = probes;
                }
                return slot;
            }
        }
        slot = (slot + 1) & mask;
    }
}

static void vibe_map_str_str_hash_rebuild(vibe_map_str_str *map, int64_t new_cap) {
    int64_t cap = vibe_next_hash_capacity(new_cap);
    int64_t *slots = (int64_t *)calloc((size_t)cap, sizeof(int64_t));
    if (slots == NULL) {
        vibe_panic("failed to allocate str_str map hash slots");
    }
    free(map->hash_buckets);
    map->hash_buckets = slots;
    map->hash_capacity = cap;
    for (int64_t i = 0; i < map->count; i++) {
        int64_t found = 0;
        int64_t probes = 0;
        int64_t slot = vibe_map_str_str_find_slot(map, map->entries[i].key, &found, &probes);
        if (slot < 0) {
            vibe_panic("failed to rebuild str_str map hash slots");
        }
        map->hash_buckets[slot] = i + 1;
    }
}

static void vibe_map_str_str_hash_ensure(vibe_map_str_str *map, int64_t min_cap) {
    if (map->hash_capacity < min_cap || map->hash_buckets == NULL) {
        vibe_map_str_str_hash_rebuild(map, min_cap);
    }
}

static void vibe_map_str_str_maybe_grow_hash(vibe_map_str_str *map) {
    if (map->hash_capacity <= 0) {
        vibe_map_str_str_hash_ensure(map, 16);
        return;
    }
    int64_t used = 0;
    for (int64_t i = 0; i < map->hash_capacity; i++) {
        if (map->hash_buckets[i] != 0) {
            used += 1;
        }
    }
    if ((used + 1) * 10 >= map->hash_capacity * 7) {
        vibe_map_str_str_hash_rebuild(map, map->hash_capacity * 2);
    }
}

static void vibe_map_str_str_ensure_capacity(vibe_map_str_str *map, int64_t min_cap) {
    if (map == NULL) {
        vibe_panic("str_str map handle is null");
    }
    if (map->capacity >= min_cap) {
        return;
    }
    int64_t next_cap = map->capacity <= 0 ? 8 : map->capacity;
    while (next_cap < min_cap) {
        next_cap *= 2;
    }
    vibe_map_str_str_entry *next_entries =
        (vibe_map_str_str_entry *)calloc((size_t)next_cap, sizeof(vibe_map_str_str_entry));
    if (next_entries == NULL) {
        vibe_panic("failed to grow str_str map");
    }
    if (map->entries != NULL && map->count > 0) {
        memcpy(next_entries, map->entries, (size_t)map->count * sizeof(vibe_map_str_str_entry));
    }
    free(map->entries);
    map->entries = next_entries;
    map->capacity = next_cap;
}

/* ── Map<Str, Str> public API ───────────────────────────────────────── */

void *vibe_map_new_str_str(int64_t initial_cap) {
    vibe_map_str_str *map = (vibe_map_str_str *)calloc(1, sizeof(vibe_map_str_str));
    if (map == NULL) {
        vibe_panic("failed to allocate str_str map");
    }
    map->tag = VIBE_CONTAINER_MAP_STR_STR;
    map->count = 0;
    map->capacity = 0;
    map->entries = NULL;
    map->hash_buckets = NULL;
    map->hash_capacity = 0;
    if (initial_cap > 0) {
        vibe_map_str_str_ensure_capacity(map, initial_cap);
    }
    return (void *)map;
}

void vibe_map_set_str_str(void *handle, const char *key, const char *value) {
    vibe_map_str_str *map = (vibe_map_str_str *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_STR) {
        vibe_panic("map.set(Str, Str) called on non-map handle");
    }
    const char *safe_key = key == NULL ? "" : key;
    const char *safe_value = value == NULL ? "" : value;
    vibe_map_str_str_hash_ensure(map, 16);
    vibe_map_str_str_maybe_grow_hash(map);
    int64_t found = 0;
    int64_t probes = 0;
    int64_t slot = vibe_map_str_str_find_slot(map, safe_key, &found, &probes);
    if (slot < 0) {
        vibe_panic("failed to find str_str map slot");
    }
    if (found) {
        int64_t entry_index = map->hash_buckets[slot] - 1;
        free(map->entries[entry_index].value);
        map->entries[entry_index].value = vibe_strdup_or_panic(safe_value);
        return;
    }
    vibe_map_str_str_ensure_capacity(map, map->count + 1);
    int64_t entry_index = map->count;
    map->entries[entry_index].key = vibe_strdup_or_panic(safe_key);
    map->entries[entry_index].value = vibe_strdup_or_panic(safe_value);
    map->entries[entry_index].hash = (int64_t)vibe_hash_cstr_key(safe_key);
    map->hash_buckets[slot] = entry_index + 1;
    map->count += 1;
}

char *vibe_map_get_str_str(void *handle, const char *key) {
    vibe_map_str_str *map = (vibe_map_str_str *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_STR) {
        vibe_panic("map.get(Str, Str) called on non-map handle");
    }
    if (map->hash_capacity <= 0 || map->hash_buckets == NULL || map->count == 0) {
        return "";
    }
    int64_t found = 0;
    int64_t probes = 0;
    int64_t slot = vibe_map_str_str_find_slot(map, key, &found, &probes);
    if (found && slot >= 0) {
        int64_t entry_index = map->hash_buckets[slot] - 1;
        return map->entries[entry_index].value;
    }
    return "";
}

int64_t vibe_map_contains_str_str(void *handle, const char *key) {
    vibe_map_str_str *map = (vibe_map_str_str *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_STR) {
        vibe_panic("map.contains(Str, Str) called on non-map handle");
    }
    if (map->hash_capacity <= 0 || map->hash_buckets == NULL || map->count == 0) {
        return 0;
    }
    int64_t found = 0;
    int64_t probes = 0;
    vibe_map_str_str_find_slot(map, key, &found, &probes);
    return found ? 1 : 0;
}

void vibe_map_remove_str_str(void *handle, const char *key) {
    vibe_map_str_str *map = (vibe_map_str_str *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_STR) {
        vibe_panic("map.remove(Str, Str) called on non-map handle");
    }
    if (map->hash_capacity <= 0 || map->hash_buckets == NULL || map->count == 0) {
        return;
    }
    int64_t found = 0;
    int64_t probes = 0;
    int64_t slot = vibe_map_str_str_find_slot(map, key, &found, &probes);
    if (!found || slot < 0) {
        return;
    }
    int64_t removed_entry = map->hash_buckets[slot] - 1;
    free(map->entries[removed_entry].key);
    free(map->entries[removed_entry].value);
    for (int64_t i = removed_entry + 1; i < map->count; i++) {
        map->entries[i - 1] = map->entries[i];
    }
    map->count -= 1;
    if (map->hash_capacity > 16 && map->count * 4 < map->hash_capacity) {
        vibe_map_str_str_hash_rebuild(map, map->hash_capacity / 2);
    } else {
        vibe_map_str_str_hash_rebuild(map, map->hash_capacity);
    }
}

int64_t vibe_map_len_str_str(void *handle) {
    vibe_map_str_str *map = (vibe_map_str_str *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_STR) {
        vibe_panic("map.len called on non str_str map handle");
    }
    return map->count;
}

char *vibe_map_value_at_str_str(void *handle, int64_t index) {
    vibe_map_str_str *map = (vibe_map_str_str *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_STR) {
        vibe_panic("map.value_at(Str, Str) called on non-map handle");
    }
    if (index < 0 || index >= map->count) {
        vibe_panic("str_str map value index out of bounds");
    }
    return map->entries[index].value;
}

int64_t vibe_container_len(void *handle) {
    if (handle == NULL) {
        vibe_panic("container len called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_LIST_I64) {
        return vibe_list_len_i64(handle);
    }
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_len_i64_i64(handle);
    }
    if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        return vibe_map_len_str_i64(handle);
    }
    if (tag == VIBE_CONTAINER_MAP_STR_STR) {
        return vibe_map_len_str_str(handle);
    }
    vibe_panic("container len called on unsupported container");
    return 0;
}

int64_t vibe_container_kind(void *handle) {
    if (handle == NULL) {
        vibe_panic("container kind called on null handle");
    }
    return *((int64_t *)handle);
}

int64_t vibe_container_get_i64(void *handle, int64_t key_or_index) {
    if (handle == NULL) {
        vibe_panic("container get(i64) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_LIST_I64) {
        return vibe_list_get_i64(handle, key_or_index);
    }
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_get_i64_i64(handle, key_or_index);
    }
    vibe_panic("container get(i64) unsupported for this container type");
    return 0;
}

int64_t vibe_container_set_i64(void *handle, int64_t key_or_index, int64_t value) {
    if (handle == NULL) {
        vibe_panic("container set(i64, i64) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_LIST_I64) {
        return vibe_list_set_i64(handle, key_or_index, value);
    }
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_set_i64_i64(handle, key_or_index, value);
    }
    vibe_panic("container set(i64, i64) unsupported for this container type");
    return 0;
}

int64_t vibe_container_contains_i64(void *handle, int64_t key) {
    if (handle == NULL) {
        vibe_panic("container contains(i64) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_contains_i64_i64(handle, key);
    }
    vibe_panic("container contains(i64) is only valid for Map<Int, Int>");
    return 0;
}

int64_t vibe_container_remove_i64(void *handle, int64_t key) {
    if (handle == NULL) {
        vibe_panic("container remove(i64) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_remove_i64_i64(handle, key);
    }
    vibe_panic("container remove(i64) is only valid for Map<Int, Int>");
    return 0;
}

int64_t vibe_container_get_str_i64(void *handle, const char *key) {
    if (handle == NULL) {
        vibe_panic("container get(Str) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        return vibe_map_get_str_i64(handle, key);
    }
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        int64_t int_key = (int64_t)(intptr_t)key;
        return vibe_map_get_i64_i64(handle, int_key);
    }
    vibe_panic("container get(Str) is only valid for Map<Str, Int>");
    return 0;
}

int64_t vibe_container_set_str_i64(void *handle, const char *key, int64_t value) {
    if (handle == NULL) {
        vibe_panic("container set(Str, Int) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        return vibe_map_set_str_i64(handle, key, value);
    }
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        int64_t int_key = (int64_t)(intptr_t)key;
        return vibe_map_set_i64_i64(handle, int_key, value);
    }
    vibe_panic("container set(Str, Int) is only valid for Map<Str, Int>");
    return 0;
}

int64_t vibe_container_contains_str_i64(void *handle, const char *key) {
    if (handle == NULL) {
        vibe_panic("container contains(Str) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        return vibe_map_contains_str_i64(handle, key);
    }
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        int64_t int_key = (int64_t)(intptr_t)key;
        return vibe_map_contains_i64_i64(handle, int_key);
    }
    vibe_panic("container contains(Str) is only valid for Map<Str, Int>");
    return 0;
}

int64_t vibe_container_remove_str_i64(void *handle, const char *key) {
    if (handle == NULL) {
        vibe_panic("container remove(Str) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        return vibe_map_remove_str_i64(handle, key);
    }
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        int64_t int_key = (int64_t)(intptr_t)key;
        return vibe_map_remove_i64_i64(handle, int_key);
    }
    vibe_panic("container remove(Str) is only valid for Map<Str, Int>");
    return 0;
}

int64_t vibe_container_get_auto_i64(void *handle, int64_t key_or_index) {
    if (handle == NULL) {
        vibe_panic("container get(auto) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_LIST_I64) {
        return vibe_list_get_i64(handle, key_or_index);
    }
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_get_i64_i64(handle, key_or_index);
    }
    if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        const char *key = (const char *)(uintptr_t)key_or_index;
        return vibe_map_get_str_i64(handle, key);
    }
    vibe_panic("container get(auto) unsupported container type");
    return 0;
}

int64_t vibe_container_set_auto_i64(void *handle, int64_t key_or_index, int64_t value) {
    if (handle == NULL) {
        vibe_panic("container set(auto) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_LIST_I64) {
        return vibe_list_set_i64(handle, key_or_index, value);
    }
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_set_i64_i64(handle, key_or_index, value);
    }
    if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        const char *key = (const char *)(uintptr_t)key_or_index;
        return vibe_map_set_str_i64(handle, key, value);
    }
    vibe_panic("container set(auto) unsupported container type");
    return 0;
}

int64_t vibe_container_contains_auto_i64(void *handle, int64_t key_raw) {
    if (handle == NULL) {
        vibe_panic("container contains(auto) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_contains_i64_i64(handle, key_raw);
    }
    if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        const char *key = (const char *)(uintptr_t)key_raw;
        return vibe_map_contains_str_i64(handle, key);
    }
    vibe_panic("container contains(auto) is only valid for map containers");
    return 0;
}

int64_t vibe_container_remove_auto_i64(void *handle, int64_t key_raw) {
    if (handle == NULL) {
        vibe_panic("container remove(auto) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_remove_i64_i64(handle, key_raw);
    }
    if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        const char *key = (const char *)(uintptr_t)key_raw;
        return vibe_map_remove_str_i64(handle, key);
    }
    vibe_panic("container remove(auto) is only valid for map containers");
    return 0;
}

int64_t vibe_map_key_at_i64(void *handle, int64_t index) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map.key_at(i64) called on non-map handle");
    }
    if (index < 0 || index >= map->len) {
        vibe_panic("map key index out of bounds");
    }
    return map->entries[index].key;
}

const char *vibe_map_key_at_str(void *handle, int64_t index) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map.key_at(Str) called on non-map handle");
    }
    if (index < 0 || index >= map->len) {
        vibe_panic("map key index out of bounds");
    }
    return map->entries[index].key;
}

int64_t vibe_container_key_at_i64(void *handle, int64_t index) {
    if (handle == NULL) {
        vibe_panic("container key_at(i64) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_key_at_i64(handle, index);
    }
    vibe_panic("container key_at(i64) is only valid for Map<Int, Int>");
    return 0;
}

const char *vibe_container_key_at_str(void *handle, int64_t index) {
    if (handle == NULL) {
        vibe_panic("container key_at(Str) called on null handle");
    }
    int64_t tag = *((int64_t *)handle);
    if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        return vibe_map_key_at_str(handle, index);
    }
    if (tag == VIBE_CONTAINER_MAP_STR_STR) {
        vibe_map_str_str *map = (vibe_map_str_str *)handle;
        if (index < 0 || index >= map->count) {
            vibe_panic("str_str map key index out of bounds");
        }
        return map->entries[index].key;
    }
    vibe_panic("container key_at(Str) is only valid for string-keyed maps");
    return NULL;
}

int64_t vibe_list_eq_i64(void *left_handle, void *right_handle) {
    vibe_list_i64 *left = (vibe_list_i64 *)left_handle;
    vibe_list_i64 *right = (vibe_list_i64 *)right_handle;
    if (left == NULL || right == NULL) {
        return left == right ? 1 : 0;
    }
    if (left->tag != VIBE_CONTAINER_LIST_I64 || right->tag != VIBE_CONTAINER_LIST_I64) {
        return 0;
    }
    if (left->len != right->len) {
        return 0;
    }
    for (int64_t i = 0; i < left->len; i++) {
        if (left->items[i] != right->items[i]) {
            return 0;
        }
    }
    return 1;
}

int64_t vibe_map_eq_i64_i64(void *left_handle, void *right_handle) {
    vibe_map_i64_i64 *left = (vibe_map_i64_i64 *)left_handle;
    vibe_map_i64_i64 *right = (vibe_map_i64_i64 *)right_handle;
    if (left == NULL || right == NULL) {
        return left == right ? 1 : 0;
    }
    if (left->tag != VIBE_CONTAINER_MAP_I64_I64 || right->tag != VIBE_CONTAINER_MAP_I64_I64) {
        return 0;
    }
    if (left->len != right->len) {
        return 0;
    }
    for (int64_t i = 0; i < left->len; i++) {
        int64_t key = left->entries[i].key;
        if (!vibe_map_contains_i64_i64(right, key)) {
            return 0;
        }
        if (vibe_map_get_i64_i64(right, key) != left->entries[i].value) {
            return 0;
        }
    }
    return 1;
}

int64_t vibe_map_eq_str_i64(void *left_handle, void *right_handle) {
    vibe_map_str_i64 *left = (vibe_map_str_i64 *)left_handle;
    vibe_map_str_i64 *right = (vibe_map_str_i64 *)right_handle;
    if (left == NULL || right == NULL) {
        return left == right ? 1 : 0;
    }
    if (left->tag != VIBE_CONTAINER_MAP_STR_I64 || right->tag != VIBE_CONTAINER_MAP_STR_I64) {
        return 0;
    }
    if (left->len != right->len) {
        return 0;
    }
    for (int64_t i = 0; i < left->len; i++) {
        const char *key = left->entries[i].key;
        if (!vibe_map_contains_str_i64(right, key)) {
            return 0;
        }
        if (vibe_map_get_str_i64(right, key) != left->entries[i].value) {
            return 0;
        }
    }
    return 1;
}

int64_t vibe_container_eq(void *left_handle, void *right_handle) {
    if (left_handle == NULL || right_handle == NULL) {
        return left_handle == right_handle ? 1 : 0;
    }
    int64_t left_tag = *((int64_t *)left_handle);
    int64_t right_tag = *((int64_t *)right_handle);
    if (left_tag != right_tag) {
        return 0;
    }
    if (left_tag == VIBE_CONTAINER_LIST_I64) {
        return vibe_list_eq_i64(left_handle, right_handle);
    }
    if (left_tag == VIBE_CONTAINER_MAP_I64_I64) {
        return vibe_map_eq_i64_i64(left_handle, right_handle);
    }
    if (left_tag == VIBE_CONTAINER_MAP_STR_I64) {
        return vibe_map_eq_str_i64(left_handle, right_handle);
    }
    return 0;
}

static uint64_t vibe_hash_bytes(const unsigned char *bytes, size_t len) {
    uint64_t hash = 1469598103934665603ull;
    for (size_t i = 0; i < len; i++) {
        hash ^= (uint64_t)bytes[i];
        hash *= 1099511628211ull;
    }
    return hash;
}

int64_t vibe_str_hash(const char *value) {
    const char *safe_value = value == NULL ? "" : value;
    size_t len = strlen(safe_value);
    uint64_t hash = vibe_hash_bytes((const unsigned char *)safe_value, len);
    return (int64_t)hash;
}

int64_t vibe_container_hash(void *handle) {
    if (handle == NULL) {
        return 0;
    }
    int64_t tag = *((int64_t *)handle);
    uint64_t hash = 1469598103934665603ull;
    hash ^= (uint64_t)tag;
    hash *= 1099511628211ull;
    if (tag == VIBE_CONTAINER_LIST_I64) {
        vibe_list_i64 *list = (vibe_list_i64 *)handle;
        for (int64_t i = 0; i < list->len; i++) {
            hash ^= (uint64_t)list->items[i];
            hash *= 1099511628211ull;
        }
    } else if (tag == VIBE_CONTAINER_MAP_I64_I64) {
        vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
        for (int64_t i = 0; i < map->len; i++) {
            hash ^= (uint64_t)map->entries[i].key;
            hash *= 1099511628211ull;
            hash ^= (uint64_t)map->entries[i].value;
            hash *= 1099511628211ull;
        }
    } else if (tag == VIBE_CONTAINER_MAP_STR_I64) {
        vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
        for (int64_t i = 0; i < map->len; i++) {
            uint64_t key_hash = vibe_hash_bytes(
                (const unsigned char *)map->entries[i].key,
                strlen(map->entries[i].key));
            hash ^= key_hash;
            hash *= 1099511628211ull;
            hash ^= (uint64_t)map->entries[i].value;
            hash *= 1099511628211ull;
        }
    }
    return (int64_t)hash;
}

int64_t vibe_str_eq(const char *left, const char *right) {
    const char *l = left == NULL ? "" : left;
    const char *r = right == NULL ? "" : right;
    return strcmp(l, r) == 0 ? 1 : 0;
}

int64_t vibe_str_len_bytes(const char *value) {
    const char *safe_value = value == NULL ? "" : value;
    return (int64_t)strlen(safe_value);
}

int64_t vibe_str_get_byte(const char *value, int64_t index) {
    const char *safe_value = value == NULL ? "" : value;
    int64_t len = (int64_t)strlen(safe_value);
    if (index < 0 || index >= len) {
        vibe_panic("string index out of bounds");
    }
    if (!vibe_utf8_is_boundary(safe_value, index, len)) {
        vibe_panic("string index is not a UTF-8 boundary");
    }
    return (int64_t)((unsigned char)safe_value[index]);
}

void *vibe_str_slice(const char *value, int64_t start, int64_t end) {
    const char *safe_value = value == NULL ? "" : value;
    int64_t len = (int64_t)strlen(safe_value);
    if (start < 0 || end < 0 || start > end || end > len) {
        vibe_panic("invalid string slice range");
    }
    if (!vibe_utf8_is_boundary(safe_value, start, len)
        || !vibe_utf8_is_boundary(safe_value, end, len)) {
        vibe_panic("string slice boundary is not UTF-8 aligned");
    }
    int64_t out_len = end - start;
    char *out = (char *)calloc((size_t)out_len + 1u, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate sliced string");
    }
    if (out_len > 0) {
        memcpy(out, safe_value + start, (size_t)out_len);
    }
    out[out_len] = '\0';
    return (void *)out;
}

/*
 * TOTAL string primitives for bytes that arrived from OUTSIDE the process.
 *
 * A VibeLang Str is a bare NUL-terminated char* with no header, so there is
 * nowhere to record "these bytes came from a stranger". Trust is therefore a
 * property of the CODE PATH: only stdlib modules that handle socket bytes call
 * these, and `vibe_str_slice` / `vibe_str_get_byte` keep panicking for trusted
 * program data (a .yb literal cannot contain invalid UTF-8 -- the lexer reads
 * source as a Rust String and has no \xNN escape -- so any invalid-UTF-8 Str in
 * a running program came from an external byte source).
 *
 * The sources that mint such Strs never validate them: vibe_net_read,
 * vibe_ws_read_frame, vibe_http_server_parse_request, the HTTP client response
 * path, and the encoding.*_decode family all hand back raw octets. The UTF-8
 * invariant is already false by the time the router sees them, so enforcing it
 * there only punishes the server for the peer's choice -- with abort().
 */

/*
 * Byte-exact slice. TOTAL: never calls vibe_panic for any (value,start,end);
 * the only failure path is OOM, which is not peer-controlled.
 *
 * Bounds are CLAMPED, never rejected: start<0 -> 0; start>len -> len;
 * end<start -> start; end>len -> len. No UTF-8 boundary check, and NO byte is
 * ever rewritten or inserted -- the result is a verbatim memcpy of a subrange
 * of `value`. Preserving length and offsets exactly is the point: a
 * sanitising slice (e.g. substituting U+FFFD, which is 3 bytes) shifts every
 * downstream offset and makes later slices land mid-character, which is worse
 * than the bug it tries to fix.
 */
void *vibe_str_slice_bytes(const char *value, int64_t start, int64_t end) {
    const char *safe_value = value == NULL ? "" : value;
    int64_t len = (int64_t)strlen(safe_value);
    int64_t lo = start < 0 ? 0 : (start > len ? len : start);
    int64_t hi = end > len ? len : end;
    if (hi < lo) {
        hi = lo;
    }
    int64_t out_len = hi - lo;
    char *out = (char *)calloc((size_t)out_len + 1u, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate sliced string");
    }
    if (out_len > 0) {
        memcpy(out, safe_value + lo, (size_t)out_len);
    }
    out[out_len] = '\0';
    return (void *)out;
}

/*
 * Raw byte at `index`, or -1 when out of range. TOTAL: no bounds panic and no
 * UTF-8 boundary check. Sibling of vibe_str_slice_bytes for network-facing
 * code that needs to inspect a single octet; vibe_str_get_byte keeps panicking
 * for trusted data.
 */
int64_t vibe_str_byte_at(const char *value, int64_t index) {
    const char *safe_value = value == NULL ? "" : value;
    int64_t len = (int64_t)strlen(safe_value);
    if (index < 0 || index >= len) {
        return -1;
    }
    return (int64_t)((unsigned char)safe_value[index]);
}

void *vibe_str_concat(const char *left, const char *right) {
    const char *safe_left = left == NULL ? "" : left;
    const char *safe_right = right == NULL ? "" : right;
    size_t left_len = strlen(safe_left);
    size_t right_len = strlen(safe_right);
    size_t out_len = left_len + right_len;
    char *out = (char *)calloc(out_len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate concatenated string");
    }
    memcpy(out, safe_left, left_len);
    memcpy(out + left_len, safe_right, right_len);
    out[out_len] = '\0';
    return (void *)out;
}

/*
 * Binary data with an explicit length. This is the whole point of the type:
 * `Str` is `char*` and its length is `strlen` (vibe_str_len_bytes), so a Str
 * carrying binary data ends at its first 0x00. Measured before this type
 * existed: a 16-byte PNG header arrived on a socket and the program saw 8
 * bytes. `vibe_bytes` never consults a terminator.
 */
typedef struct vibe_bytes {
    int64_t tag;
    int64_t len;
    int64_t cap;
    uint8_t *data;
} vibe_bytes;

void *vibe_bytes_new(int64_t len) {
    if (len < 0) {
        len = 0;
    }
    vibe_bytes *b = (vibe_bytes *)calloc(1, sizeof(vibe_bytes));
    if (b == NULL) {
        vibe_panic("failed to allocate Bytes");
    }
    b->tag = VIBE_CONTAINER_BYTES;
    b->len = len;
    b->cap = len;
    b->data = (uint8_t *)calloc((size_t)len + 1, sizeof(uint8_t));
    if (b->data == NULL) {
        vibe_panic("failed to allocate Bytes buffer");
    }
    return b;
}

int64_t vibe_bytes_len(void *handle) {
    const vibe_bytes *b = (const vibe_bytes *)handle;
    return b == NULL ? 0 : b->len;
}

int64_t vibe_bytes_get(void *handle, int64_t index) {
    const vibe_bytes *b = (const vibe_bytes *)handle;
    if (b == NULL || index < 0 || index >= b->len) {
        return -1;
    }
    return (int64_t)b->data[index];
}

/*
 * Byte-exact slice. Clamped exactly the way vibe_str_slice_bytes is (see the
 * long comment above that function): TOTAL, never calls vibe_panic for any
 * (handle, start, end) -- the only failure path is OOM, which is not
 * peer-controlled. Bounds are CLAMPED, never rejected: start<0 -> 0;
 * start>len -> len; end<start -> start; end>len -> len. A NULL handle is
 * treated as a zero-length Bytes, so the result is an empty Bytes rather than
 * a panic.
 */
void *vibe_bytes_slice(void *handle, int64_t start, int64_t end) {
    const vibe_bytes *b = (const vibe_bytes *)handle;
    int64_t len = b == NULL ? 0 : b->len;
    int64_t lo = start < 0 ? 0 : (start > len ? len : start);
    int64_t hi = end > len ? len : end;
    if (hi < lo) {
        hi = lo;
    }
    int64_t out_len = hi - lo;
    vibe_bytes *out = (vibe_bytes *)vibe_bytes_new(out_len);
    if (out_len > 0) {
        memcpy(out->data, b->data + lo, (size_t)out_len);
    }
    return out;
}

/*
 * Concatenation. NULL operands are treated as zero-length Bytes. TOTAL
 * except for OOM.
 */
void *vibe_bytes_concat(void *a_handle, void *b_handle) {
    const vibe_bytes *a = (const vibe_bytes *)a_handle;
    const vibe_bytes *b = (const vibe_bytes *)b_handle;
    int64_t a_len = a == NULL ? 0 : a->len;
    int64_t b_len = b == NULL ? 0 : b->len;
    vibe_bytes *out = (vibe_bytes *)vibe_bytes_new(a_len + b_len);
    if (a_len > 0) {
        memcpy(out->data, a->data, (size_t)a_len);
    }
    if (b_len > 0) {
        memcpy(out->data + a_len, b->data, (size_t)b_len);
    }
    return out;
}

/*
 * Copies up to the NUL. This is the on-ramp from trusted program data (a .yb
 * string literal) into Bytes; the result never carries bytes past the
 * source's terminator, because a Str's length IS its terminator. A NULL
 * pointer is treated as an empty string.
 */
void *vibe_bytes_from_str(const char *s) {
    const char *safe_s = s == NULL ? "" : s;
    int64_t len = (int64_t)strlen(safe_s);
    vibe_bytes *out = (vibe_bytes *)vibe_bytes_new(len);
    if (len > 0) {
        memcpy(out->data, safe_s, (size_t)len);
    }
    return out;
}

/*
 * NUL-terminated copy of the Bytes buffer. DELIBERATELY LOSSY: any embedded
 * 0x00 byte in the source truncates the result exactly the way it would for
 * any NUL-scanning C string consumer (strlen, printf %s, ...). This is a
 * known, intentional boundary of the conversion, not a bug to fix here --
 * binary data with an embedded NUL has no faithful Str representation,
 * because a Str's length IS the offset of its first NUL. Callers that need
 * every byte back out losslessly should use vibe_bytes_to_hex instead. A
 * NULL handle returns an empty string.
 */
char *vibe_bytes_to_str(void *handle) {
    const vibe_bytes *b = (const vibe_bytes *)handle;
    int64_t len = b == NULL ? 0 : b->len;
    char *out = (char *)calloc((size_t)len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate Bytes-to-Str copy");
    }
    if (len > 0) {
        memcpy(out, b->data, (size_t)len);
    }
    out[len] = '\0';
    return out;
}

/* Forward declaration: defined below, shared with vibe_encoding_hex_decode
 * and vibe_encoding_hex_encode. Declared here so the Bytes<->hex functions,
 * which live beside the rest of the Bytes helpers, can use it too. */
static int vibe_hex_value(char ch);

/*
 * Decodes a hex string into Bytes. This is ONE on-ramp for a NUL byte, or
 * any other arbitrary octet, into program-controlled data (base64 decoding
 * into Bytes, vibe_encoding_base64_decode_bytes below, is another): a .yb
 * string literal cannot express \x00 (the lexer reads source as a Rust
 * String with no \xNN escape), so hex (or base64) is how later tests mint
 * binary inputs that plain string literals cannot.
 *
 * TOTAL, matching vibe_encoding_hex_decode's existing precedent in this file
 * (see that function below): an odd-length input, or any character outside
 * [0-9a-fA-F], yields a zero-length Bytes rather than a partial decode or a
 * panic. A partial decode would leave it ambiguous how many bytes decoded
 * before the bad one; an all-or-nothing empty result is unambiguous and
 * matches the sibling hex decoder already in this file. Never aborts on
 * attacker-shaped hex.
 */
void *vibe_bytes_from_hex(const char *hex) {
    const char *src = hex == NULL ? "" : hex;
    size_t len = strlen(src);
    if ((len % 2) != 0) {
        return vibe_bytes_new(0);
    }
    vibe_bytes *out = (vibe_bytes *)vibe_bytes_new((int64_t)(len / 2));
    for (size_t i = 0; i < len; i += 2) {
        int hi = vibe_hex_value(src[i]);
        int lo = vibe_hex_value(src[i + 1]);
        if (hi < 0 || lo < 0) {
            /* `out` hasn't escaped to any caller yet, so it is ours to
             * release before handing back the empty result. */
            free(out->data);
            free(out);
            return vibe_bytes_new(0);
        }
        out->data[i / 2] = (uint8_t)((hi << 4) | lo);
    }
    return out;
}

/*
 * Lowercase hex encoding of the whole Bytes buffer. Unlike vibe_bytes_to_str,
 * this is LOSSLESS: every byte, including an embedded NUL, round-trips
 * through vibe_bytes_from_hex. A NULL handle returns an empty string.
 */
char *vibe_bytes_to_hex(void *handle) {
    static const char hex_digits[] = "0123456789abcdef";
    const vibe_bytes *b = (const vibe_bytes *)handle;
    int64_t len = b == NULL ? 0 : b->len;
    char *out = (char *)calloc((size_t)len * 2 + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate Bytes-to-hex output");
    }
    for (int64_t i = 0; i < len; i++) {
        uint8_t byte_value = b->data[i];
        out[i * 2] = hex_digits[(byte_value >> 4) & 0x0f];
        out[i * 2 + 1] = hex_digits[byte_value & 0x0f];
    }
    out[len * 2] = '\0';
    return out;
}

/*
 * Result layout (from codegen): 2 x int64 slots — [0] tag (0 = Ok, 1 = Err),
 * [1] payload (Ok/Err value as int64; Str uses char* bitcast).
 */
void *vibe_result_wrap_err(void *result, const char *context) {
    if (result == NULL) {
        return NULL;
    }
    int64_t *slots = (int64_t *)result;
    if (slots[0] == 0) {
        return result;
    }
    const char *ctx = context == NULL ? "" : context;
    char *part1 = (char *)vibe_str_concat(ctx, ": ");
    const char *err_part = (const char *)(intptr_t)slots[1];
    void *combined = vibe_str_concat(part1, err_part == NULL ? "" : err_part);
    free(part1);
    void *out = vibe_record_alloc(2);
    int64_t *out_slots = (int64_t *)out;
    out_slots[0] = 1;
    out_slots[1] = (int64_t)(intptr_t)combined;
    return out;
}

int64_t vibe_result_unwrap_or(void *result, int64_t default_value) {
    if (result == NULL) {
        return default_value;
    }
    int64_t *slots = (int64_t *)result;
    if (slots[0] != 0) {
        return default_value;
    }
    return slots[1];
}

int8_t vibe_result_is_ok(void *result) {
    if (result == NULL) {
        return 0;
    }
    return (int8_t)(((int64_t *)result)[0] == 0 ? 1 : 0);
}

int8_t vibe_result_is_err(void *result) {
    if (result == NULL) {
        return 0;
    }
    return (int8_t)(((int64_t *)result)[0] != 0 ? 1 : 0);
}

#define VIBE_STR_BUILDER_MAGIC UINT64_C(0x53747242756C6472) /* "StrBuldr" */

typedef struct vibe_str_builder {
    uint64_t magic;
    char *buf;
    size_t len;
    size_t cap;
} vibe_str_builder;

static vibe_str_builder *vibe_str_builder_checked(void *handle) {
    vibe_handle_check(&vibe_str_builder_live, (uintptr_t)handle, "invalid str_builder handle");
    vibe_str_builder *sb = (vibe_str_builder *)handle;
    if (sb->magic != VIBE_STR_BUILDER_MAGIC) {
        vibe_panic("invalid str_builder handle");
    }
    return sb;
}

void *vibe_str_builder_new(int64_t initial_cap) {
    size_t cap = initial_cap > 0 ? (size_t)initial_cap : 64;
    vibe_str_builder *sb = (vibe_str_builder *)calloc(1, sizeof(vibe_str_builder));
    if (sb == NULL) vibe_panic("failed to allocate string builder");
    sb->buf = (char *)calloc(cap + 1, sizeof(char));
    if (sb->buf == NULL) vibe_panic("failed to allocate string builder buffer");
    sb->magic = VIBE_STR_BUILDER_MAGIC;
    sb->len = 0;
    sb->cap = cap;
    vibe_handle_register(&vibe_str_builder_live, (uintptr_t)sb);
    return (void *)sb;
}

void *vibe_str_builder_append(void *handle, const char *str) {
    vibe_str_builder *sb = vibe_str_builder_checked(handle);
    const char *safe_str = str == NULL ? "" : str;
    size_t slen = strlen(safe_str);
    if (sb->len + slen > sb->cap) {
        size_t new_cap = sb->cap * 2;
        while (new_cap < sb->len + slen) new_cap *= 2;
        char *new_buf = (char *)realloc(sb->buf, new_cap + 1);
        if (new_buf == NULL) vibe_panic("failed to grow string builder");
        sb->buf = new_buf;
        sb->cap = new_cap;
    }
    memcpy(sb->buf + sb->len, safe_str, slen);
    sb->len += slen;
    sb->buf[sb->len] = '\0';
    return handle;
}

void *vibe_str_builder_append_char(void *handle, int64_t ch) {
    vibe_str_builder *sb = vibe_str_builder_checked(handle);
    if (sb->len + 1 > sb->cap) {
        size_t new_cap = sb->cap * 2;
        char *new_buf = (char *)realloc(sb->buf, new_cap + 1);
        if (new_buf == NULL) vibe_panic("failed to grow string builder");
        sb->buf = new_buf;
        sb->cap = new_cap;
    }
    sb->buf[sb->len] = (char)ch;
    sb->len += 1;
    sb->buf[sb->len] = '\0';
    return handle;
}

void *vibe_str_builder_finish(void *handle) {
    vibe_str_builder *sb = vibe_str_builder_checked(handle);
    vibe_handle_unregister(&vibe_str_builder_live, (uintptr_t)sb);
    sb->magic = 0;
    char *result = sb->buf;
    free(sb);
    return (void *)result;
}

char *vibe_text_trim(const char *raw) {
    const char *text = raw == NULL ? "" : raw;
    const char *start = text;
    while (*start != '\0' && isspace((unsigned char)*start)) {
        start += 1;
    }
    const char *end = text + strlen(text);
    while (end > start && isspace((unsigned char)*(end - 1))) {
        end -= 1;
    }
    size_t len = (size_t)(end - start);
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate text.trim output");
    }
    if (len > 0) {
        memcpy(out, start, len);
    }
    out[len] = '\0';
    return out;
}

int64_t vibe_text_contains(const char *text, const char *needle) {
    const char *hay = text == NULL ? "" : text;
    const char *pat = needle == NULL ? "" : needle;
    if (pat[0] == '\0') {
        return 1;
    }
    return strstr(hay, pat) != NULL ? 1 : 0;
}

int64_t vibe_text_starts_with(const char *text, const char *prefix) {
    const char *hay = text == NULL ? "" : text;
    const char *pre = prefix == NULL ? "" : prefix;
    size_t pre_len = strlen(pre);
    size_t hay_len = strlen(hay);
    if (pre_len > hay_len) {
        return 0;
    }
    return strncmp(hay, pre, pre_len) == 0 ? 1 : 0;
}

int64_t vibe_text_ends_with(const char *text, const char *suffix) {
    const char *hay = text == NULL ? "" : text;
    const char *suf = suffix == NULL ? "" : suffix;
    size_t suf_len = strlen(suf);
    size_t hay_len = strlen(hay);
    if (suf_len > hay_len) {
        return 0;
    }
    return strncmp(hay + (hay_len - suf_len), suf, suf_len) == 0 ? 1 : 0;
}

char *vibe_text_replace(const char *text, const char *from, const char *to) {
    const char *src = text == NULL ? "" : text;
    const char *needle = from == NULL ? "" : from;
    const char *replacement = to == NULL ? "" : to;
    if (needle[0] == '\0') {
        return vibe_strdup_or_panic(src);
    }
    size_t src_len = strlen(src);
    size_t needle_len = strlen(needle);
    size_t replacement_len = strlen(replacement);
    vibe_string_builder builder;
    vibe_builder_init(&builder, src_len + 1);
    const char *cursor = src;
    while (*cursor != '\0') {
        const char *match = strstr(cursor, needle);
        if (match == NULL) {
            vibe_builder_append_bytes(&builder, cursor, strlen(cursor));
            break;
        }
        size_t prefix_len = (size_t)(match - cursor);
        if (prefix_len > 0) {
            vibe_builder_append_bytes(&builder, cursor, prefix_len);
        }
        if (replacement_len > 0) {
            vibe_builder_append_bytes(&builder, replacement, replacement_len);
        }
        cursor = match + needle_len;
    }
    return builder.data;
}

char *vibe_text_to_lower(const char *text) {
    const char *src = text == NULL ? "" : text;
    size_t len = strlen(src);
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate text.to_lower output");
    }
    for (size_t i = 0; i < len; i++) {
        out[i] = (char)tolower((unsigned char)src[i]);
    }
    out[len] = '\0';
    return out;
}

char *vibe_text_to_upper(const char *text) {
    const char *src = text == NULL ? "" : text;
    size_t len = strlen(src);
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate text.to_upper output");
    }
    for (size_t i = 0; i < len; i++) {
        out[i] = (char)toupper((unsigned char)src[i]);
    }
    out[len] = '\0';
    return out;
}

int64_t vibe_text_byte_len(const char *text) {
    return (int64_t)strlen(text == NULL ? "" : text);
}

char *vibe_text_split_part(const char *text, const char *sep, int64_t index) {
    const char *src = text == NULL ? "" : text;
    const char *delim = sep == NULL ? "" : sep;
    if (index < 0) {
        return vibe_strdup_or_panic("");
    }
    if (delim[0] == '\0') {
        return index == 0 ? vibe_strdup_or_panic(src) : vibe_strdup_or_panic("");
    }
    size_t delim_len = strlen(delim);
    int64_t current = 0;
    const char *cursor = src;
    while (1) {
        const char *match = strstr(cursor, delim);
        const char *part_end = match == NULL ? (cursor + strlen(cursor)) : match;
        if (current == index) {
            size_t len = (size_t)(part_end - cursor);
            char *out = (char *)calloc(len + 1, sizeof(char));
            if (out == NULL) {
                vibe_panic("failed to allocate text.split_part output");
            }
            if (len > 0) {
                memcpy(out, cursor, len);
            }
            out[len] = '\0';
            return out;
        }
        if (match == NULL) {
            break;
        }
        cursor = match + delim_len;
        current += 1;
    }
    return vibe_strdup_or_panic("");
}

int64_t vibe_text_index_of(const char *haystack, const char *needle) {
    const char *h = haystack == NULL ? "" : haystack;
    const char *n = needle == NULL ? "" : needle;
    if (n[0] == '\0') {
        return 0;
    }
    const char *found = strstr(h, n);
    if (found == NULL) {
        return -1;
    }
    return (int64_t)(found - h);
}

static int vibe_hex_value(char ch) {
    if (ch >= '0' && ch <= '9') {
        return (int)(ch - '0');
    }
    if (ch >= 'a' && ch <= 'f') {
        return (int)(ch - 'a' + 10);
    }
    if (ch >= 'A' && ch <= 'F') {
        return (int)(ch - 'A' + 10);
    }
    return -1;
}

char *vibe_encoding_hex_encode(const char *text) {
    static const char hex[] = "0123456789abcdef";
    const unsigned char *src = (const unsigned char *)(text == NULL ? "" : text);
    size_t len = strlen((const char *)src);
    char *out = (char *)calloc(len * 2 + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate hex_encode output");
    }
    for (size_t i = 0; i < len; i++) {
        out[i * 2] = hex[(src[i] >> 4) & 0x0f];
        out[i * 2 + 1] = hex[src[i] & 0x0f];
    }
    out[len * 2] = '\0';
    return out;
}

char *vibe_encoding_hex_decode(const char *hex_text) {
    const char *src = hex_text == NULL ? "" : hex_text;
    size_t len = strlen(src);
    if ((len % 2) != 0) {
        return vibe_strdup_or_panic("");
    }
    char *out = (char *)calloc(len / 2 + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate hex_decode output");
    }
    for (size_t i = 0; i < len; i += 2) {
        int hi = vibe_hex_value(src[i]);
        int lo = vibe_hex_value(src[i + 1]);
        if (hi < 0 || lo < 0) {
            free(out);
            return vibe_strdup_or_panic("");
        }
        out[i / 2] = (char)((hi << 4) | lo);
    }
    out[len / 2] = '\0';
    return out;
}

static const char VIBE_BASE64_ALPHABET[] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/*
 * Shared base64 encode core: encodes exactly `len` bytes at `src` (`src` may
 * be NULL only when `len` is 0) into a freshly allocated, NUL-terminated
 * Str. vibe_encoding_base64_encode (Str input) and
 * vibe_encoding_base64_encode_bytes (Bytes input) differ only in how they
 * resolve their argument to a (pointer, length) pair; both call this for the
 * actual triple-packing loop, so there is exactly one copy of it.
 */
static char *vibe_base64_encode_core(const uint8_t *src, size_t len) {
    size_t out_len = ((len + 2) / 3) * 4;
    char *out = (char *)calloc(out_len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate base64 encode output");
    }
    size_t in_idx = 0;
    size_t out_idx = 0;
    while (in_idx < len) {
        size_t rem = len - in_idx;
        uint32_t octet_a = src[in_idx++];
        uint32_t octet_b = rem > 1 ? src[in_idx++] : 0;
        uint32_t octet_c = rem > 2 ? src[in_idx++] : 0;
        uint32_t triple = (octet_a << 16) | (octet_b << 8) | octet_c;
        out[out_idx++] = VIBE_BASE64_ALPHABET[(triple >> 18) & 0x3f];
        out[out_idx++] = VIBE_BASE64_ALPHABET[(triple >> 12) & 0x3f];
        out[out_idx++] = rem > 1 ? VIBE_BASE64_ALPHABET[(triple >> 6) & 0x3f] : '=';
        out[out_idx++] = rem > 2 ? VIBE_BASE64_ALPHABET[triple & 0x3f] : '=';
    }
    out[out_len] = '\0';
    return out;
}

char *vibe_encoding_base64_encode(const char *text) {
    const char *safe = text == NULL ? "" : text;
    return vibe_base64_encode_core((const uint8_t *)safe, strlen(safe));
}

/*
 * Same encode core as vibe_encoding_base64_encode, but the input length
 * comes from the vibe_bytes struct's explicit `len` field rather than
 * strlen, so an embedded 0x00 byte is encoded like any other byte instead of
 * truncating the input. A NULL handle is treated as zero-length Bytes.
 */
char *vibe_encoding_base64_encode_bytes(void *handle) {
    const vibe_bytes *b = (const vibe_bytes *)handle;
    int64_t len = b == NULL ? 0 : b->len;
    const uint8_t *src = b == NULL ? NULL : b->data;
    return vibe_base64_encode_core(src, (size_t)len);
}

static int vibe_base64_value(char ch) {
    if (ch >= 'A' && ch <= 'Z') {
        return ch - 'A';
    }
    if (ch >= 'a' && ch <= 'z') {
        return ch - 'a' + 26;
    }
    if (ch >= '0' && ch <= '9') {
        return ch - '0' + 52;
    }
    if (ch == '+') {
        return 62;
    }
    if (ch == '/') {
        return 63;
    }
    return -1;
}

/*
 * Shared base64 decode core. PRECONDITION: len > 0 and len % 4 == 0 --
 * callers check this themselves before allocating `out`, so a
 * doomed-by-length request never allocates. Writes up to (len/4)*3 bytes
 * into `out` (which the caller must size to at least that) and returns the
 * number written, or -1 if any character in `src` is outside the base64
 * alphabet (padding characters aside). Never calls vibe_panic -- `src` may
 * be attacker-controlled text.
 *
 * KNOWN LENIENCY (inherited from the pre-existing vibe_encoding_base64_decode
 * this was extracted from; not introduced or fixed here, and explicitly out
 * of scope for the byte-taking-encoding-and-hashing task): this decoder
 * accepts non-canonical padding ("====" decodes as one zero byte, "AA==AAAA"
 * decodes as four bytes) and rejects any line-wrapped input (a "\n" inside
 * base64 fails the character check), so PEM/MIME-style bodies do not decode.
 * Both callers below share this behavior because they share this loop --
 * fixing it here fixes it for both at once, which is the point of the
 * extraction.
 */
static int64_t vibe_base64_decode_core(const char *src, size_t len, uint8_t *out) {
    size_t out_idx = 0;
    for (size_t i = 0; i < len; i += 4) {
        int vals[4];
        for (int j = 0; j < 4; j++) {
            char ch = src[i + (size_t)j];
            vals[j] = (ch == '=') ? -2 : vibe_base64_value(ch);
            if (vals[j] < -1) {
                // -2 is valid padding
                continue;
            }
            if (vals[j] < 0) {
                return -1;
            }
        }
        uint32_t triple = 0;
        int pad = 0;
        for (int j = 0; j < 4; j++) {
            if (vals[j] == -2) {
                vals[j] = 0;
                pad += 1;
            }
            triple = (triple << 6) | (uint32_t)(vals[j] & 0x3f);
        }
        out[out_idx++] = (uint8_t)((triple >> 16) & 0xff);
        if (pad < 2) {
            out[out_idx++] = (uint8_t)((triple >> 8) & 0xff);
        }
        if (pad < 1) {
            out[out_idx++] = (uint8_t)(triple & 0xff);
        }
    }
    return (int64_t)out_idx;
}

char *vibe_encoding_base64_decode(const char *base64_text) {
    const char *src = base64_text == NULL ? "" : base64_text;
    size_t len = strlen(src);
    if (len == 0 || (len % 4) != 0) {
        return vibe_strdup_or_panic("");
    }
    size_t out_cap = (len / 4) * 3;
    char *out = (char *)calloc(out_cap + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate base64_decode output");
    }
    int64_t written = vibe_base64_decode_core(src, len, (uint8_t *)out);
    if (written < 0) {
        free(out);
        return vibe_strdup_or_panic("");
    }
    out[written] = '\0';
    return out;
}

/*
 * Same decode core as vibe_encoding_base64_decode, but the result is Bytes
 * rather than a NUL-terminated Str, so a decoded 0x00 byte survives. The
 * input is base64 text, which is legitimately NUL-terminated, so reading its
 * length with strlen is correct here -- only the OUTPUT needed to change
 * shape. TOTAL: any malformed base64 (bad length, invalid character) yields
 * an empty Bytes, matching vibe_encoding_base64_decode's own precedent,
 * rather than a panic on attacker-controlled input.
 *
 * `out->cap` is sized to the length-implied upper bound `(len/4)*3` and may
 * end up larger than `out->len` once padding is accounted for; nothing reads
 * `cap` on a Bytes value after construction (grep confirms the only use is
 * inside vibe_bytes_new itself), so this is safe.
 */
void *vibe_encoding_base64_decode_bytes(const char *base64_text) {
    const char *src = base64_text == NULL ? "" : base64_text;
    size_t len = strlen(src);
    if (len == 0 || (len % 4) != 0) {
        return vibe_bytes_new(0);
    }
    size_t out_cap = (len / 4) * 3;
    vibe_bytes *out = (vibe_bytes *)vibe_bytes_new((int64_t)out_cap);
    int64_t written = vibe_base64_decode_core(src, len, out->data);
    if (written < 0) {
        free(out->data);
        free(out);
        return vibe_bytes_new(0);
    }
    out->len = written;
    return out;
}

char *vibe_encoding_url_encode(const char *text) {
    static const char hex[] = "0123456789ABCDEF";
    const unsigned char *src = (const unsigned char *)(text == NULL ? "" : text);
    size_t len = strlen((const char *)src);
    vibe_string_builder builder;
    vibe_builder_init(&builder, len * 3 + 1);
    for (size_t i = 0; i < len; i++) {
        unsigned char ch = src[i];
        int safe = (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z') ||
                   (ch >= '0' && ch <= '9') || ch == '-' || ch == '_' || ch == '.' || ch == '~';
        if (safe) {
            char c = (char)ch;
            vibe_builder_append_bytes(&builder, &c, 1);
        } else {
            char esc[3];
            esc[0] = '%';
            esc[1] = hex[(ch >> 4) & 0x0f];
            esc[2] = hex[ch & 0x0f];
            vibe_builder_append_bytes(&builder, esc, 3);
        }
    }
    return builder.data;
}

char *vibe_encoding_url_decode(const char *text) {
    const char *src = text == NULL ? "" : text;
    size_t len = strlen(src);
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate url_decode output");
    }
    size_t out_idx = 0;
    for (size_t i = 0; i < len; i++) {
        if (src[i] == '%' && i + 2 < len) {
            int hi = vibe_hex_value(src[i + 1]);
            int lo = vibe_hex_value(src[i + 2]);
            if (hi >= 0 && lo >= 0) {
                out[out_idx++] = (char)((hi << 4) | lo);
                i += 2;
                continue;
            }
        }
        if (src[i] == '+') {
            out[out_idx++] = ' ';
        } else {
            out[out_idx++] = src[i];
        }
    }
    out[out_idx] = '\0';
    return out;
}

#ifndef _WIN32
#define VIBE_REGEX_CACHE_SIZE 32

typedef struct vibe_regex_cache_entry {
    char *pattern;
    regex_t compiled;
    int in_use;
} vibe_regex_cache_entry;

static vibe_regex_cache_entry vibe_regex_cache[VIBE_REGEX_CACHE_SIZE];
static int vibe_regex_cache_next = 0;

static regex_t *vibe_regex_get_cached(const char *pattern) {
    for (int i = 0; i < VIBE_REGEX_CACHE_SIZE; i++) {
        if (vibe_regex_cache[i].in_use && strcmp(vibe_regex_cache[i].pattern, pattern) == 0) {
            return &vibe_regex_cache[i].compiled;
        }
    }
    int slot = vibe_regex_cache_next;
    vibe_regex_cache_next = (vibe_regex_cache_next + 1) % VIBE_REGEX_CACHE_SIZE;
    if (vibe_regex_cache[slot].in_use) {
        regfree(&vibe_regex_cache[slot].compiled);
        free(vibe_regex_cache[slot].pattern);
    }
    vibe_regex_cache[slot].pattern = vibe_strdup_or_panic(pattern);
    int rc = regcomp(&vibe_regex_cache[slot].compiled, pattern, REG_EXTENDED | REG_NEWLINE);
    if (rc != 0) {
        free(vibe_regex_cache[slot].pattern);
        vibe_regex_cache[slot].in_use = 0;
        return NULL;
    }
    vibe_regex_cache[slot].in_use = 1;
    return &vibe_regex_cache[slot].compiled;
}
#endif

int64_t vibe_regex_count(const char *text, const char *pattern) {
#ifdef _WIN32
    (void)text;
    (void)pattern;
    vibe_panic("regex.count is not supported on windows runtime");
    return 0;
#else
    const char *safe_text = text == NULL ? "" : text;
    const char *safe_pattern = pattern == NULL ? "" : pattern;
    if (safe_pattern[0] == '\0') {
        return 0;
    }
    regex_t *compiled = vibe_regex_get_cached(safe_pattern);
    if (compiled == NULL) {
        vibe_panic("regex.count failed to compile pattern");
    }

    int64_t count = 0;
    const char *cursor = safe_text;
    size_t remaining = strlen(safe_text);
    while (remaining > 0) {
        regmatch_t match;
        int exec_rc = regexec(compiled, cursor, 1, &match, 0);
        if (exec_rc == REG_NOMATCH) {
            break;
        }
        if (exec_rc != 0) {
            vibe_panic("regex.count execution failed");
        }
        if (match.rm_so < 0 || match.rm_eo < 0) {
            break;
        }
        size_t start = (size_t)match.rm_so;
        size_t end = (size_t)match.rm_eo;
        if (end < start || end > remaining) {
            vibe_panic("regex.count produced invalid match bounds");
        }
        count += 1;
        size_t advance = end > start ? end : start + 1;
        if (advance > remaining) {
            break;
        }
        cursor += advance;
        remaining -= advance;
    }
    return count;
#endif
}

char *vibe_regex_replace_all(const char *text, const char *pattern, const char *replacement) {
#ifdef _WIN32
    (void)text;
    (void)pattern;
    (void)replacement;
    vibe_panic("regex.replace_all is not supported on windows runtime");
    return vibe_strdup_or_panic("");
#else
    const char *safe_text = text == NULL ? "" : text;
    const char *safe_pattern = pattern == NULL ? "" : pattern;
    const char *safe_replacement = replacement == NULL ? "" : replacement;
    if (safe_pattern[0] == '\0') {
        return vibe_strdup_or_panic(safe_text);
    }

    regex_t *compiled = vibe_regex_get_cached(safe_pattern);
    if (compiled == NULL) {
        vibe_panic("regex.replace_all failed to compile pattern");
    }

    size_t text_len = strlen(safe_text);
    vibe_string_builder builder;
    vibe_builder_init(&builder, text_len + 1);

    const char *cursor = safe_text;
    size_t remaining = text_len;
    while (remaining > 0) {
        regmatch_t match;
        int exec_rc = regexec(compiled, cursor, 1, &match, 0);
        if (exec_rc == REG_NOMATCH) {
            break;
        }
        if (exec_rc != 0) {
            free(builder.data);
            vibe_panic("regex.replace_all execution failed");
        }
        if (match.rm_so < 0 || match.rm_eo < 0) {
            break;
        }
        size_t start = (size_t)match.rm_so;
        size_t end = (size_t)match.rm_eo;
        if (end < start || end > remaining) {
            free(builder.data);
            vibe_panic("regex.replace_all produced invalid match bounds");
        }
        if (start > 0) {
            vibe_builder_append_bytes(&builder, cursor, start);
        }
        vibe_builder_append_bytes(&builder, safe_replacement, strlen(safe_replacement));
        size_t advance = end > start ? end : start + 1;
        if (advance > remaining) {
            break;
        }
        cursor += advance;
        remaining -= advance;
    }
    if (remaining > 0) {
        vibe_builder_append_bytes(&builder, cursor, remaining);
    }
    return builder.data;
#endif
}

#define VIBE_CHAN_MAGIC UINT64_C(0x4368616E49363421) /* "ChanI64!" */

/* Channels are never freed (goroutines may still hold them), so live
 * channels stay registered for the whole program run. */
static vibe_chan_i64 *vibe_chan_checked(void *handle) {
    vibe_handle_check(&vibe_chan_live, (uintptr_t)handle, "invalid channel handle");
    vibe_chan_i64 *ch = (vibe_chan_i64 *)handle;
    if (ch->magic != VIBE_CHAN_MAGIC) {
        vibe_panic("invalid channel handle");
    }
    return ch;
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
    ch->magic = VIBE_CHAN_MAGIC;
    ch->capacity = capacity;
    ch->head = 0;
    ch->tail = 0;
    ch->size = 0;
    ch->closed = 0;
    pthread_mutex_init(&ch->mu, NULL);
    pthread_cond_init(&ch->can_send, NULL);
    pthread_cond_init(&ch->can_recv, NULL);
    vibe_handle_register(&vibe_chan_live, (uintptr_t)ch);
    return (void *)ch;
}

int64_t vibe_chan_send_i64(void *handle, int64_t value) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    int64_t lock_contended = 0;
    if (pthread_mutex_trylock(&ch->mu) == 0) {
        if (ch->closed) {
            pthread_mutex_unlock(&ch->mu);
            return 1;
        }
        if (ch->size < ch->capacity) {
            ch->buffer[ch->tail] = value;
            ch->tail += 1;
            if (ch->tail >= ch->capacity) {
                ch->tail = 0;
            }
            ch->size += 1;
            ch->send_fast_path_hits += 1;
            pthread_cond_signal(&ch->can_recv);
            pthread_mutex_unlock(&ch->mu);
            return 0;
        }
        ch->contention_count += 1;
        pthread_mutex_unlock(&ch->mu);
    } else {
        lock_contended = 1;
    }
    pthread_mutex_lock(&ch->mu);
    if (lock_contended) {
        ch->contention_count += 1;
    }
    while (!ch->closed && ch->size >= ch->capacity) {
        ch->send_wait_count += 1;
        pthread_cond_wait(&ch->can_send, &ch->mu);
    }
    if (ch->closed) {
        pthread_mutex_unlock(&ch->mu);
        return 1;
    }
    ch->buffer[ch->tail] = value;
    ch->tail += 1;
    if (ch->tail >= ch->capacity) {
        ch->tail = 0;
    }
    ch->size += 1;
    ch->send_slow_path_hits += 1;
    pthread_cond_signal(&ch->can_recv);
    pthread_mutex_unlock(&ch->mu);
    return 0;
}

int64_t vibe_chan_recv_i64(void *handle) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    int64_t lock_contended = 0;
    if (pthread_mutex_trylock(&ch->mu) == 0) {
        if (ch->size > 0) {
            int64_t value = ch->buffer[ch->head];
            ch->head += 1;
            if (ch->head >= ch->capacity) {
                ch->head = 0;
            }
            ch->size -= 1;
            ch->recv_fast_path_hits += 1;
            pthread_cond_signal(&ch->can_send);
            pthread_mutex_unlock(&ch->mu);
            return value;
        }
        if (ch->closed) {
            pthread_mutex_unlock(&ch->mu);
            return 0;
        }
        ch->contention_count += 1;
        pthread_mutex_unlock(&ch->mu);
    } else {
        lock_contended = 1;
    }
    pthread_mutex_lock(&ch->mu);
    if (lock_contended) {
        ch->contention_count += 1;
    }
    while (!ch->closed && ch->size == 0) {
        ch->recv_wait_count += 1;
        pthread_cond_wait(&ch->can_recv, &ch->mu);
    }
    if (ch->size == 0) {
        pthread_mutex_unlock(&ch->mu);
        return 0;
    }
    int64_t value = ch->buffer[ch->head];
    ch->head += 1;
    if (ch->head >= ch->capacity) {
        ch->head = 0;
    }
    ch->size -= 1;
    ch->recv_slow_path_hits += 1;
    pthread_cond_signal(&ch->can_send);
    pthread_mutex_unlock(&ch->mu);
    return value;
}

int64_t vibe_chan_try_recv_i64(void *handle, int64_t *out_value) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    if (ch->size > 0) {
        int64_t value = ch->buffer[ch->head];
        ch->head += 1;
        if (ch->head >= ch->capacity) {
            ch->head = 0;
        }
        ch->size -= 1;
        ch->recv_fast_path_hits += 1;
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
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    int64_t ready = ch->size > 0 ? 1 : 0;
    pthread_mutex_unlock(&ch->mu);
    return ready;
}

void vibe_chan_close_i64(void *handle) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    ch->closed = 1;
    pthread_cond_broadcast(&ch->can_send);
    pthread_cond_broadcast(&ch->can_recv);
    pthread_mutex_unlock(&ch->mu);
}

int64_t vibe_chan_is_closed_i64(void *handle) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    int64_t result = ch->closed ? 1 : 0;
    pthread_mutex_unlock(&ch->mu);
    return result;
}

int64_t vibe_chan_send_fast_hits_i64(void *handle) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    int64_t value = ch->send_fast_path_hits;
    pthread_mutex_unlock(&ch->mu);
    return value;
}

int64_t vibe_chan_recv_fast_hits_i64(void *handle) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    int64_t value = ch->recv_fast_path_hits;
    pthread_mutex_unlock(&ch->mu);
    return value;
}

int64_t vibe_chan_send_slow_hits_i64(void *handle) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    int64_t value = ch->send_slow_path_hits;
    pthread_mutex_unlock(&ch->mu);
    return value;
}

int64_t vibe_chan_recv_slow_hits_i64(void *handle) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    int64_t value = ch->recv_slow_path_hits;
    pthread_mutex_unlock(&ch->mu);
    return value;
}

int64_t vibe_chan_send_wait_count_i64(void *handle) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    int64_t value = ch->send_wait_count;
    pthread_mutex_unlock(&ch->mu);
    return value;
}

int64_t vibe_chan_recv_wait_count_i64(void *handle) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    int64_t value = ch->recv_wait_count;
    pthread_mutex_unlock(&ch->mu);
    return value;
}

int64_t vibe_chan_contention_count_i64(void *handle) {
    vibe_chan_i64 *ch = vibe_chan_checked(handle);
    pthread_mutex_lock(&ch->mu);
    int64_t value = ch->contention_count;
    pthread_mutex_unlock(&ch->mu);
    return value;
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

typedef void (*vibe_closure_fn0)(void *env);
typedef void (*vibe_closure_fn1)(void *env, int64_t arg0);

static void *vibe_spawn_closure0_entry(void *opaque) {
    void *closure = opaque;
    if (closure == NULL) {
        return NULL;
    }
    void *code = *(void **)closure;
    if (code == NULL) {
        return NULL;
    }
    ((vibe_closure_fn0)code)(closure);
    return NULL;
}

typedef struct vibe_spawn_closure1_ctx {
    void *closure;
    int64_t arg0;
} vibe_spawn_closure1_ctx;

static void *vibe_spawn_closure1_entry(void *opaque) {
    vibe_spawn_closure1_ctx *ctx = (vibe_spawn_closure1_ctx *)opaque;
    if (ctx == NULL) {
        return NULL;
    }
    void *closure = ctx->closure;
    void *code = closure != NULL ? *(void **)closure : NULL;
    if (code != NULL) {
        ((vibe_closure_fn1)code)(closure, ctx->arg0);
    }
    free(ctx);
    return NULL;
}

int64_t vibe_spawn_closure0(void *closure) {
    if (closure == NULL) {
        return 1;
    }
    pthread_t tid;
    int rc = pthread_create(&tid, NULL, vibe_spawn_closure0_entry, closure);
    if (rc != 0) {
        return 1;
    }
    pthread_detach(tid);
    return 0;
}

int64_t vibe_spawn_closure1(void *closure, int64_t arg0) {
    if (closure == NULL) {
        return 1;
    }
    vibe_spawn_closure1_ctx *ctx = (vibe_spawn_closure1_ctx *)calloc(1, sizeof(vibe_spawn_closure1_ctx));
    if (ctx == NULL) {
        return 1;
    }
    ctx->closure = closure;
    ctx->arg0 = arg0;
    pthread_t tid;
    int rc = pthread_create(&tid, NULL, vibe_spawn_closure1_entry, ctx);
    if (rc != 0) {
        free(ctx);
        return 1;
    }
    pthread_detach(tid);
    return 0;
}

int64_t vibe_async_i64(int64_t value) {
    return value;
}

void *vibe_async_ptr(void *value) {
    return value;
}

int64_t vibe_await_i64(int64_t value) {
    return value;
}

void *vibe_await_ptr(void *value) {
    return value;
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

int64_t vibe_time_now_ms(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) {
        return 0;
    }
    return (int64_t)ts.tv_sec * 1000 + (int64_t)(ts.tv_nsec / 1000000);
}

int64_t vibe_time_monotonic_now_ms(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return vibe_time_now_ms();
    }
    return (int64_t)ts.tv_sec * 1000 + (int64_t)(ts.tv_nsec / 1000000);
}

void vibe_time_sleep_ms(int64_t ms) {
    vibe_sleep_ms(ms);
}

int64_t vibe_time_duration_ms(int64_t seconds) {
    if (seconds <= 0) {
        return 0;
    }
    if (seconds > INT64_MAX / 1000) {
        return INT64_MAX;
    }
    return seconds * 1000;
}

static const char *vibe_last_path_sep(const char *path) {
    if (path == NULL) {
        return NULL;
    }
    const char *last = NULL;
    for (const char *cur = path; *cur != '\0'; ++cur) {
        if (*cur == '/' || *cur == '\\') {
            last = cur;
        }
    }
    return last;
}

char *vibe_path_join(const char *base, const char *leaf) {
    const char *lhs = base == NULL ? "" : base;
    const char *rhs = leaf == NULL ? "" : leaf;
    size_t lhs_len = strlen(lhs);
    size_t rhs_len = strlen(rhs);
    int needs_sep = lhs_len > 0 && rhs_len > 0 && lhs[lhs_len - 1] != '/' && rhs[0] != '/';
    size_t out_len = lhs_len + rhs_len + (needs_sep ? 1 : 0);
    char *out = (char *)calloc(out_len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate path.join output");
    }
    if (lhs_len > 0) {
        memcpy(out, lhs, lhs_len);
    }
    size_t pos = lhs_len;
    if (needs_sep) {
        out[pos++] = '/';
    }
    if (rhs_len > 0) {
        memcpy(out + pos, rhs, rhs_len);
    }
    out[out_len] = '\0';
    return out;
}

char *vibe_path_parent(const char *path) {
    if (path == NULL || path[0] == '\0') {
        return vibe_strdup_or_panic(".");
    }
    const char *last_sep = vibe_last_path_sep(path);
    if (last_sep == NULL) {
        return vibe_strdup_or_panic(".");
    }
    if (last_sep == path) {
        return vibe_strdup_or_panic("/");
    }
    size_t len = (size_t)(last_sep - path);
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate path.parent output");
    }
    memcpy(out, path, len);
    out[len] = '\0';
    return out;
}

char *vibe_path_basename(const char *path) {
    if (path == NULL || path[0] == '\0') {
        return vibe_strdup_or_panic("");
    }
    const char *last_sep = vibe_last_path_sep(path);
    if (last_sep == NULL) {
        return vibe_strdup_or_panic(path);
    }
    return vibe_strdup_or_panic(last_sep + 1);
}

int64_t vibe_path_is_absolute(const char *path) {
    if (path == NULL || path[0] == '\0') {
        return 0;
    }
    if (path[0] == '/' || path[0] == '\\') {
        return 1;
    }
    if (isalpha((unsigned char)path[0]) && path[1] == ':') {
        return 1;
    }
    return 0;
}

int64_t vibe_fs_exists(const char *path) {
    if (path == NULL || path[0] == '\0') {
        return 0;
    }
    return access(path, F_OK) == 0 ? 1 : 0;
}

/*
 * Returns a file's byte size, or -1 when it cannot be stat'd (missing,
 * unreadable, or any other stat() failure). This exists so a caller can
 * pre-check a file against fs.read_bytes's VIBE_FS_READ_BYTES_MAX ceiling
 * and tell apart the three cases fs.read_bytes' own return value cannot
 * distinguish on its own: a missing file, a genuinely empty file, and a
 * file that was refused for being over the ceiling all otherwise produce
 * the same zero-length Bytes. fs.exists is not a substitute for this check
 * either -- it only calls access(path, F_OK), so it reports `true` for an
 * oversize file exactly as readily as for a normal one.
 *
 * TODO(Result): -1-as-sentinel is a stopgap matching this stdlib's current
 * error-signaling convention elsewhere (e.g. bytes.get's -1 for
 * out-of-range), not the final shape. Once the stdlib has a Result type
 * (a later plan item, not yet built), this should become
 * fs.size(path: Str) -> Result<Int, Error> and this sentinel should go
 * away.
 */
int64_t vibe_fs_size(const char *path) {
    if (path == NULL || path[0] == '\0') {
        return -1;
    }
    struct stat st;
    if (stat(path, &st) != 0) {
        return -1;
    }
    return (int64_t)st.st_size;
}

char *vibe_fs_read_text(const char *path) {
    if (path == NULL || path[0] == '\0') {
        return vibe_strdup_or_panic("");
    }
    FILE *f = fopen(path, "rb");
    if (f == NULL) {
        return vibe_strdup_or_panic("");
    }
    if (fseek(f, 0, SEEK_END) != 0) {
        fclose(f);
        return vibe_strdup_or_panic("");
    }
    long file_len = ftell(f);
    if (file_len < 0) {
        fclose(f);
        return vibe_strdup_or_panic("");
    }
    rewind(f);
    char *buffer = (char *)calloc((size_t)file_len + 1, sizeof(char));
    if (buffer == NULL) {
        fclose(f);
        vibe_panic("failed to allocate file read buffer");
    }
    size_t read_len = fread(buffer, 1, (size_t)file_len, f);
    fclose(f);
    buffer[read_len] = '\0';
    return buffer;
}

/*
 * Byte-exact whole-file read: length comes from the file's actual size, not
 * strlen, so an embedded 0x00 anywhere in the file no longer truncates the
 * result the way vibe_fs_read_text's Str return does.
 *
 * A file's size is not attacker-controlled in the direct, per-request way a
 * socket's max_bytes argument is (see vibe_net_read_bytes above), but it is
 * still unbounded input driving an allocation -- an attacker with any way to
 * place or grow a file this call later reads (an upload saved to disk and
 * re-read, a path traversal onto a huge or special file) still gets to pick
 * the allocation size. So this gets its own stated ceiling,
 * VIBE_FS_READ_BYTES_MAX (64 MiB), chosen to comfortably cover real files
 * (images, documents, small datasets) while keeping this call's own
 * worst-case allocation, and the worst case of the downstream amplifiers
 * bytes.to_hex (2x) and bytes.concat (sum of two), bounded to double- and
 * low-hundreds-of-MiB territory rather than unbounded.
 *
 * A file STRICTLY LARGER than the ceiling (file_len > VIBE_FS_READ_BYTES_MAX,
 * so a file of exactly VIBE_FS_READ_BYTES_MAX bytes is still read in full)
 * is treated as a read failure, the same as a missing file or a seek/tell
 * failure a few lines up: it returns an empty Bytes rather than either (a)
 * calling vibe_panic, whose abort() would take down the whole process under
 * the documented `while true { conn := net.accept(l); go handle(conn) }`
 * server pattern if this were reachable from a request handler, or (b)
 * silently handing back only the first VIBE_FS_READ_BYTES_MAX bytes of the
 * file mislabeled as the complete contents, which would reintroduce a
 * silent-truncation bug of exactly the shape this function exists to
 * eliminate.
 *
 * That empty-Bytes-on-failure return is indistinguishable from "the file is
 * genuinely empty" or "the file does not exist" from inside this function
 * alone -- fs.size (vibe_fs_size, above) is what lets a caller tell the
 * three cases apart before ever calling this function.
 */
#define VIBE_FS_READ_BYTES_MAX (64 * 1024 * 1024)

void *vibe_fs_read_bytes(const char *path) {
    if (path == NULL || path[0] == '\0') {
        return vibe_bytes_new(0);
    }
    FILE *f = fopen(path, "rb");
    if (f == NULL) {
        return vibe_bytes_new(0);
    }
    if (fseek(f, 0, SEEK_END) != 0) {
        fclose(f);
        return vibe_bytes_new(0);
    }
    long file_len = ftell(f);
    if (file_len < 0) {
        fclose(f);
        return vibe_bytes_new(0);
    }
    if (file_len > VIBE_FS_READ_BYTES_MAX) {
        fclose(f);
        return vibe_bytes_new(0);
    }
    rewind(f);
    vibe_bytes *out = (vibe_bytes *)vibe_bytes_new((int64_t)file_len);
    size_t read_len = fread(out->data, 1, (size_t)file_len, f);
    fclose(f);
    out->len = (int64_t)read_len;
    return out;
}

int64_t vibe_fs_write_text(const char *path, const char *content) {
    if (path == NULL || path[0] == '\0') {
        return 0;
    }
    FILE *f = fopen(path, "wb");
    if (f == NULL) {
        return 0;
    }
    const char *raw = content == NULL ? "" : content;
    size_t len = strlen(raw);
    size_t written = 0;
    if (len > 0) {
        written = fwrite(raw, 1, len, f);
    }
    int close_rc = fclose(f);
    if (len > 0 && written != len) {
        return 0;
    }
    return close_rc == 0 ? 1 : 0;
}

/*
 * Byte-exact file write: writes exactly the Bytes value's `len` bytes,
 * unlike vibe_fs_write_text which stops at `content`'s NUL terminator. A
 * NULL handle or a zero-length Bytes value writes a zero-length file,
 * matching vibe_fs_write_text's empty-string behavior.
 */
int64_t vibe_fs_write_bytes(const char *path, void *handle) {
    if (path == NULL || path[0] == '\0') {
        return 0;
    }
    FILE *f = fopen(path, "wb");
    if (f == NULL) {
        return 0;
    }
    const vibe_bytes *b = (const vibe_bytes *)handle;
    int64_t len = b == NULL ? 0 : b->len;
    size_t written = 0;
    if (len > 0) {
        written = fwrite(b->data, 1, (size_t)len, f);
    }
    int close_rc = fclose(f);
    if (len > 0 && written != (size_t)len) {
        return 0;
    }
    return close_rc == 0 ? 1 : 0;
}

int64_t vibe_fs_create_dir(const char *path) {
    if (path == NULL || path[0] == '\0') {
        return 0;
    }
#if defined(_WIN32)
    int rc = _mkdir(path);
#else
    int rc = mkdir(path, 0777);
#endif
    if (rc == 0) {
        return 1;
    }
    return errno == EEXIST ? 1 : 0;
}

void vibe_log_info(const char *msg) {
    fprintf(stdout, "[info] %s\n", msg == NULL ? "" : msg);
    fflush(stdout);
}

void vibe_log_warn(const char *msg) {
    fprintf(stdout, "[warn] %s\n", msg == NULL ? "" : msg);
    fflush(stdout);
}

void vibe_log_error(const char *msg) {
    fprintf(stderr, "[error] %s\n", msg == NULL ? "" : msg);
    fflush(stderr);
}

char *vibe_env_get(const char *key) {
    if (key == NULL || key[0] == '\0') {
        return vibe_strdup_or_panic("");
    }
    const char *value = getenv(key);
    return vibe_strdup_or_panic(value == NULL ? "" : value);
}

int64_t vibe_env_has(const char *key) {
    if (key == NULL || key[0] == '\0') {
        return 0;
    }
    return getenv(key) == NULL ? 0 : 1;
}

char *vibe_env_get_required(const char *key) {
    return vibe_env_get(key);
}

/*
 * Recovers this process's own argv as a NUL-separated blob (argv[0], NUL,
 * argv[1], NUL, ...), matching the exact format /proc/self/cmdline already
 * provides on Linux, so vibe_cli_args_len/vibe_cli_arg's parsing below is
 * shared across platforms. Cranelift-compiled VibeLang programs declare
 * `main` with no C-level argc/argv parameters, so this is the only way for
 * cli.args_len/cli.arg to see the process's real command line at all.
 *
 * Found while implementing this task: on macOS there was previously no
 * fallback here at all (only #ifdef __linux__, else empty), so
 * cli.args_len() always returned 0 and cli.arg(i) always returned "" on
 * every non-Linux platform, regardless of index -- not a question of which
 * index carries the port, but a total absence of argv access. _NSGetArgv /
 * _NSGetArgc (from crt_externs.h) is the standard portable way to recover a
 * process's own argv on macOS without changing main's signature, so it is
 * used here to build the identical blob format the Linux path already
 * produces.
 */
static char *vibe_cli_read_cmdline_blob(size_t *out_len) {
#ifdef __linux__
    FILE *f = fopen("/proc/self/cmdline", "rb");
    if (f == NULL) {
        if (out_len != NULL) {
            *out_len = 0;
        }
        return vibe_strdup_or_panic("");
    }
    vibe_string_builder builder;
    vibe_builder_init(&builder, 256);
    char chunk[256];
    while (1) {
        size_t n = fread(chunk, 1, sizeof(chunk), f);
        if (n > 0) {
            vibe_builder_append_bytes(&builder, chunk, n);
        }
        if (n < sizeof(chunk)) {
            if (feof(f) != 0 || ferror(f) != 0) {
                break;
            }
        }
    }
    fclose(f);
    if (out_len != NULL) {
        *out_len = builder.len;
    }
    return builder.data;
#elif defined(__APPLE__)
    char **argv = *_NSGetArgv();
    int argc = *_NSGetArgc();
    if (argv == NULL || argc <= 0) {
        if (out_len != NULL) {
            *out_len = 0;
        }
        return vibe_strdup_or_panic("");
    }
    vibe_string_builder builder;
    vibe_builder_init(&builder, 256);
    for (int i = 0; i < argc; i++) {
        const char *a = argv[i] == NULL ? "" : argv[i];
        vibe_builder_append_bytes(&builder, a, strlen(a));
        vibe_builder_append_bytes(&builder, "\0", 1);
    }
    if (out_len != NULL) {
        *out_len = builder.len;
    }
    return builder.data;
#else
    if (out_len != NULL) {
        *out_len = 0;
    }
    return vibe_strdup_or_panic("");
#endif
}

int64_t vibe_cli_args_len(void) {
    size_t len = 0;
    char *blob = vibe_cli_read_cmdline_blob(&len);
    if (blob == NULL || len == 0) {
        free(blob);
        return 0;
    }
    int64_t count = 0;
    size_t i = 0;
    while (i < len) {
        size_t start = i;
        while (i < len && blob[i] != '\0') {
            i += 1;
        }
        if (i > start) {
            count += 1;
        }
        i += 1;
    }
    free(blob);
    if (count <= 1) {
        return 0;
    }
    return count - 1; // exclude argv[0]
}

char *vibe_cli_arg(int64_t index) {
    if (index < 0) {
        return vibe_strdup_or_panic("");
    }
    size_t len = 0;
    char *blob = vibe_cli_read_cmdline_blob(&len);
    if (blob == NULL || len == 0) {
        free(blob);
        return vibe_strdup_or_panic("");
    }
    int64_t target = index + 1; // skip argv[0]
    int64_t current = 0;
    size_t i = 0;
    while (i < len) {
        size_t start = i;
        while (i < len && blob[i] != '\0') {
            i += 1;
        }
        if (current == target) {
            size_t arg_len = i - start;
            char *out = (char *)calloc(arg_len + 1, sizeof(char));
            if (out == NULL) {
                free(blob);
                vibe_panic("failed to allocate cli.arg output");
            }
            if (arg_len > 0) {
                memcpy(out, blob + start, arg_len);
            }
            out[arg_len] = '\0';
            free(blob);
            return out;
        }
        if (i > start) {
            current += 1;
        }
        i += 1;
    }
    free(blob);
    return vibe_strdup_or_panic("");
}

/* Socket handles are OS fds, not pointers. 0 stays the documented failure
 * sentinel (callers already branch on it) and non-positive values keep
 * their graceful no-op behavior; positive handles must be live fds handed
 * out by listen/accept/connect, otherwise a forged or stale handle could
 * read, write, or close an unrelated file descriptor. */
static int vibe_net_checked_fd(int64_t fd_raw) {
    if (fd_raw > 0) {
        vibe_handle_check(&vibe_net_live, (uintptr_t)fd_raw, "invalid net handle");
    }
    return (int)fd_raw;
}

static void vibe_net_register_fd(int64_t fd_raw) {
    if (fd_raw > 0) {
        vibe_handle_register(&vibe_net_live, (uintptr_t)fd_raw);
    }
}

int64_t vibe_net_listen(const char *host, int64_t port) {
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
    vibe_net_register_fd((int64_t)fd);
    return (int64_t)fd;
#endif
}

int64_t vibe_net_listener_port(int64_t listener_fd) {
#ifdef _WIN32
    (void)listener_fd;
    return 0;
#else
    int fd = vibe_net_checked_fd(listener_fd);
    struct sockaddr_in sa;
    socklen_t len = (socklen_t)sizeof(sa);
    memset(&sa, 0, sizeof(sa));
    if (getsockname(fd, (struct sockaddr *)&sa, &len) != 0) {
        return 0;
    }
    return (int64_t)ntohs(sa.sin_port);
#endif
}

int64_t vibe_net_accept(int64_t listener_fd) {
#ifdef _WIN32
    (void)listener_fd;
    return 0;
#else
    int fd = vibe_net_checked_fd(listener_fd);
    int conn = accept(fd, NULL, NULL);
    if (conn < 0) {
        return 0;
    }
    vibe_net_register_fd((int64_t)conn);
    return (int64_t)conn;
#endif
}

int64_t vibe_net_connect(const char *host, int64_t port) {
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
    vibe_net_register_fd((int64_t)fd);
    return (int64_t)fd;
#endif
}

/*
 * Ceiling shared by vibe_net_read and vibe_net_read_bytes below: a single
 * recv() call never fills more than this many bytes, regardless of what a
 * caller passes as max_bytes. This is the hard requirement this task exists
 * to satisfy -- max_bytes is effectively peer-influenced (a caller may
 * forward a length it just read off the same connection, e.g. a frame's
 * length prefix), so it is clamped to this ceiling BEFORE either function
 * allocates. Named and shared by both call sites so the two copies cannot
 * drift apart.
 */
#define VIBE_NET_READ_MAX_BYTES (4 * 1024 * 1024)

char *vibe_net_read(int64_t fd_raw, int64_t max_bytes_raw) {
#ifdef _WIN32
    (void)fd_raw;
    (void)max_bytes_raw;
    return vibe_strdup_or_panic("");
#else
    int fd = vibe_net_checked_fd(fd_raw);
    int64_t max_bytes = max_bytes_raw;
    if (fd <= 0 || max_bytes <= 0) {
        return vibe_strdup_or_panic("");
    }
    if (max_bytes > VIBE_NET_READ_MAX_BYTES) {
        max_bytes = VIBE_NET_READ_MAX_BYTES;
    }
    char *buffer = (char *)calloc((size_t)max_bytes + 1, sizeof(char));
    if (buffer == NULL) {
        vibe_panic("failed to allocate net.read buffer");
    }
    ssize_t n = recv(fd, buffer, (size_t)max_bytes, 0);
    if (n <= 0) {
        free(buffer);
        return vibe_strdup_or_panic("");
    }
    buffer[(size_t)n] = '\0';
    return buffer;
#endif
}

/*
 * Byte-exact socket read. Mirrors vibe_net_read above but returns the exact
 * number of bytes recv() reported as a Bytes value, instead of
 * NUL-terminating the buffer and throwing that count away. This is the fix
 * for the measured regression: a 16-byte PNG header read through net.read
 * arrived as 8 bytes, because Str length is strlen and the header's ninth
 * byte is 0x00.
 *
 * max_bytes is effectively peer-influenced -- a caller may forward a length
 * it just read off the same connection, e.g. a frame's length prefix -- so
 * it is NEVER trusted past the same VIBE_NET_READ_MAX_BYTES ceiling
 * vibe_net_read already enforces on itself (see the shared #define above
 * vibe_net_read). The clamp happens before any allocation, so this
 * function's worst-case allocation is bounded no matter what max_bytes a
 * caller passes: a call site cannot turn a hostile length into an
 * unbounded calloc, which is what would otherwise make vibe_bytes_new's
 * vibe_panic-on-OOM path (vibe_panic ends in abort()) reachable from a
 * single peer under the `while true { conn := net.accept(l); go handle(conn) }`
 * server pattern this runtime documents -- an abort there kills the
 * listener and every other connection, not just the offending request.
 */
void *vibe_net_read_bytes(int64_t fd_raw, int64_t max_bytes_raw) {
#ifdef _WIN32
    (void)fd_raw;
    (void)max_bytes_raw;
    return vibe_bytes_new(0);
#else
    int fd = vibe_net_checked_fd(fd_raw);
    int64_t max_bytes = max_bytes_raw;
    if (fd <= 0 || max_bytes <= 0) {
        return vibe_bytes_new(0);
    }
    if (max_bytes > VIBE_NET_READ_MAX_BYTES) {
        max_bytes = VIBE_NET_READ_MAX_BYTES;
    }
    char *buffer = (char *)calloc((size_t)max_bytes, sizeof(char));
    if (buffer == NULL) {
        vibe_panic("failed to allocate net.read_bytes buffer");
    }
    ssize_t n = recv(fd, buffer, (size_t)max_bytes, 0);
    if (n <= 0) {
        free(buffer);
        return vibe_bytes_new(0);
    }
    vibe_bytes *out = (vibe_bytes *)vibe_bytes_new((int64_t)n);
    memcpy(out->data, buffer, (size_t)n);
    free(buffer);
    return out;
#endif
}

int64_t vibe_net_write(int64_t fd_raw, const char *data) {
#ifdef _WIN32
    (void)fd_raw;
    (void)data;
    return 0;
#else
    int fd = vibe_net_checked_fd(fd_raw);
    const char *raw = data == NULL ? "" : data;
    size_t len = strlen(raw);
    if (fd <= 0 || len == 0) {
        return 0;
    }
    size_t total = 0;
    while (total < len) {
        ssize_t n = send(fd, raw + total, len - total, 0);
        if (n <= 0) break;
        total += (size_t)n;
    }
    return (int64_t)total;
#endif
}

int64_t vibe_net_close(int64_t fd_raw) {
#ifdef _WIN32
    (void)fd_raw;
    return 0;
#else
    int fd = vibe_net_checked_fd(fd_raw);
    if (fd <= 0) {
        return 0;
    }
    vibe_handle_unregister(&vibe_net_live, (uintptr_t)fd_raw);
    return close(fd) == 0 ? 1 : 0;
#endif
}

char *vibe_net_resolve_first(const char *host) {
#ifdef _WIN32
    (void)host;
    return vibe_strdup_or_panic("");
#else
    const char *query = (host == NULL || host[0] == '\0') ? "localhost" : host;
    struct addrinfo hints;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    struct addrinfo *results = NULL;
    if (getaddrinfo(query, NULL, &hints, &results) != 0 || results == NULL) {
        return vibe_strdup_or_panic("");
    }
    char ip[INET_ADDRSTRLEN];
    ip[0] = '\0';
    for (struct addrinfo *it = results; it != NULL; it = it->ai_next) {
        if (it->ai_family != AF_INET || it->ai_addr == NULL) {
            continue;
        }
        struct sockaddr_in *sa = (struct sockaddr_in *)it->ai_addr;
        if (inet_ntop(AF_INET, &sa->sin_addr, ip, sizeof(ip)) != NULL) {
            break;
        }
    }
    freeaddrinfo(results);
    if (ip[0] == '\0') {
        return vibe_strdup_or_panic("");
    }
    return vibe_strdup_or_panic(ip);
#endif
}

static const char *vibe_trim_start(const char *raw) {
    while (raw != NULL && *raw != '\0' && isspace((unsigned char)*raw)) {
        raw += 1;
    }
    return raw;
}

static const char *vibe_trim_end_ptr(const char *raw) {
    const char *end = raw + strlen(raw);
    while (end > raw && isspace((unsigned char)*(end - 1))) {
        end -= 1;
    }
    return end;
}

static int64_t vibe_parse_i64_strict(const char *start, const char *end, int64_t *out_value) {
    if (start == NULL || end == NULL || start >= end || out_value == NULL) {
        return 0;
    }
    const char *cur = start;
    int negative = 0;
    if (*cur == '+' || *cur == '-') {
        negative = (*cur == '-');
        cur += 1;
        if (cur >= end) {
            return 0;
        }
    }
    uint64_t value = 0;
    uint64_t limit = negative ? ((uint64_t)INT64_MAX + 1ull) : (uint64_t)INT64_MAX;
    int saw_digit = 0;
    while (cur < end) {
        unsigned char ch = (unsigned char)(*cur);
        if (ch < '0' || ch > '9') {
            return 0;
        }
        saw_digit = 1;
        uint64_t digit = (uint64_t)(ch - '0');
        if (value > (limit - digit) / 10ull) {
            return 0;
        }
        value = value * 10ull + digit;
        cur += 1;
    }
    if (!saw_digit) {
        return 0;
    }
    if (negative) {
        if (value == (uint64_t)INT64_MAX + 1ull) {
            *out_value = INT64_MIN;
        } else {
            *out_value = -(int64_t)value;
        }
    } else {
        *out_value = (int64_t)value;
    }
    return 1;
}

typedef struct vibe_md5_ctx {
    uint32_t state[4];
    uint64_t bitlen;
    unsigned char buffer[64];
} vibe_md5_ctx;

static void vibe_format_shortest_f64(double value, char out[64]) {
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

int64_t vibe_convert_to_int(const char *raw) {
    if (raw == NULL) {
        return 0;
    }
    const char *start = vibe_trim_start(raw);
    const char *end = vibe_trim_end_ptr(start);
    int64_t value = 0;
    if (!vibe_parse_i64_strict(start, end, &value)) {
        return 0;
    }
    return value;
}

double vibe_convert_to_f64(const char *raw) {
    if (raw == NULL) {
        return 0.0;
    }
    const char *start = vibe_trim_start(raw);
    const char *end = vibe_trim_end_ptr(start);
    if (end <= start) {
        return 0.0;
    }
    size_t len = (size_t)(end - start);
    char *buf = (char *)calloc(len + 1, sizeof(char));
    if (buf == NULL) {
        vibe_panic("failed to allocate convert.to_float buffer");
    }
    memcpy(buf, start, len);
    buf[len] = '\0';
    char *tail = NULL;
    double value = strtod(buf, &tail);
    int ok = tail != NULL && *tail == '\0';
    free(buf);
    return ok ? value : 0.0;
}

char *vibe_convert_i64_to_str(int64_t value) {
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%lld", (long long)value);
    return vibe_strdup_or_panic(buffer);
}

char *vibe_convert_f64_to_str(double value) {
    char out[64];
    vibe_format_shortest_f64(value, out);
    return vibe_strdup_or_panic(out);
}

double vibe_i64_to_f64(int64_t value) {
    return (double)value;
}

int64_t vibe_f64_to_bits(double value) {
    int64_t bits;
    memcpy(&bits, &value, sizeof(bits));
    return bits;
}

double vibe_f64_from_bits(int64_t bits) {
    double value;
    memcpy(&value, &bits, sizeof(value));
    return value;
}

double vibe_math_sqrt(double x) {
    return sqrt(x);
}

char *vibe_format_f64(double value, int64_t precision) {
    if (precision < 0) precision = 0;
    if (precision > 20) precision = 20;
    char out[64];
    snprintf(out, sizeof(out), "%.*f", (int)precision, value);
    return vibe_strdup_or_panic(out);
}

static uint32_t vibe_md5_left_rotate(uint32_t x, uint32_t c) {
    return (x << c) | (x >> (32u - c));
}

static void vibe_md5_transform(vibe_md5_ctx *ctx, const unsigned char data[64]) {
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
        b = b + vibe_md5_left_rotate(sum, r[i]);
        a = temp;
    }

    ctx->state[0] += a;
    ctx->state[1] += b;
    ctx->state[2] += c;
    ctx->state[3] += d;
}

static void vibe_md5_init(vibe_md5_ctx *ctx) {
    ctx->bitlen = 0;
    ctx->state[0] = 0x67452301u;
    ctx->state[1] = 0xefcdab89u;
    ctx->state[2] = 0x98badcfeu;
    ctx->state[3] = 0x10325476u;
    memset(ctx->buffer, 0, sizeof(ctx->buffer));
}

static void vibe_md5_update(vibe_md5_ctx *ctx, const unsigned char *data, size_t len) {
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
        vibe_md5_transform(ctx, ctx->buffer);
        i += fill;
        idx = 0;
    }
    while (i + 64u <= len) {
        vibe_md5_transform(ctx, data + i);
        i += 64u;
    }
    if (i < len) {
        memcpy(ctx->buffer, data + i, len - i);
    }
}

static void vibe_md5_final(vibe_md5_ctx *ctx, unsigned char out[16]) {
    unsigned char pad[64];
    memset(pad, 0, sizeof(pad));
    pad[0] = 0x80;

    size_t idx = (size_t)((ctx->bitlen / 8u) % 64u);
    size_t pad_len = (idx < 56u) ? (56u - idx) : (120u - idx);
    uint64_t original_bits = ctx->bitlen;
    vibe_md5_update(ctx, pad, pad_len);

    unsigned char len_bytes[8];
    uint64_t bits = original_bits;
    for (int i = 0; i < 8; i++) {
        len_bytes[i] = (unsigned char)((bits >> (8u * (uint32_t)i)) & 0xffu);
    }
    vibe_md5_update(ctx, len_bytes, 8);

    for (int i = 0; i < 4; i++) {
        out[i * 4] = (unsigned char)(ctx->state[i] & 0xffu);
        out[i * 4 + 1] = (unsigned char)((ctx->state[i] >> 8u) & 0xffu);
        out[i * 4 + 2] = (unsigned char)((ctx->state[i] >> 16u) & 0xffu);
        out[i * 4 + 3] = (unsigned char)((ctx->state[i] >> 24u) & 0xffu);
    }
}

char *vibe_md5_bytes_hex(void *handle) {
    vibe_list_i64 *list = (vibe_list_i64 *)handle;
    if (list == NULL || list->tag != VIBE_CONTAINER_LIST_I64) {
        vibe_panic("md5_bytes_hex called on non-list handle");
    }
    size_t len = (size_t)list->len;
    unsigned char *buf = (unsigned char *)malloc(len);
    if (buf == NULL) vibe_panic("md5_bytes_hex: alloc failed");
    for (size_t i = 0; i < len; i++) {
        buf[i] = (unsigned char)(list->items[i] & 0xff);
    }
    vibe_md5_ctx ctx;
    vibe_md5_init(&ctx);
    vibe_md5_update(&ctx, buf, len);
    unsigned char digest[16];
    vibe_md5_final(&ctx, digest);
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

char *vibe_md5_hex(const char *raw) {
    const unsigned char *bytes = (const unsigned char *)(raw == NULL ? "" : raw);
    size_t len = strlen((const char *)bytes);
    vibe_md5_ctx ctx;
    vibe_md5_init(&ctx);
    vibe_md5_update(&ctx, bytes, len);
    unsigned char digest[16];
    vibe_md5_final(&ctx, digest);

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

char *vibe_json_canonical(const char *raw) {
    vibe_counter_inc(&vibe_json_canonical_calls);
    if (raw == NULL) {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic("");
    }
    size_t len = strlen(raw);
    vibe_string_builder builder;
    vibe_builder_init(&builder, len + 32);

    int in_string = 0;
    int escaped = 0;
    size_t i = 0;
    while (i < len) {
        char ch = raw[i];
        if (in_string) {
            vibe_builder_append_bytes(&builder, &ch, 1);
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
            vibe_builder_append_bytes(&builder, &ch, 1);
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
                vibe_builder_append_bytes(&builder, raw + start, tok_len);
            } else {
                char num_buf[128];
                size_t copy_len = tok_len < sizeof(num_buf) - 1 ? tok_len : sizeof(num_buf) - 1;
                memcpy(num_buf, raw + start, copy_len);
                num_buf[copy_len] = '\0';
                char *endptr = NULL;
                double value = strtod(num_buf, &endptr);
                if (endptr == num_buf) {
                    vibe_builder_append_bytes(&builder, raw + start, tok_len);
                } else {
                    char out_num[64];
                    vibe_format_shortest_f64(value, out_num);
                    int wrote = (int)strlen(out_num);
                    if (wrote <= 0) {
                        vibe_builder_append_bytes(&builder, raw + start, tok_len);
                    } else {
                        vibe_builder_append_bytes(&builder, out_num, (size_t)wrote);
                    }
                }
            }
            continue;
        }
        vibe_builder_append_bytes(&builder, &ch, 1);
        i += 1;
    }

    vibe_counter_inc(&vibe_json_allocations);
    return builder.data;
}

/*
 * Scans a JSON number token starting at p, following the RFC 8259 grammar
 * for the exponent: after 'e'/'E' an optional single '+' or '-', then at
 * least one digit. '+' is accepted only in that exponent-sign position.
 * Returns the end of the token (setting *out_saw_float when a fraction or
 * exponent was seen), or NULL when the text at p is not a number token.
 * The exponent digits are enforced here rather than left to strtod because
 * strtod's handling of dangling exponents ("1e-") varies across libcs.
 */
static const char *vibe_json_scan_number(const char *p, int *out_saw_float) {
    int saw_float = 0;
    int saw_mantissa_digit = 0;
    if (*p == '-') {
        p += 1;
    }
    while (*p >= '0' && *p <= '9') {
        saw_mantissa_digit = 1;
        p += 1;
    }
    if (*p == '.') {
        saw_float = 1;
        p += 1;
        while (*p >= '0' && *p <= '9') {
            saw_mantissa_digit = 1;
            p += 1;
        }
    }
    if (!saw_mantissa_digit) {
        return NULL;
    }
    if (*p == 'e' || *p == 'E') {
        const char *exp = p + 1;
        if (*exp == '+' || *exp == '-') {
            exp += 1;
        }
        if (*exp < '0' || *exp > '9') {
            return NULL;
        }
        saw_float = 1;
        p = exp;
        while (*p >= '0' && *p <= '9') {
            p += 1;
        }
    }
    *out_saw_float = saw_float;
    return p;
}

/*
 * `vibe_json_is_valid` used to live here as a first/last-non-space-character
 * check that never ran the grammar: it answered true for "[x]" and "[1,2,]".
 * It is now defined next to the parser it must agree with (search
 * VIBE_JSON_MAX_DEPTH), because agreement with json.parse is the whole point of
 * the function.
 */

static char *vibe_json_quote_string(const char *raw) {
    const char *text = raw == NULL ? "" : raw;
    vibe_string_builder builder;
    vibe_builder_init(&builder, strlen(text) + 16);
    vibe_builder_append_bytes(&builder, "\"", 1);
    for (const unsigned char *p = (const unsigned char *)text; *p != '\0'; p++) {
        switch (*p) {
            case '"':
                vibe_builder_append_bytes(&builder, "\\\"", 2);
                break;
            case '\\':
                vibe_builder_append_bytes(&builder, "\\\\", 2);
                break;
            case '\n':
                vibe_builder_append_bytes(&builder, "\\n", 2);
                break;
            case '\r':
                vibe_builder_append_bytes(&builder, "\\r", 2);
                break;
            case '\t':
                vibe_builder_append_bytes(&builder, "\\t", 2);
                break;
            default:
                if (*p < 0x20) {
                    char escaped[7];
                    snprintf(escaped, sizeof(escaped), "\\u%04x", (unsigned int)*p);
                    vibe_builder_append_bytes(&builder, escaped, 6);
                } else {
                    char ch = (char)*p;
                    vibe_builder_append_bytes(&builder, &ch, 1);
                }
                break;
        }
    }
    vibe_builder_append_bytes(&builder, "\"", 1);
    vibe_counter_inc(&vibe_json_allocations);
    return builder.data;
}

/* --- Minimal process-local metrics (counters + gauges), mutex-protected linked list --- */

typedef struct vibe_metric_entry {
    char *name;
    int is_gauge;
    int64_t value;
    struct vibe_metric_entry *next;
} vibe_metric_entry;

static pthread_mutex_t vibe_metrics_mu = PTHREAD_MUTEX_INITIALIZER;
static vibe_metric_entry *vibe_metrics_head = NULL;

static vibe_metric_entry *vibe_metrics_find_locked(const char *name, int is_gauge) {
    const char *key = name == NULL ? "" : name;
    for (vibe_metric_entry *e = vibe_metrics_head; e != NULL; e = e->next) {
        if (e->is_gauge == is_gauge && strcmp(e->name, key) == 0) {
            return e;
        }
    }
    return NULL;
}

static vibe_metric_entry *vibe_metrics_get_or_create_locked(const char *name, int is_gauge) {
    vibe_metric_entry *e = vibe_metrics_find_locked(name, is_gauge);
    if (e != NULL) {
        return e;
    }
    e = (vibe_metric_entry *)calloc(1, sizeof(vibe_metric_entry));
    if (e == NULL) {
        vibe_panic("vibe_metrics: entry allocation failed");
    }
    const char *key = name == NULL ? "" : name;
    e->name = vibe_strdup_or_panic(key);
    e->is_gauge = is_gauge;
    e->value = 0;
    e->next = vibe_metrics_head;
    vibe_metrics_head = e;
    return e;
}

void vibe_metrics_counter_inc(const char *name, int64_t delta) {
    pthread_mutex_lock(&vibe_metrics_mu);
    vibe_metric_entry *e = vibe_metrics_get_or_create_locked(name, 0);
    e->value += delta;
    pthread_mutex_unlock(&vibe_metrics_mu);
}

int64_t vibe_metrics_counter_get(const char *name) {
    pthread_mutex_lock(&vibe_metrics_mu);
    vibe_metric_entry *e = vibe_metrics_find_locked(name, 0);
    int64_t v = e != NULL ? e->value : 0;
    pthread_mutex_unlock(&vibe_metrics_mu);
    return v;
}

void vibe_metrics_gauge_set(const char *name, int64_t value) {
    pthread_mutex_lock(&vibe_metrics_mu);
    vibe_metric_entry *e = vibe_metrics_get_or_create_locked(name, 1);
    e->value = value;
    pthread_mutex_unlock(&vibe_metrics_mu);
}

int64_t vibe_metrics_gauge_get(const char *name) {
    pthread_mutex_lock(&vibe_metrics_mu);
    vibe_metric_entry *e = vibe_metrics_find_locked(name, 1);
    int64_t v = e != NULL ? e->value : 0;
    pthread_mutex_unlock(&vibe_metrics_mu);
    return v;
}

char *vibe_metrics_snapshot_json(void) {
    pthread_mutex_lock(&vibe_metrics_mu);
    vibe_string_builder builder;
    vibe_builder_init(&builder, 128);
    vibe_builder_append_bytes(&builder, "{\"counters\":{", 13);
    int need_comma = 0;
    for (vibe_metric_entry *e = vibe_metrics_head; e != NULL; e = e->next) {
        if (e->is_gauge) {
            continue;
        }
        if (need_comma) {
            vibe_builder_append_bytes(&builder, ",", 1);
        }
        need_comma = 1;
        char *quoted = vibe_json_quote_string(e->name);
        vibe_builder_append_bytes(&builder, quoted, strlen(quoted));
        free(quoted);
        vibe_builder_append_bytes(&builder, ":", 1);
        char num[32];
        snprintf(num, sizeof(num), "%lld", (long long)e->value);
        vibe_builder_append_bytes(&builder, num, strlen(num));
    }
    vibe_builder_append_bytes(&builder, "},\"gauges\":{", 12);
    need_comma = 0;
    for (vibe_metric_entry *e = vibe_metrics_head; e != NULL; e = e->next) {
        if (!e->is_gauge) {
            continue;
        }
        if (need_comma) {
            vibe_builder_append_bytes(&builder, ",", 1);
        }
        need_comma = 1;
        char *quoted = vibe_json_quote_string(e->name);
        vibe_builder_append_bytes(&builder, quoted, strlen(quoted));
        free(quoted);
        vibe_builder_append_bytes(&builder, ":", 1);
        char num[32];
        snprintf(num, sizeof(num), "%lld", (long long)e->value);
        vibe_builder_append_bytes(&builder, num, strlen(num));
    }
    vibe_builder_append_bytes(&builder, "}}", 2);
    pthread_mutex_unlock(&vibe_metrics_mu);
    vibe_counter_inc(&vibe_json_allocations);
    return builder.data;
}

static int64_t vibe_looks_like_integer(const char *s) {
    if (s == NULL || *s == '\0') return 0;
    const char *p = s;
    if (*p == '-') p++;
    if (*p == '\0') return 0;
    while (*p != '\0') {
        if (*p < '0' || *p > '9') return 0;
        p++;
    }
    return 1;
}

char *vibe_json_from_str_str_map(void *handle) {
    vibe_map_str_str *map = (vibe_map_str_str *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_STR) {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic("{}");
    }
    if (map->count == 0) {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic("{}");
    }
    vibe_string_builder builder;
    vibe_builder_init(&builder, (size_t)(map->count * 32 + 4));
    vibe_builder_append_bytes(&builder, "{", 1);
    for (int64_t i = 0; i < map->count; i++) {
        if (i > 0) {
            vibe_builder_append_bytes(&builder, ",", 1);
        }
        char *quoted_key = vibe_json_quote_string(map->entries[i].key);
        vibe_builder_append_bytes(&builder, quoted_key, strlen(quoted_key));
        free(quoted_key);
        vibe_builder_append_bytes(&builder, ":", 1);
        const char *val = map->entries[i].value;
        if (val == NULL) val = "";
        if (vibe_looks_like_integer(val)) {
            vibe_builder_append_bytes(&builder, val, strlen(val));
        } else if (strcmp(val, "true") == 0 || strcmp(val, "false") == 0) {
            vibe_builder_append_bytes(&builder, val, strlen(val));
        } else {
            char *quoted_val = vibe_json_quote_string(val);
            vibe_builder_append_bytes(&builder, quoted_val, strlen(quoted_val));
            free(quoted_val);
        }
    }
    vibe_builder_append_bytes(&builder, "}", 1);
    vibe_counter_inc(&vibe_json_allocations);
    return builder.data;
}

static const char *vibe_json_skip_ws(const char *p);
static char *vibe_json_unquote_string(const char *start, const char *end);
char *vibe_json_stringify_i64(int64_t value);

typedef enum vibe_json_kind {
    VIBE_JSON_NULL = 0,
    VIBE_JSON_BOOL = 1,
    VIBE_JSON_I64 = 2,
    VIBE_JSON_F64 = 3,
    VIBE_JSON_STR = 4,
    VIBE_JSON_ARRAY = 5,
    VIBE_JSON_OBJECT = 6,
} vibe_json_kind;

typedef struct vibe_json_value vibe_json_value;

typedef struct vibe_json_array {
    vibe_json_value **items;
    int64_t count;
    int64_t capacity;
} vibe_json_array;

typedef struct vibe_json_object {
    char **keys;
    vibe_json_value **values;
    int64_t count;
    int64_t capacity;
} vibe_json_object;

struct vibe_json_value {
    int64_t kind;
    union {
        int64_t bool_value;
        int64_t int_value;
        double float_value;
        char *str_value;
        vibe_json_array array;
        vibe_json_object object;
    } as;
};

typedef struct vibe_json_builder_ctx {
    int64_t kind;
    int first;
    int expecting_value;
} vibe_json_builder_ctx;

typedef struct vibe_json_builder_handle {
    vibe_string_builder out;
    vibe_json_builder_ctx *stack;
    int64_t depth;
    int64_t stack_cap;
    int root_written;
} vibe_json_builder_handle;

static vibe_json_value *vibe_json_new_value(int64_t kind) {
    vibe_json_value *value = (vibe_json_value *)calloc(1, sizeof(vibe_json_value));
    if (value == NULL) {
        vibe_panic("failed to allocate Json value");
    }
    value->kind = kind;
    return value;
}

static void vibe_json_array_push(vibe_json_array *array, vibe_json_value *value) {
    if (array->count >= array->capacity) {
        int64_t next_cap = array->capacity <= 0 ? 4 : array->capacity * 2;
        vibe_json_value **next_items = (vibe_json_value **)realloc(
            array->items,
            (size_t)next_cap * sizeof(vibe_json_value *)
        );
        if (next_items == NULL) {
            vibe_panic("failed to grow Json array");
        }
        array->items = next_items;
        array->capacity = next_cap;
    }
    array->items[array->count++] = value;
}

static void vibe_json_object_put(vibe_json_object *object, const char *key, vibe_json_value *value) {
    if (object->count >= object->capacity) {
        int64_t next_cap = object->capacity <= 0 ? 4 : object->capacity * 2;
        char **next_keys = (char **)realloc(object->keys, (size_t)next_cap * sizeof(char *));
        vibe_json_value **next_values = (vibe_json_value **)realloc(
            object->values,
            (size_t)next_cap * sizeof(vibe_json_value *)
        );
        if (next_keys == NULL || next_values == NULL) {
            vibe_panic("failed to grow Json object");
        }
        object->keys = next_keys;
        object->values = next_values;
        object->capacity = next_cap;
    }
    object->keys[object->count] = vibe_strdup_or_panic(key == NULL ? "" : key);
    object->values[object->count] = value;
    object->count += 1;
}

/*
 * VL-HTTP-03. vibe_json_parse_value_internal, vibe_json_parse_array_value and
 * vibe_json_parse_object_value are mutually recursive and had no limit, so a
 * request body of repeated '[' walked the handler thread's stack off its end.
 * That is not a recoverable error: the process dies by signal, taking the
 * listener and every other in-flight connection with it. Every recursive entry
 * now carries its nesting depth and is refused past VIBE_JSON_MAX_DEPTH.
 *
 * The cap is measured, not guessed. Method: an instrumented build printed the
 * address of a stack local in vibe_json_parse_value_internal at depth 1 and at
 * depth 401 of the same document (aarch64-apple-darwin, cc -O2 -std=c11, the
 * flags crates/vibe_runtime/src/lib.rs:95-115 uses for a release build). The
 * frame delta is a property of the compiled code, so it is exact and does not
 * depend on catching a crash:
 *   - `go handle(conn)` thread: frames 0x16f8d6dd7 -> 0x16f8bddd7 across 400
 *     levels = 0x19000 = 102400 bytes, i.e. exactly 256 bytes per nesting level
 *     across the two mutually recursive frames.
 *   - main thread: 0x16d1aa227 -> 0x16d191227, the same 102400 bytes / 400
 *     levels = 256 bytes per level. Two different stacks, one answer.
 *   - The thread that matters is the `go` one: pthread_create(NULL attr) +
 *     pthread_detach, whose stack pthread_get_stacksize_np reported as 536576
 *     bytes (the 512 KB macOS default), NOT the 8372224-byte main stack.
 *   - 256 levels therefore cost 65536 bytes: 12.2% of that 536576-byte stack
 *     (12.5% of a nominal 512 KB), leaving the rest to the handler frames
 *     sitting above the parser. 512 levels would have cost 131072 bytes, which
 *     is exactly 25.0% of a nominal 512 KB stack -- at the review bar rather
 *     than under it -- so the cap is 256 and not the 512 first proposed.
 *   - Independently, on an unpatched build the deepest body a live `go handle`
 *     thread survived was 1969 levels (1970 was fatal, exit 138 = SIGBUS), so
 *     the cap sits 7.7x below the depth that actually kills a handler.
 *   - The deepest legitimate JSON document anywhere in this repository is 7
 *     levels (editor-support/vscode/syntaxes/vibelang.tmLanguage.json, found by
 *     walking every .json file and every JSON literal in .yb/.rs/.md sources),
 *     so the cap sits ~36x above real traffic.
 *
 * json.parse, json.try_parse and json.is_valid all route through this one core,
 * so the three can never disagree about what is parseable.
 */
#define VIBE_JSON_MAX_DEPTH 256
#define VIBE_JSON_DEPTH_STR_INNER(x) #x
#define VIBE_JSON_DEPTH_STR(x) VIBE_JSON_DEPTH_STR_INNER(x)

typedef enum vibe_json_parse_status {
    VIBE_JSON_PARSE_OK = 0,
    VIBE_JSON_PARSE_MALFORMED = 1,
    VIBE_JSON_PARSE_TRAILING = 2,
    VIBE_JSON_PARSE_TOO_DEEP = 3,
} vibe_json_parse_status;

typedef struct vibe_json_parse_state {
    vibe_json_parse_status status;
} vibe_json_parse_state;

/*
 * MATERIALIZING vs VALIDATING. Throughout this parser a NULL `out` (and a NULL
 * `out_str` in vibe_json_parse_string_value) means "run the grammar but build
 * nothing": every allocation is skipped, only the accept/reject verdict and the
 * end pointer come back. json.is_valid uses that mode.
 *
 * It is one code path, not a second parser, and that is deliberate on both
 * counts:
 *   - CORRECTNESS. json.is_valid's whole contract is that is_valid(s) == true
 *     implies json.parse(s) returns. A separately written validator can drift
 *     from the parser; a `if (out != NULL)` around each allocation cannot,
 *     because the grammar, the trailing-content rule and the depth cap are
 *     literally the same statements. (The version this replaced compared the
 *     first and last non-space characters and answered true for "[x]".)
 *   - MEMORY. Building the DOM just to throw it away would make json.is_valid
 *     allocate on the order of the body size -- measured at ~30x the body in
 *     transient heap for a hostile body of nothing but '[' -- which would hand
 *     a peer a memory-amplification lever on a server that only validates and
 *     forwards. Validation therefore allocates nothing per node: no value
 *     nodes, no array/object tables, no unquoted strings, no object keys. The
 *     only heap it touches is the small fixed-size buffer strtod needs for one
 *     float at a time, which is freed before the next token is read, and which
 *     the previous implementation allocated too.
 */
static const char *vibe_json_parse_value_internal(
    const char *p,
    vibe_json_value **out,
    int64_t depth,
    vibe_json_parse_state *state
);

/*
 * Release a value tree. Only ever called on trees that failed to parse, which
 * are unreachable from anywhere else by construction (nothing has been handed
 * to the caller yet). Without it, an entry point that SURVIVES malformed input
 * -- json.try_parse and the rewritten json.is_valid -- would leak the partial
 * tree on every hostile request, which would trade an abort for a slow leak.
 * The recursion here is bounded by VIBE_JSON_MAX_DEPTH, because that is the
 * deepest tree the parser is able to build.
 */
static void vibe_json_free_value(vibe_json_value *value) {
    if (value == NULL) {
        return;
    }
    switch ((vibe_json_kind)value->kind) {
        case VIBE_JSON_STR:
            free(value->as.str_value);
            break;
        case VIBE_JSON_ARRAY:
            for (int64_t i = 0; i < value->as.array.count; i++) {
                vibe_json_free_value(value->as.array.items[i]);
            }
            free(value->as.array.items);
            break;
        case VIBE_JSON_OBJECT:
            for (int64_t i = 0; i < value->as.object.count; i++) {
                free(value->as.object.keys[i]);
                vibe_json_free_value(value->as.object.values[i]);
            }
            free(value->as.object.keys);
            free(value->as.object.values);
            break;
        default:
            break;
    }
    free(value);
}

/* out_str == NULL scans the string without unquoting it (validate-only mode). */
static const char *vibe_json_parse_string_value(const char *p, char **out_str) {
    if (p == NULL || *p != '"') {
        return NULL;
    }
    const char *start = p;
    int escaped = 0;
    p += 1;
    while (*p != '\0') {
        if (escaped) {
            escaped = 0;
        } else if (*p == '\\') {
            escaped = 1;
        } else if (*p == '"') {
            p += 1;
            if (out_str != NULL) {
                *out_str = vibe_json_unquote_string(start, p);
            }
            return p;
        }
        p += 1;
    }
    return NULL;
}

static const char *vibe_json_parse_number_value(const char *p, vibe_json_value **out) {
    const char *start = p;
    int saw_float = 0;
    p = vibe_json_scan_number(p, &saw_float);
    if (p == NULL) {
        return NULL;
    }
    if (!saw_float) {
        int64_t value = 0;
        if (!vibe_parse_i64_strict(start, p, &value)) {
            return NULL;
        }
        if (out != NULL) {
            vibe_json_value *json = vibe_json_new_value(VIBE_JSON_I64);
            json->as.int_value = value;
            *out = json;
        }
        return p;
    }
    /*
     * The copy is what makes strtod's "did it consume the whole token" answer
     * exact, so validate-only mode keeps it rather than running strtod over the
     * rest of the document: acceptance of numbers must not differ between
     * json.is_valid and json.parse. It is bounded by one number's length and is
     * freed before the next token, so it does not amplify with body size.
     */
    size_t len = (size_t)(p - start);
    char *buffer = (char *)calloc(len + 1, sizeof(char));
    if (buffer == NULL) {
        vibe_panic("failed to allocate JSON number buffer");
    }
    memcpy(buffer, start, len);
    buffer[len] = '\0';
    char *endptr = NULL;
    double value = strtod(buffer, &endptr);
    int fully_consumed = endptr != NULL && *endptr == '\0';
    free(buffer);
    if (!fully_consumed) {
        return NULL;
    }
    if (out != NULL) {
        vibe_json_value *json = vibe_json_new_value(VIBE_JSON_F64);
        json->as.float_value = value;
        *out = json;
    }
    return p;
}

static const char *vibe_json_parse_array_value(
    const char *p,
    vibe_json_value **out,
    int64_t depth,
    vibe_json_parse_state *state
) {
    if (p == NULL || *p != '[') {
        return NULL;
    }
    if (depth >= VIBE_JSON_MAX_DEPTH) {
        state->status = VIBE_JSON_PARSE_TOO_DEEP;
        return NULL;
    }
    vibe_json_value *json = out != NULL ? vibe_json_new_value(VIBE_JSON_ARRAY) : NULL;
    p = vibe_json_skip_ws(p + 1);
    if (*p == ']') {
        if (out != NULL) {
            *out = json;
        }
        return p + 1;
    }
    while (*p != '\0') {
        vibe_json_value *item = NULL;
        p = vibe_json_parse_value_internal(p, out != NULL ? &item : NULL, depth + 1, state);
        if (p == NULL) {
            vibe_json_free_value(json);
            return NULL;
        }
        if (json != NULL) {
            vibe_json_array_push(&json->as.array, item);
        }
        p = vibe_json_skip_ws(p);
        if (*p == ',') {
            p = vibe_json_skip_ws(p + 1);
            continue;
        }
        if (*p == ']') {
            if (out != NULL) {
                *out = json;
            }
            return p + 1;
        }
        vibe_json_free_value(json);
        return NULL;
    }
    vibe_json_free_value(json);
    return NULL;
}

static const char *vibe_json_parse_object_value(
    const char *p,
    vibe_json_value **out,
    int64_t depth,
    vibe_json_parse_state *state
) {
    if (p == NULL || *p != '{') {
        return NULL;
    }
    if (depth >= VIBE_JSON_MAX_DEPTH) {
        state->status = VIBE_JSON_PARSE_TOO_DEEP;
        return NULL;
    }
    vibe_json_value *json = out != NULL ? vibe_json_new_value(VIBE_JSON_OBJECT) : NULL;
    p = vibe_json_skip_ws(p + 1);
    if (*p == '}') {
        if (out != NULL) {
            *out = json;
        }
        return p + 1;
    }
    while (*p != '\0') {
        char *key = NULL;
        p = vibe_json_parse_string_value(p, out != NULL ? &key : NULL);
        if (p == NULL) {
            vibe_json_free_value(json);
            return NULL;
        }
        p = vibe_json_skip_ws(p);
        if (*p != ':') {
            free(key);
            vibe_json_free_value(json);
            return NULL;
        }
        p = vibe_json_skip_ws(p + 1);
        vibe_json_value *value = NULL;
        p = vibe_json_parse_value_internal(p, out != NULL ? &value : NULL, depth + 1, state);
        if (p == NULL) {
            free(key);
            vibe_json_free_value(json);
            return NULL;
        }
        if (json != NULL) {
            vibe_json_object_put(&json->as.object, key, value);
        }
        free(key);
        p = vibe_json_skip_ws(p);
        if (*p == ',') {
            p = vibe_json_skip_ws(p + 1);
            continue;
        }
        if (*p == '}') {
            if (out != NULL) {
                *out = json;
            }
            return p + 1;
        }
        vibe_json_free_value(json);
        return NULL;
    }
    vibe_json_free_value(json);
    return NULL;
}

static const char *vibe_json_parse_value_internal(
    const char *p,
    vibe_json_value **out,
    int64_t depth,
    vibe_json_parse_state *state
) {
    p = vibe_json_skip_ws(p);
    if (p == NULL || *p == '\0') {
        return NULL;
    }
    if (*p == '{') {
        return vibe_json_parse_object_value(p, out, depth, state);
    }
    if (*p == '[') {
        return vibe_json_parse_array_value(p, out, depth, state);
    }
    if (*p == '"') {
        char *value = NULL;
        const char *after = vibe_json_parse_string_value(p, out != NULL ? &value : NULL);
        if (after == NULL) {
            return NULL;
        }
        if (out != NULL) {
            vibe_json_value *json = vibe_json_new_value(VIBE_JSON_STR);
            json->as.str_value = value;
            *out = json;
        }
        return after;
    }
    if (strncmp(p, "true", 4) == 0) {
        if (out != NULL) {
            vibe_json_value *json = vibe_json_new_value(VIBE_JSON_BOOL);
            json->as.bool_value = 1;
            *out = json;
        }
        return p + 4;
    }
    if (strncmp(p, "false", 5) == 0) {
        if (out != NULL) {
            vibe_json_value *json = vibe_json_new_value(VIBE_JSON_BOOL);
            json->as.bool_value = 0;
            *out = json;
        }
        return p + 5;
    }
    if (strncmp(p, "null", 4) == 0) {
        if (out != NULL) {
            *out = vibe_json_new_value(VIBE_JSON_NULL);
        }
        return p + 4;
    }
    return vibe_json_parse_number_value(p, out);
}

static void vibe_json_stringify_into(
    vibe_string_builder *builder,
    const vibe_json_value *value,
    int pretty,
    int indent,
    int level
) {
    if (value == NULL) {
        vibe_builder_append_bytes(builder, "null", 4);
        return;
    }
    switch ((vibe_json_kind)value->kind) {
        case VIBE_JSON_NULL:
            vibe_builder_append_bytes(builder, "null", 4);
            break;
        case VIBE_JSON_BOOL:
            vibe_builder_append_bytes(builder, value->as.bool_value ? "true" : "false", value->as.bool_value ? 4 : 5);
            break;
        case VIBE_JSON_I64: {
            char *encoded = vibe_json_stringify_i64(value->as.int_value);
            vibe_builder_append_bytes(builder, encoded, strlen(encoded));
            free(encoded);
            break;
        }
        case VIBE_JSON_F64: {
            char out_num[64];
            vibe_format_shortest_f64(value->as.float_value, out_num);
            vibe_builder_append_bytes(builder, out_num, strlen(out_num));
            break;
        }
        case VIBE_JSON_STR: {
            char *quoted = vibe_json_quote_string(value->as.str_value);
            vibe_builder_append_bytes(builder, quoted, strlen(quoted));
            free(quoted);
            break;
        }
        case VIBE_JSON_ARRAY: {
            vibe_builder_append_bytes(builder, "[", 1);
            for (int64_t i = 0; i < value->as.array.count; i++) {
                if (i > 0) {
                    vibe_builder_append_bytes(builder, ",", 1);
                }
                if (pretty) {
                    vibe_builder_append_bytes(builder, "\n", 1);
                    for (int j = 0; j < (level + 1) * indent; j++) {
                        vibe_builder_append_bytes(builder, " ", 1);
                    }
                }
                vibe_json_stringify_into(builder, value->as.array.items[i], pretty, indent, level + 1);
            }
            if (pretty && value->as.array.count > 0) {
                vibe_builder_append_bytes(builder, "\n", 1);
                for (int j = 0; j < level * indent; j++) {
                    vibe_builder_append_bytes(builder, " ", 1);
                }
            }
            vibe_builder_append_bytes(builder, "]", 1);
            break;
        }
        case VIBE_JSON_OBJECT: {
            vibe_builder_append_bytes(builder, "{", 1);
            for (int64_t i = 0; i < value->as.object.count; i++) {
                if (i > 0) {
                    vibe_builder_append_bytes(builder, ",", 1);
                }
                if (pretty) {
                    vibe_builder_append_bytes(builder, "\n", 1);
                    for (int j = 0; j < (level + 1) * indent; j++) {
                        vibe_builder_append_bytes(builder, " ", 1);
                    }
                }
                char *quoted_key = vibe_json_quote_string(value->as.object.keys[i]);
                vibe_builder_append_bytes(builder, quoted_key, strlen(quoted_key));
                free(quoted_key);
                vibe_builder_append_bytes(builder, pretty ? ": " : ":", pretty ? 2 : 1);
                vibe_json_stringify_into(builder, value->as.object.values[i], pretty, indent, level + 1);
            }
            if (pretty && value->as.object.count > 0) {
                vibe_builder_append_bytes(builder, "\n", 1);
                for (int j = 0; j < level * indent; j++) {
                    vibe_builder_append_bytes(builder, " ", 1);
                }
            }
            vibe_builder_append_bytes(builder, "}", 1);
            break;
        }
    }
}

void *vibe_json_null(void) {
    return vibe_json_new_value(VIBE_JSON_NULL);
}

void *vibe_json_bool(int64_t value) {
    vibe_json_value *json = vibe_json_new_value(VIBE_JSON_BOOL);
    json->as.bool_value = value != 0 ? 1 : 0;
    return json;
}

void *vibe_json_i64(int64_t value) {
    vibe_json_value *json = vibe_json_new_value(VIBE_JSON_I64);
    json->as.int_value = value;
    return json;
}

void *vibe_json_f64(double value) {
    vibe_json_value *json = vibe_json_new_value(VIBE_JSON_F64);
    json->as.float_value = value;
    return json;
}

void *vibe_json_str(const char *value) {
    vibe_json_value *json = vibe_json_new_value(VIBE_JSON_STR);
    json->as.str_value = vibe_strdup_or_panic(value == NULL ? "" : value);
    return json;
}

/*
 * The one parse core behind json.parse, json.try_parse and json.is_valid.
 * Returns the end pointer on success (with *out owning the tree), or NULL with
 * state->status saying WHY, so each entry point can pick its own reaction to
 * the same verdict without any of them re-deciding the grammar.
 *
 * out == NULL means validate-only: same grammar, same depth cap, same
 * trailing-content rule, no tree built and nothing to free.
 */
static const char *vibe_json_parse_document(
    const char *raw,
    vibe_json_value **out,
    vibe_json_parse_state *state
) {
    state->status = VIBE_JSON_PARSE_OK;
    if (out != NULL) {
        *out = NULL;
    }
    vibe_json_value *value = NULL;
    const char *end = vibe_json_parse_value_internal(raw, out != NULL ? &value : NULL, 0, state);
    if (end == NULL) {
        if (state->status == VIBE_JSON_PARSE_OK) {
            state->status = VIBE_JSON_PARSE_MALFORMED;
        }
        vibe_json_free_value(value);
        return NULL;
    }
    end = vibe_json_skip_ws(end);
    if (end == NULL || *end != '\0') {
        state->status = VIBE_JSON_PARSE_TRAILING;
        vibe_json_free_value(value);
        return NULL;
    }
    if (out != NULL) {
        *out = value;
    }
    return end;
}

/*
 * UNCHANGED CONTRACT: json.parse still aborts on malformed input. A program
 * parsing its own trusted data (a build script reading a manifest it wrote)
 * should keep failing loudly; trust is a property of the code path, so a server
 * reaching for peer bytes picks json.try_parse instead.
 *
 * The one behavioural change is the depth cap, and it is strictly an
 * improvement even for trusted data: input past VIBE_JSON_MAX_DEPTH used to
 * exhaust the stack and die by signal with no diagnostic, and now produces a
 * deterministic, distinctly-worded panic.
 */
void *vibe_json_parse(const char *raw) {
    vibe_counter_inc(&vibe_json_parse_calls);
    if (raw == NULL) {
        vibe_panic("json.parse received null input");
    }
    vibe_json_value *value = NULL;
    vibe_json_parse_state state;
    if (vibe_json_parse_document(raw, &value, &state) == NULL) {
        if (state.status == VIBE_JSON_PARSE_TOO_DEEP) {
            vibe_panic(
                "json.parse exceeds maximum nesting depth of "
                VIBE_JSON_DEPTH_STR(VIBE_JSON_MAX_DEPTH)
            );
        }
        if (state.status == VIBE_JSON_PARSE_TRAILING) {
            vibe_panic("json.parse invalid trailing content");
        }
        vibe_panic("json.parse invalid JSON");
    }
    return value;
}

/*
 * VL-HTTP-04: recoverable parse for JSON that arrived from outside the process.
 * TOTAL with respect to input -- no byte sequence, however malformed and
 * however deeply nested, reaches vibe_panic through this entry point, so eight
 * bytes of garbage in a POST body can no longer end the server.
 *
 * Returns the standard 2-slot Result record: slot 0 is the tag (0 = Ok,
 * 1 = Err), slot 1 is the payload (a Json node, or a Str explaining the
 * rejection). Read it from VibeLang with result.is_ok / result.is_err plus
 * json.result_value / json.result_error.
 */
void *vibe_json_try_parse(const char *raw) {
    vibe_counter_inc(&vibe_json_parse_calls);
    void *result = vibe_record_alloc(2);
    int64_t *slots = (int64_t *)result;
    if (raw == NULL) {
        slots[0] = 1;
        slots[1] = (int64_t)(intptr_t)vibe_strdup_or_panic("json.try_parse received null input");
        return result;
    }
    vibe_json_value *value = NULL;
    vibe_json_parse_state state;
    if (vibe_json_parse_document(raw, &value, &state) == NULL) {
        const char *message = "invalid JSON";
        if (state.status == VIBE_JSON_PARSE_TOO_DEEP) {
            message = "JSON nesting exceeds the maximum depth of "
                      VIBE_JSON_DEPTH_STR(VIBE_JSON_MAX_DEPTH);
        } else if (state.status == VIBE_JSON_PARSE_TRAILING) {
            message = "invalid trailing content";
        }
        slots[0] = 1;
        slots[1] = (int64_t)(intptr_t)vibe_strdup_or_panic(message);
        return result;
    }
    slots[0] = 0;
    slots[1] = (int64_t)(intptr_t)value;
    return result;
}

/*
 * Ok payload of a json.try_parse result, or a fresh JSON null node when it was
 * Err. TOTAL: the error branch of a request handler must not itself be able to
 * end the process, so this never panics and never returns NULL.
 * (std.result.unwrap_or only supports Result<Int, _>, so a Json result needs
 * its own accessor.)
 */
void *vibe_json_result_value(void *result) {
    if (result == NULL) {
        return vibe_json_new_value(VIBE_JSON_NULL);
    }
    int64_t *slots = (int64_t *)result;
    if (slots[0] != 0) {
        return vibe_json_new_value(VIBE_JSON_NULL);
    }
    void *value = (void *)(intptr_t)slots[1];
    return value == NULL ? vibe_json_new_value(VIBE_JSON_NULL) : value;
}

/*
 * Err message of a json.try_parse result, or "" when it was Ok. TOTAL, for the
 * same reason as vibe_json_result_value.
 */
char *vibe_json_result_error(void *result) {
    if (result == NULL) {
        return vibe_strdup_or_panic("");
    }
    int64_t *slots = (int64_t *)result;
    if (slots[0] == 0) {
        return vibe_strdup_or_panic("");
    }
    const char *message = (const char *)(intptr_t)slots[1];
    return vibe_strdup_or_panic(message == NULL ? "" : message);
}

/*
 * Genuine validation. This used to compare the first and last non-space
 * characters and nothing else, so json.is_valid("[x]") and
 * json.is_valid("[1,2,]") both answered true -- which meant the
 * guard-then-parse idiom the book recommends still aborted the process on the
 * json.parse that followed. Verified live before this change: the guarded repro
 * server died with SIGABRT (exit 134) on a 3-byte body of "[x]".
 *
 * It now runs the real grammar through the shared core in validate-only mode,
 * so it agrees with json.parse on grammar, on trailing content and on the depth
 * cap. The guarantee callers can rely on: if json.is_valid(s) is true then
 * json.parse(s) returns rather than aborts.
 *
 * Cost: it walks the whole document instead of reading two characters, which is
 * the price of the answer being true. It does NOT build the document -- see the
 * MATERIALIZING vs VALIDATING note above -- so a server that validates and
 * forwards without parsing keeps its old allocation profile and a peer cannot
 * use a large body to multiply the server's heap.
 */
int64_t vibe_json_is_valid(const char *raw) {
    vibe_counter_inc(&vibe_json_validate_calls);
    if (raw == NULL) {
        return 0;
    }
    vibe_json_parse_state state;
    return vibe_json_parse_document(raw, NULL, &state) != NULL ? 1 : 0;
}

char *vibe_json_stringify(void *raw) {
    vibe_counter_inc(&vibe_json_stringify_calls);
    if (raw == NULL) {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic("");
    }
    vibe_string_builder builder;
    vibe_builder_init(&builder, 128);
    vibe_json_stringify_into(&builder, (const vibe_json_value *)raw, 0, 2, 0);
    vibe_counter_inc(&vibe_json_allocations);
    return builder.data;
}

char *vibe_json_stringify_pretty(void *raw) {
    if (raw == NULL) {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic("");
    }
    vibe_string_builder builder;
    vibe_builder_init(&builder, 128);
    vibe_json_stringify_into(&builder, (const vibe_json_value *)raw, 1, 2, 0);
    vibe_counter_inc(&vibe_json_allocations);
    return builder.data;
}

static void vibe_json_builder_push(vibe_json_builder_handle *builder, int64_t kind) {
    if (builder->depth >= builder->stack_cap) {
        int64_t next_cap = builder->stack_cap <= 0 ? 8 : builder->stack_cap * 2;
        vibe_json_builder_ctx *next_stack = (vibe_json_builder_ctx *)realloc(
            builder->stack,
            (size_t)next_cap * sizeof(vibe_json_builder_ctx)
        );
        if (next_stack == NULL) {
            vibe_panic("failed to grow json builder stack");
        }
        builder->stack = next_stack;
        builder->stack_cap = next_cap;
    }
    builder->stack[builder->depth].kind = kind;
    builder->stack[builder->depth].first = 1;
    builder->stack[builder->depth].expecting_value = 0;
    builder->depth += 1;
}

static void vibe_json_builder_begin_value(vibe_json_builder_handle *builder) {
    if (builder->depth == 0) {
        if (builder->root_written) {
            vibe_panic("json.builder cannot write multiple root values");
        }
        builder->root_written = 1;
        return;
    }
    vibe_json_builder_ctx *ctx = &builder->stack[builder->depth - 1];
    if (ctx->kind == VIBE_JSON_OBJECT) {
        if (!ctx->expecting_value) {
            vibe_panic("json.builder expected object key before value");
        }
        ctx->expecting_value = 0;
        return;
    }
    if (!ctx->first) {
        vibe_builder_append_bytes(&builder->out, ",", 1);
    }
    ctx->first = 0;
}

void *vibe_json_builder_new(int64_t capacity) {
    vibe_json_builder_handle *builder =
        (vibe_json_builder_handle *)calloc(1, sizeof(vibe_json_builder_handle));
    if (builder == NULL) {
        vibe_panic("failed to allocate json builder");
    }
    vibe_builder_init(&builder->out, capacity <= 0 ? 128 : (size_t)capacity);
    return builder;
}

void *vibe_json_builder_begin_object(void *handle) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL) {
        vibe_panic("json.builder handle is null");
    }
    vibe_json_builder_begin_value(builder);
    vibe_builder_append_bytes(&builder->out, "{", 1);
    vibe_json_builder_push(builder, VIBE_JSON_OBJECT);
    return builder;
}

void *vibe_json_builder_end_object(void *handle) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL || builder->depth <= 0) {
        vibe_panic("json.builder end_object without object context");
    }
    vibe_json_builder_ctx *ctx = &builder->stack[builder->depth - 1];
    if (ctx->kind != VIBE_JSON_OBJECT || ctx->expecting_value) {
        vibe_panic("json.builder invalid end_object state");
    }
    builder->depth -= 1;
    vibe_builder_append_bytes(&builder->out, "}", 1);
    return builder;
}

void *vibe_json_builder_begin_array(void *handle) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL) {
        vibe_panic("json.builder handle is null");
    }
    vibe_json_builder_begin_value(builder);
    vibe_builder_append_bytes(&builder->out, "[", 1);
    vibe_json_builder_push(builder, VIBE_JSON_ARRAY);
    return builder;
}

void *vibe_json_builder_end_array(void *handle) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL || builder->depth <= 0) {
        vibe_panic("json.builder end_array without array context");
    }
    vibe_json_builder_ctx *ctx = &builder->stack[builder->depth - 1];
    if (ctx->kind != VIBE_JSON_ARRAY) {
        vibe_panic("json.builder invalid end_array state");
    }
    builder->depth -= 1;
    vibe_builder_append_bytes(&builder->out, "]", 1);
    return builder;
}

void *vibe_json_builder_key(void *handle, const char *name) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL || builder->depth <= 0) {
        vibe_panic("json.builder key without object context");
    }
    vibe_json_builder_ctx *ctx = &builder->stack[builder->depth - 1];
    if (ctx->kind != VIBE_JSON_OBJECT || ctx->expecting_value) {
        vibe_panic("json.builder invalid key state");
    }
    if (!ctx->first) {
        vibe_builder_append_bytes(&builder->out, ",", 1);
    }
    ctx->first = 0;
    char *quoted = vibe_json_quote_string(name == NULL ? "" : name);
    vibe_builder_append_bytes(&builder->out, quoted, strlen(quoted));
    free(quoted);
    vibe_builder_append_bytes(&builder->out, ":", 1);
    ctx->expecting_value = 1;
    return builder;
}

void *vibe_json_builder_value_null(void *handle) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL) {
        vibe_panic("json.builder handle is null");
    }
    vibe_json_builder_begin_value(builder);
    vibe_builder_append_bytes(&builder->out, "null", 4);
    return builder;
}

void *vibe_json_builder_value_bool(void *handle, int64_t value) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL) {
        vibe_panic("json.builder handle is null");
    }
    vibe_json_builder_begin_value(builder);
    vibe_builder_append_bytes(&builder->out, value != 0 ? "true" : "false", value != 0 ? 4 : 5);
    return builder;
}

void *vibe_json_builder_value_i64(void *handle, int64_t value) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL) {
        vibe_panic("json.builder handle is null");
    }
    vibe_json_builder_begin_value(builder);
    char *encoded = vibe_json_stringify_i64(value);
    vibe_builder_append_bytes(&builder->out, encoded, strlen(encoded));
    free(encoded);
    return builder;
}

void *vibe_json_builder_value_f64(void *handle, double value) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL) {
        vibe_panic("json.builder handle is null");
    }
    vibe_json_builder_begin_value(builder);
    char out_num[64];
    vibe_format_shortest_f64(value, out_num);
    vibe_builder_append_bytes(&builder->out, out_num, strlen(out_num));
    return builder;
}

void *vibe_json_builder_value_str(void *handle, const char *value) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL) {
        vibe_panic("json.builder handle is null");
    }
    vibe_json_builder_begin_value(builder);
    char *quoted = vibe_json_quote_string(value == NULL ? "" : value);
    vibe_builder_append_bytes(&builder->out, quoted, strlen(quoted));
    free(quoted);
    return builder;
}

void *vibe_json_builder_value_json(void *handle, void *value) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL) {
        vibe_panic("json.builder handle is null");
    }
    vibe_json_builder_begin_value(builder);
    char *encoded = vibe_json_stringify(value);
    vibe_builder_append_bytes(&builder->out, encoded, strlen(encoded));
    free(encoded);
    return builder;
}

char *vibe_json_builder_finish(void *handle) {
    vibe_json_builder_handle *builder = (vibe_json_builder_handle *)handle;
    if (builder == NULL) {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic("");
    }
    if (builder->depth != 0) {
        vibe_panic("json.builder finish with unclosed object/array");
    }
    char *out = builder->out.data;
    free(builder->stack);
    free(builder);
    vibe_counter_inc(&vibe_json_allocations);
    return out;
}

static const char *vibe_json_skip_ws(const char *p) {
    while (p != NULL && *p != '\0' && isspace((unsigned char)*p)) {
        p += 1;
    }
    return p;
}

static int64_t vibe_json_find_key_value(
    const char *raw,
    const char *key,
    const char **out_start,
    const char **out_end
) {
    if (raw == NULL || key == NULL || key[0] == '\0' || out_start == NULL || out_end == NULL) {
        return 0;
    }
    size_t key_len = strlen(key);
    size_t pattern_len = key_len + 2;
    char *pattern = (char *)calloc(pattern_len + 1, sizeof(char));
    if (pattern == NULL) {
        vibe_panic("failed to allocate json key pattern");
    }
    pattern[0] = '"';
    memcpy(pattern + 1, key, key_len);
    pattern[key_len + 1] = '"';
    pattern[pattern_len] = '\0';

    const char *cursor = raw;
    while (cursor != NULL && *cursor != '\0') {
        const char *match = strstr(cursor, pattern);
        if (match == NULL) {
            break;
        }
        const char *after_key = vibe_json_skip_ws(match + pattern_len);
        if (after_key != NULL && *after_key == ':') {
            const char *value_start = vibe_json_skip_ws(after_key + 1);
            if (value_start == NULL || *value_start == '\0') {
                break;
            }
            const char *value_end = value_start;
            if (*value_start == '"') {
                int escaped = 0;
                value_end = value_start + 1;
                while (*value_end != '\0') {
                    if (escaped) {
                        escaped = 0;
                    } else if (*value_end == '\\') {
                        escaped = 1;
                    } else if (*value_end == '"') {
                        value_end += 1;
                        break;
                    }
                    value_end += 1;
                }
            } else if (*value_start == '{' || *value_start == '[') {
                char open = *value_start;
                char close = (open == '{') ? '}' : ']';
                int depth = 1;
                int in_str = 0;
                int esc = 0;
                value_end = value_start + 1;
                while (*value_end != '\0' && depth > 0) {
                    if (esc) { esc = 0; }
                    else if (*value_end == '\\') { esc = 1; }
                    else if (*value_end == '"') { in_str = !in_str; }
                    else if (!in_str && *value_end == open) { depth++; }
                    else if (!in_str && *value_end == close) { depth--; }
                    value_end += 1;
                }
            } else {
                while (*value_end != '\0' && *value_end != ',' && *value_end != '}' && *value_end != ']') {
                    value_end += 1;
                }
                while (value_end > value_start && isspace((unsigned char)*(value_end - 1))) {
                    value_end -= 1;
                }
            }
            *out_start = value_start;
            *out_end = value_end;
            free(pattern);
            return 1;
        }
        cursor = match + 1;
    }

    free(pattern);
    return 0;
}

static char *vibe_json_unquote_string(const char *start, const char *end) {
    if (start == NULL || end == NULL || end <= start || *start != '"') {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic("");
    }
    vibe_string_builder builder;
    vibe_builder_init(&builder, (size_t)(end - start) + 1);
    const char *p = start + 1;
    const char *limit = end - 1;
    while (p < limit) {
        char ch = *p;
        if (ch == '\\' && (p + 1) < limit) {
            char next = *(p + 1);
            switch (next) {
                case 'n':
                    ch = '\n';
                    p += 2;
                    break;
                case 'r':
                    ch = '\r';
                    p += 2;
                    break;
                case 't':
                    ch = '\t';
                    p += 2;
                    break;
                case '\\':
                case '"':
                case '/':
                    ch = next;
                    p += 2;
                    break;
                default:
                    // Keep unknown escape payload as-is for deterministic fallback.
                    p += 1;
                    break;
            }
        } else {
            p += 1;
        }
        vibe_builder_append_bytes(&builder, &ch, 1);
    }
    vibe_counter_inc(&vibe_json_allocations);
    return builder.data;
}

char *vibe_json_stringify_i64(int64_t value);

int64_t vibe_json_get_i64(const char *raw, const char *key, int64_t fallback) {
    const char *start = NULL;
    const char *end = NULL;
    if (!vibe_json_find_key_value(raw, key, &start, &end) || start == NULL || end == NULL) {
        return fallback;
    }
    const char *parse_start = start;
    const char *parse_end = end;
    if (*start == '"' && end > start + 1 && *(end - 1) == '"') {
        parse_start = start + 1;
        parse_end = end - 1;
    }
    int64_t out = 0;
    if (!vibe_parse_i64_strict(parse_start, parse_end, &out)) {
        return fallback;
    }
    return out;
}

int64_t vibe_json_get_bool(const char *raw, const char *key, int64_t fallback) {
    const char *start = NULL;
    const char *end = NULL;
    if (!vibe_json_find_key_value(raw, key, &start, &end) || start == NULL || end == NULL) {
        return fallback ? 1 : 0;
    }
    size_t len = (size_t)(end - start);
    if (len == 4 && strncmp(start, "true", 4) == 0) {
        return 1;
    }
    if (len == 5 && strncmp(start, "false", 5) == 0) {
        return 0;
    }
    return fallback ? 1 : 0;
}

char *vibe_json_get_str(const char *raw, const char *key, const char *fallback) {
    const char *start = NULL;
    const char *end = NULL;
    if (!vibe_json_find_key_value(raw, key, &start, &end) || start == NULL || end == NULL) {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic(fallback == NULL ? "" : fallback);
    }
    if (*start == '"' && end > start + 1 && *(end - 1) == '"') {
        return vibe_json_unquote_string(start, end);
    }
    size_t len = (size_t)(end - start);
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate json string field");
    }
    memcpy(out, start, len);
    out[len] = '\0';
    vibe_counter_inc(&vibe_json_allocations);
    return out;
}

static const char *vibe_json_schema_skip_type(const char *p) {
    if (*p == '{') {
        int depth = 1;
        p++;
        while (*p != '\0' && depth > 0) {
            if (*p == '{') depth++;
            else if (*p == '}') depth--;
            p++;
        }
        return p;
    }
    while (*p != '\0' && *p != ';') {
        p++;
    }
    return p;
}

static void vibe_json_encode_record_into(vibe_string_builder *builder, const int64_t *slots, const char *schema) {
    vibe_builder_append_bytes(builder, "{", 1);
    const char *cursor = schema;
    int first = 1;
    int64_t slot_index = 0;
    while (*cursor != '\0') {
        const char *entry_start = cursor;
        const char *colon = NULL;
        for (const char *s = entry_start; *s != '\0'; s++) {
            if (*s == ':') { colon = s; break; }
        }
        if (colon == NULL || colon <= entry_start) {
            break;
        }
        const char *key_start = entry_start;
        size_t key_len = (size_t)(colon - key_start);
        const char *type_start = colon + 1;
        const char *type_end = vibe_json_schema_skip_type(type_start);
        size_t type_len = (size_t)(type_end - type_start);
        if (!first) {
            vibe_builder_append_bytes(builder, ",", 1);
        }
        first = 0;
        vibe_builder_append_bytes(builder, "\"", 1);
        vibe_builder_append_bytes(builder, key_start, key_len);
        vibe_builder_append_bytes(builder, "\":", 2);
        int64_t slot_value = slots[slot_index];
        if (*type_start == '{') {
            const char *inner_schema_start = type_start + 1;
            size_t inner_len = type_len >= 2 ? type_len - 2 : 0;
            char *inner_schema = (char *)malloc(inner_len + 1);
            if (inner_schema == NULL) { vibe_panic("alloc failed in json encode"); }
            memcpy(inner_schema, inner_schema_start, inner_len);
            inner_schema[inner_len] = '\0';
            void *nested = (void *)(intptr_t)slot_value;
            if (nested == NULL) {
                vibe_builder_append_bytes(builder, "null", 4);
            } else {
                vibe_json_encode_record_into(builder, (const int64_t *)nested, inner_schema);
            }
            free(inner_schema);
        } else if (type_len == 3 && strncmp(type_start, "Int", 3) == 0) {
            char *encoded = vibe_json_stringify_i64(slot_value);
            vibe_builder_append_bytes(builder, encoded, strlen(encoded));
            free(encoded);
        } else if (type_len == 4 && strncmp(type_start, "Bool", 4) == 0) {
            const char *bval = slot_value != 0 ? "true" : "false";
            vibe_builder_append_bytes(builder, bval, strlen(bval));
        } else if (type_len == 3 && strncmp(type_start, "Str", 3) == 0) {
            char *encoded = vibe_json_quote_string((const char *)(intptr_t)slot_value);
            vibe_builder_append_bytes(builder, encoded, strlen(encoded));
            free(encoded);
        } else if (type_len == 4 && strncmp(type_start, "Json", 4) == 0) {
            void *json_handle = (void *)(intptr_t)slot_value;
            if (json_handle == NULL) {
                vibe_builder_append_bytes(builder, "null", 4);
        } else {
                char *encoded = vibe_json_stringify(json_handle);
                if (encoded == NULL || encoded[0] == '\0') {
                    vibe_builder_append_bytes(builder, "null", 4);
                } else {
                    vibe_builder_append_bytes(builder, encoded, strlen(encoded));
                }
                if (encoded != NULL) free(encoded);
            }
        } else {
            vibe_builder_append_bytes(builder, "null", 4);
        }
        slot_index += 1;
        cursor = type_end;
        if (*cursor == ';') cursor++;
    }
    vibe_builder_append_bytes(builder, "}", 1);
}

char *vibe_json_encode_record(void *record, const char *schema) {
    if (record == NULL || schema == NULL || schema[0] == '\0') {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic("{}");
    }
    vibe_string_builder builder;
    vibe_builder_init(&builder, strlen(schema) + 64);
    vibe_json_encode_record_into(&builder, (const int64_t *)record, schema);
    vibe_counter_inc(&vibe_json_allocations);
    return builder.data;
}

static int64_t vibe_json_schema_count_fields(const char *schema) {
    if (schema == NULL || *schema == '\0') return 0;
    int64_t count = 0;
    const char *cursor = schema;
    while (*cursor != '\0') {
        const char *colon = NULL;
        for (const char *s = cursor; *s != '\0'; s++) {
            if (*s == ':') { colon = s; break; }
        }
        if (colon == NULL) break;
        const char *type_start = colon + 1;
        const char *type_end = vibe_json_schema_skip_type(type_start);
        count++;
        cursor = type_end;
        if (*cursor == ';') cursor++;
    }
    return count;
}

void *vibe_json_decode_record(const char *raw, const char *schema, void *fallback, void *out_record) {
    if (schema == NULL || out_record == NULL) {
        return out_record;
    }
    const int64_t *fallback_slots = (const int64_t *)fallback;
    int64_t *out_slots = (int64_t *)out_record;
    const char *cursor = schema;
    int64_t slot_index = 0;
    while (*cursor != '\0') {
        const char *colon = NULL;
        for (const char *s = cursor; *s != '\0'; s++) {
            if (*s == ':') { colon = s; break; }
        }
        if (colon == NULL || colon <= cursor) break;
        const char *key_start = cursor;
        size_t key_len = (size_t)(colon - key_start);
        const char *type_start = colon + 1;
        const char *type_end = vibe_json_schema_skip_type(type_start);
        size_t type_len = (size_t)(type_end - type_start);

        char *key = (char *)calloc(key_len + 1, sizeof(char));
        if (key == NULL) { vibe_panic("failed to allocate json codec key"); }
        memcpy(key, key_start, key_len);
        key[key_len] = '\0';

        int64_t fallback_value = fallback_slots == NULL ? 0 : fallback_slots[slot_index];

        if (*type_start == '{') {
            const char *inner_schema_start = type_start + 1;
            size_t inner_len = type_len >= 2 ? type_len - 2 : 0;
            char *inner_schema = (char *)malloc(inner_len + 1);
            if (inner_schema == NULL) { vibe_panic("alloc failed in json decode"); }
            memcpy(inner_schema, inner_schema_start, inner_len);
            inner_schema[inner_len] = '\0';

            const char *val_start = NULL;
            const char *val_end = NULL;
            if (vibe_json_find_key_value(raw, key, &val_start, &val_end) && val_start != NULL && *val_start == '{') {
                size_t obj_len = (size_t)(val_end - val_start);
                char *sub_json = (char *)calloc(obj_len + 1, sizeof(char));
                if (sub_json == NULL) { vibe_panic("alloc failed in json decode nested"); }
                memcpy(sub_json, val_start, obj_len);
                sub_json[obj_len] = '\0';

                int64_t nested_slot_count = vibe_json_schema_count_fields(inner_schema);
                void *nested_record = vibe_record_alloc(nested_slot_count > 0 ? nested_slot_count : 1);
                void *nested_fallback = (void *)(intptr_t)fallback_value;
                if (nested_fallback == NULL) {
                    nested_fallback = vibe_record_alloc(nested_slot_count > 0 ? nested_slot_count : 1);
                }
                vibe_json_decode_record(sub_json, inner_schema, nested_fallback, nested_record);
                out_slots[slot_index] = (int64_t)(intptr_t)nested_record;
                free(sub_json);
            } else {
                out_slots[slot_index] = fallback_value;
            }
            free(inner_schema);
        } else if (type_len == 3 && strncmp(type_start, "Int", 3) == 0) {
            out_slots[slot_index] = vibe_json_get_i64(raw, key, fallback_value);
        } else if (type_len == 4 && strncmp(type_start, "Bool", 4) == 0) {
            out_slots[slot_index] = vibe_json_get_bool(raw, key, fallback_value != 0);
        } else if (type_len == 3 && strncmp(type_start, "Str", 3) == 0) {
            const char *fallback_str = (const char *)(intptr_t)fallback_value;
            char *decoded = vibe_json_get_str(raw, key, fallback_str == NULL ? "" : fallback_str);
            out_slots[slot_index] = (int64_t)(intptr_t)decoded;
        } else {
            out_slots[slot_index] = fallback_value;
        }
        free(key);
        slot_index += 1;
        cursor = type_end;
        if (*cursor == ';') cursor++;
    }
    return out_record;
}

int64_t vibe_json_parse_i64(const char *raw) {
    vibe_counter_inc(&vibe_json_parse_calls);
    if (raw == NULL) {
        return 0;
    }
    const char *start = vibe_trim_start(raw);
    const char *end = vibe_trim_end_ptr(start);
    int64_t value = 0;
    if (!vibe_parse_i64_strict(start, end, &value)) {
        return 0;
    }
    return value;
}

char *vibe_json_stringify_i64(int64_t value) {
    vibe_counter_inc(&vibe_json_stringify_calls);
    char buffer[32];
    int pos = (int)(sizeof(buffer) - 1);
    buffer[pos] = '\0';
    uint64_t magnitude;
    int negative = value < 0;
    if (negative) {
        magnitude = (uint64_t)(-(value + 1)) + 1ull;
    } else {
        magnitude = (uint64_t)value;
    }
    do {
        uint64_t digit = magnitude % 10ull;
        buffer[--pos] = (char)('0' + digit);
        magnitude /= 10ull;
    } while (magnitude > 0);
    if (negative) {
        buffer[--pos] = '-';
    }
    vibe_counter_inc(&vibe_json_allocations);
    return vibe_strdup_or_panic(&buffer[pos]);
}

char *vibe_json_minify(const char *raw) {
    vibe_counter_inc(&vibe_json_minify_calls);
    if (raw == NULL) {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic("");
    }
    size_t len = strlen(raw);
    int has_whitespace = 0;
    for (size_t i = 0; i < len; i++) {
        if (isspace((unsigned char)raw[i])) {
            has_whitespace = 1;
            break;
        }
    }
    if (!has_whitespace) {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic(raw);
    }
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate json.minify output");
    }
    vibe_counter_inc(&vibe_json_allocations);
    int in_string = 0;
    int escaped = 0;
    size_t out_idx = 0;
    for (size_t i = 0; i < len; i++) {
        char ch = raw[i];
        if (in_string) {
            out[out_idx++] = ch;
            if (escaped) {
                escaped = 0;
            } else if (ch == '\\') {
                escaped = 1;
            } else if (ch == '"') {
                in_string = 0;
            }
            continue;
        }
        if (ch == '"') {
            in_string = 1;
            out[out_idx++] = ch;
            continue;
        }
        if (isspace((unsigned char)ch)) {
            continue;
        }
        out[out_idx++] = ch;
    }
    out[out_idx] = '\0';
    return out;
}

char *vibe_json_repeat_array(const char *item, int64_t n) {
    if (n <= 0) {
        vibe_counter_inc(&vibe_json_allocations);
        return vibe_strdup_or_panic("[]");
    }
    const char *value = item == NULL ? "" : item;
    size_t item_len = strlen(value);
    size_t approx = 2 + (size_t)n * item_len + (size_t)(n - 1);
    vibe_string_builder builder;
    vibe_builder_init(&builder, approx + 16);
    vibe_builder_append_bytes(&builder, "[", 1);
    for (int64_t i = 0; i < n; i++) {
        if (i > 0) {
            vibe_builder_append_bytes(&builder, ",", 1);
        }
        vibe_builder_append_bytes(&builder, value, item_len);
    }
    vibe_builder_append_bytes(&builder, "]", 1);
    vibe_counter_inc(&vibe_json_allocations);
    return builder.data;
}

int64_t vibe_json_parse_call_count(void) {
    return vibe_json_parse_calls;
}

int64_t vibe_json_stringify_call_count(void) {
    return vibe_json_stringify_calls;
}

int64_t vibe_json_minify_call_count(void) {
    return vibe_json_minify_calls;
}

int64_t vibe_json_validate_call_count(void) {
    return vibe_json_validate_calls;
}

int64_t vibe_json_allocation_count(void) {
    return vibe_json_allocations;
}

char *vibe_http_status_text(int64_t code) {
    switch (code) {
        case 200:
            return vibe_strdup_or_panic("OK");
        case 201:
            return vibe_strdup_or_panic("Created");
        case 204:
            return vibe_strdup_or_panic("No Content");
        case 400:
            return vibe_strdup_or_panic("Bad Request");
        case 401:
            return vibe_strdup_or_panic("Unauthorized");
        case 403:
            return vibe_strdup_or_panic("Forbidden");
        case 404:
            return vibe_strdup_or_panic("Not Found");
        case 500:
            return vibe_strdup_or_panic("Internal Server Error");
        default:
            return vibe_strdup_or_panic("Unknown");
    }
}

int64_t vibe_http_default_port(const char *scheme) {
    if (scheme == NULL) {
        return 80;
    }
    if (strcasecmp(scheme, "https") == 0 || strcasecmp(scheme, "wss") == 0) {
        return 443;
    }
    return 80;
}

char *vibe_http_build_request_line(const char *method, const char *path) {
    const char *verb = (method == NULL || method[0] == '\0') ? "GET" : method;
    const char *target = (path == NULL || path[0] == '\0') ? "/" : path;
    size_t len = strlen(verb) + 1 + strlen(target) + strlen(" HTTP/1.1");
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate http request line");
    }
    snprintf(out, len + 1, "%s %s HTTP/1.1", verb, target);
    return out;
}

static char *vibe_http_server_dup_range(const char *start, size_t len) {
    if (start == NULL || len == 0) {
        return vibe_strdup_or_panic("");
    }
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("vibe_http_server_dup_range: allocation failed");
    }
    memcpy(out, start, len);
    out[len] = '\0';
    return out;
}

void *vibe_http_server_parse_request(const char *raw) {
    void *record = vibe_record_alloc(5);
    int64_t *slots = (int64_t *)record;
    if (raw == NULL || raw[0] == '\0') {
        slots[0] = (int64_t)(intptr_t)vibe_strdup_or_panic("");
        slots[1] = (int64_t)(intptr_t)vibe_strdup_or_panic("");
        slots[2] = (int64_t)(intptr_t)vibe_strdup_or_panic("");
        slots[3] = (int64_t)(intptr_t)vibe_strdup_or_panic("");
        slots[4] = (int64_t)(intptr_t)vibe_strdup_or_panic("");
        return record;
    }

    const char *r = raw;
    const char *line_end = strstr(r, "\r\n");
    size_t line_term = 2;
    if (line_end == NULL) {
        line_end = strchr(r, '\n');
        line_term = line_end != NULL ? 1 : 0;
    }

    size_t line1_len;
    if (line_end != NULL) {
        line1_len = (size_t)(line_end - r);
    } else {
        line1_len = strlen(r);
        line_end = r + line1_len;
        line_term = 0;
    }

    const char *l1 = r;
    const char *l1_end = r + line1_len;
    while (l1 < l1_end && isspace((unsigned char)*l1)) {
        l1++;
    }

    const char *m_start = l1;
    const char *m_end = m_start;
    while (m_end < l1_end && !isspace((unsigned char)*m_end)) {
        m_end++;
    }

    const char *u_start = m_end;
    while (u_start < l1_end && isspace((unsigned char)*u_start)) {
        u_start++;
    }
    const char *u_end = u_start;
    while (u_end < l1_end && !isspace((unsigned char)*u_end)) {
        u_end++;
    }

    char *method = vibe_http_server_dup_range(m_start, (size_t)(m_end - m_start));

    char *path = vibe_strdup_or_panic("");
    char *query = vibe_strdup_or_panic("");
    size_t uri_len = (size_t)(u_end - u_start);
    if (uri_len > 0) {
        char *uri = vibe_http_server_dup_range(u_start, uri_len);
        char *qm = strchr(uri, '?');
        if (qm != NULL) {
            *qm = '\0';
            free(path);
            path = vibe_strdup_or_panic(uri);
            char *q_src = qm + 1;
            char *hash = strchr(q_src, '#');
            size_t q_len = hash != NULL ? (size_t)(hash - q_src) : strlen(q_src);
            free(query);
            query = vibe_http_server_dup_range(q_src, q_len);
        } else {
            char *hash = strchr(uri, '#');
            if (hash != NULL) {
                *hash = '\0';
            }
            free(path);
            path = uri;
            uri = NULL;
        }
        if (uri != NULL) {
            free(uri);
        }
    }

    const char *hdr_start = line_term > 0 ? line_end + line_term : line_end;
    char *headers = vibe_strdup_or_panic("");
    char *body = vibe_strdup_or_panic("");

    const char *body_sep = strstr(hdr_start, "\r\n\r\n");
    size_t body_skip = 4;
    if (body_sep == NULL) {
        body_sep = strstr(hdr_start, "\n\n");
        body_skip = 2;
    }

    if (body_sep != NULL) {
        free(headers);
        headers = vibe_http_server_dup_range(hdr_start, (size_t)(body_sep - hdr_start));
        body = vibe_strdup_or_panic(body_sep + body_skip);
    } else if (*hdr_start != '\0') {
        free(headers);
        headers = vibe_strdup_or_panic(hdr_start);
    }

    slots[0] = (int64_t)(intptr_t)method;
    slots[1] = (int64_t)(intptr_t)path;
    slots[2] = (int64_t)(intptr_t)query;
    slots[3] = (int64_t)(intptr_t)headers;
    slots[4] = (int64_t)(intptr_t)body;
    return record;
}

char *vibe_http_server_format_response(int64_t status, const char *headers, const char *body) {
    const char *b = body == NULL ? "" : body;
    const char *hdrs = headers == NULL ? "" : headers;
    char *reason = vibe_http_status_text(status);
    size_t body_len = strlen(b);
    char cl_buf[32];
    snprintf(cl_buf, sizeof(cl_buf), "%zu", body_len);

    vibe_string_builder sb;
    vibe_builder_init(&sb, 256 + strlen(reason) + strlen(hdrs) + body_len);
    vibe_builder_append_bytes(&sb, "HTTP/1.1 ", 9);
    char sc_buf[16];
    int sc_len = snprintf(sc_buf, sizeof(sc_buf), "%lld", (long long)status);
    vibe_builder_append_bytes(&sb, sc_buf, sc_len);
    vibe_builder_append_bytes(&sb, " ", 1);
    vibe_builder_append_bytes(&sb, reason, strlen(reason));
    vibe_builder_append_bytes(&sb, "\r\nContent-Length: ", 18);
    vibe_builder_append_bytes(&sb, cl_buf, strlen(cl_buf));
    vibe_builder_append_bytes(&sb, "\r\n", 2);
    if (hdrs[0] != '\0') {
        vibe_builder_append_bytes(&sb, hdrs, strlen(hdrs));
        size_t hlen = strlen(hdrs);
        if (hlen < 2 || hdrs[hlen - 2] != '\r' || hdrs[hlen - 1] != '\n') {
            vibe_builder_append_bytes(&sb, "\r\n", 2);
        }
    }
    vibe_builder_append_bytes(&sb, "Connection: close\r\n\r\n", 21);
    vibe_builder_append_bytes(&sb, b, body_len);
    free(reason);
    return sb.data;
}

char *vibe_http_server_cors_headers(void) {
    static const char cors[] =
        "Access-Control-Allow-Origin: *\r\n"
        "Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n"
        "Access-Control-Allow-Headers: Content-Type, Authorization\r\n";
    return vibe_strdup_or_panic(cors);
}

char *vibe_http_build_response(int64_t status, const char *body) {
    const char *b = (body == NULL) ? "" : body;
    const char *status_text = "Unknown";
    switch (status) {
        case 200: status_text = "OK"; break;
        case 201: status_text = "Created"; break;
        case 204: status_text = "No Content"; break;
        case 400: status_text = "Bad Request"; break;
        case 401: status_text = "Unauthorized"; break;
        case 403: status_text = "Forbidden"; break;
        case 404: status_text = "Not Found"; break;
        case 405: status_text = "Method Not Allowed"; break;
        case 422: status_text = "Unprocessable Entity"; break;
        case 500: status_text = "Internal Server Error"; break;
    }
    size_t body_len = strlen(b);
    char cl_buf[32];
    snprintf(cl_buf, sizeof(cl_buf), "%zu", body_len);
    vibe_string_builder sb;
    vibe_builder_init(&sb, 256 + body_len);
    vibe_builder_append_bytes(&sb, "HTTP/1.1 ", 9);
    char sc_buf[16];
    int sc_len = snprintf(sc_buf, sizeof(sc_buf), "%lld", (long long)status);
    vibe_builder_append_bytes(&sb, sc_buf, sc_len);
    vibe_builder_append_bytes(&sb, " ", 1);
    vibe_builder_append_bytes(&sb, status_text, strlen(status_text));
    vibe_builder_append_bytes(&sb, "\r\nContent-Type: application/json\r\nContent-Length: ", 50);
    vibe_builder_append_bytes(&sb, cl_buf, strlen(cl_buf));
    vibe_builder_append_bytes(&sb, "\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nConnection: close\r\n\r\n", 155);
    vibe_builder_append_bytes(&sb, b, body_len);
    return sb.data;
}

char *vibe_http_format_response(void *record) {
    if (record == NULL) {
        return vibe_http_build_response(500, "");
    }
    int64_t *slots = (int64_t *)record;
    int64_t status = slots[0];
    const char *body = (const char *)(intptr_t)slots[2];
    return vibe_http_build_response(status, body);
}

static const char *vibe_http_find_body(const char *req);

static char *vibe_shell_single_quote(const char *raw) {
    const char *text = raw == NULL ? "" : raw;
    vibe_string_builder builder;
    vibe_builder_init(&builder, strlen(text) + 8);
    vibe_builder_append_bytes(&builder, "'", 1);
    for (const char *p = text; *p != '\0'; p++) {
        if (*p == '\'') {
            vibe_builder_append_bytes(&builder, "'\\''", 4);
        } else {
            vibe_builder_append_bytes(&builder, p, 1);
        }
    }
    vibe_builder_append_bytes(&builder, "'", 1);
    return builder.data;
}

static char *vibe_exec_capture_stdout(const char *cmd) {
    if (cmd == NULL || cmd[0] == '\0') {
        return vibe_strdup_or_panic("");
    }
    FILE *pipe = popen(cmd, "r");
    if (pipe == NULL) {
        return vibe_strdup_or_panic("");
    }
    vibe_string_builder builder;
    vibe_builder_init(&builder, 1024);
    char chunk[2048];
    while (1) {
        size_t n = fread(chunk, 1, sizeof(chunk), pipe);
        if (n > 0) {
            vibe_builder_append_bytes(&builder, chunk, n);
        }
        if (n < sizeof(chunk)) {
            if (feof(pipe) != 0 || ferror(pipe) != 0) {
                break;
            }
        }
    }
    (void)pclose(pipe);
    return builder.data;
}

static int64_t vibe_http_parse_url(
    const char *url,
    char *host_out,
    size_t host_cap,
    int64_t *port_out,
    const char **path_out,
    int *is_https_out
) {
    if (url == NULL || host_out == NULL || host_cap == 0 || port_out == NULL || path_out == NULL ||
        is_https_out == NULL) {
        return 0;
    }
    const char *rest = NULL;
    int is_https = 0;
    if (strncmp(url, "http://", 7) == 0) {
        rest = url + 7;
        is_https = 0;
    } else if (strncmp(url, "https://", 8) == 0) {
        rest = url + 8;
        is_https = 1;
    } else {
        return 0;
    }
    const char *path = strchr(rest, '/');
    const char *host_end = path == NULL ? (rest + strlen(rest)) : path;
    if (host_end <= rest) {
        return 0;
    }
    size_t host_port_len = (size_t)(host_end - rest);
    if (host_port_len >= host_cap) {
        return 0;
    }
    char host_port[512];
    memset(host_port, 0, sizeof(host_port));
    memcpy(host_port, rest, host_port_len);
    host_port[host_port_len] = '\0';

    int64_t port = is_https ? 443 : 80;
    char *colon = strrchr(host_port, ':');
    if (colon != NULL && *(colon + 1) != '\0') {
        char *end = NULL;
        long parsed = strtol(colon + 1, &end, 10);
        if (end != NULL && *end == '\0' && parsed > 0 && parsed <= 65535) {
            port = (int64_t)parsed;
            *colon = '\0';
        }
    }
    if (host_port[0] == '\0') {
        return 0;
    }
    snprintf(host_out, host_cap, "%s", host_port);
    *port_out = port;
    *path_out = path == NULL ? "/" : path;
    *is_https_out = is_https;
    return 1;
}

static void vibe_http_apply_timeout(int fd, int64_t timeout_ms) {
    if (fd <= 0 || timeout_ms <= 0) {
        return;
    }
#ifdef _WIN32
    (void)fd;
    (void)timeout_ms;
#else
    struct timeval tv;
    tv.tv_sec = (time_t)(timeout_ms / 1000);
    tv.tv_usec = (suseconds_t)((timeout_ms % 1000) * 1000);
    (void)setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
    (void)setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv));
#endif
}

static char *vibe_http_read_all_response(int fd, int64_t max_bytes) {
    if (fd <= 0 || max_bytes <= 0) {
        return vibe_strdup_or_panic("");
    }
    if (max_bytes > 4 * 1024 * 1024) {
        max_bytes = 4 * 1024 * 1024;
    }
    vibe_string_builder builder;
    vibe_builder_init(&builder, 4096);
    while ((int64_t)builder.len < max_bytes) {
        char tmp[4096];
        size_t remain = (size_t)(max_bytes - (int64_t)builder.len);
        size_t to_read = remain < sizeof(tmp) ? remain : sizeof(tmp);
        ssize_t n = recv(fd, tmp, to_read, 0);
        if (n <= 0) {
            break;
        }
        vibe_builder_append_bytes(&builder, tmp, (size_t)n);
    }
    return builder.data;
}

static int64_t vibe_http_parse_status_code(const char *raw_response) {
    if (raw_response == NULL) {
        return 0;
    }
    const char *space = strchr(raw_response, ' ');
    if (space == NULL) {
        return 0;
    }
    while (*space == ' ') {
        space += 1;
    }
    if (*space < '0' || *space > '9') {
        return 0;
    }
    char *end = NULL;
    long code = strtol(space, &end, 10);
    if (end == space || code < 0) {
        return 0;
    }
    return (int64_t)code;
}

static void vibe_http_fill_response_record(void *record, int64_t status, const char *headers, const char *body) {
    int64_t *slots = (int64_t *)record;
    slots[0] = status;
    slots[1] = (int64_t)(intptr_t)vibe_strdup_or_panic(headers == NULL ? "" : headers);
    slots[2] = (int64_t)(intptr_t)vibe_strdup_or_panic(body == NULL ? "" : body);
}

static void vibe_http_parse_raw_response(const char *raw, int64_t *out_status, const char **out_headers, const char **out_body) {
    *out_status = 0;
    *out_headers = "";
    *out_body = "";
    if (raw == NULL || raw[0] == '\0') return;
    *out_status = vibe_http_parse_status_code(raw);
    const char *hdr_start = strchr(raw, '\n');
    if (hdr_start != NULL) hdr_start++;
    else hdr_start = raw;
    const char *body_sep = strstr(raw, "\r\n\r\n");
    if (body_sep != NULL) {
        *out_headers = hdr_start;
        *out_body = body_sep + 4;
    } else {
        *out_headers = hdr_start;
    }
}

static void vibe_http_curl_send(
    const char *method,
    const char *url,
    const char *custom_headers,
    const char *body,
    int64_t timeout_ms,
    void *out_record
) {
    const char *verb = (method == NULL || method[0] == '\0') ? "GET" : method;
    const char *payload = body == NULL ? "" : body;
    const char *hdrs = custom_headers == NULL ? "" : custom_headers;
    double timeout_sec = timeout_ms > 0 ? ((double)timeout_ms / 1000.0) : 10.0;
    if (timeout_sec < 0.001) timeout_sec = 0.001;
    char timeout_buf[32];
    snprintf(timeout_buf, sizeof(timeout_buf), "%.3f", timeout_sec);
    char *q_method = vibe_shell_single_quote(verb);
    char *q_url = vibe_shell_single_quote(url == NULL ? "" : url);
    char *q_body = vibe_shell_single_quote(payload);
    vibe_string_builder cmd;
    vibe_builder_init(&cmd, strlen(url == NULL ? "" : url) + strlen(payload) + strlen(hdrs) + 256);
    vibe_builder_append_bytes(&cmd, "curl -sS -L -i --max-time ", 26);
    vibe_builder_append_bytes(&cmd, timeout_buf, strlen(timeout_buf));
    vibe_builder_append_bytes(&cmd, " -X ", 4);
    vibe_builder_append_bytes(&cmd, q_method, strlen(q_method));

    if (hdrs[0] != '\0') {
        const char *p = hdrs;
        while (*p != '\0') {
            const char *line_end = p;
            while (*line_end != '\0' && *line_end != '\r' && *line_end != '\n') line_end++;
            size_t line_len = (size_t)(line_end - p);
            if (line_len > 0) {
                char *hdr_line = (char *)calloc(line_len + 1, 1);
                if (hdr_line) {
                    memcpy(hdr_line, p, line_len);
                    char *q_hdr = vibe_shell_single_quote(hdr_line);
                    vibe_builder_append_bytes(&cmd, " -H ", 4);
                    vibe_builder_append_bytes(&cmd, q_hdr, strlen(q_hdr));
                    free(q_hdr);
                    free(hdr_line);
                }
            }
            p = line_end;
            while (*p == '\r' || *p == '\n') p++;
        }
    }

    if (payload[0] != '\0') {
        vibe_builder_append_bytes(&cmd, " --data-binary ", 15);
        vibe_builder_append_bytes(&cmd, q_body, strlen(q_body));
    }
    vibe_builder_append_bytes(&cmd, " ", 1);
    vibe_builder_append_bytes(&cmd, q_url, strlen(q_url));
    vibe_builder_append_bytes(&cmd, " 2>/dev/null", 12);
    char *raw = vibe_exec_capture_stdout(cmd.data);
    free(q_method);
    free(q_url);
    free(q_body);
    free(cmd.data);

    int64_t status = 0;
    const char *resp_hdrs = "";
    const char *resp_body = "";
    vibe_http_parse_raw_response(raw, &status, &resp_hdrs, &resp_body);
    vibe_http_fill_response_record(out_record, status, resp_hdrs, resp_body);
    free(raw);
}

static char *vibe_http_request_body_curl(
    const char *method,
    const char *url,
    const char *body,
    int64_t timeout_ms
) {
    const char *verb = (method == NULL || method[0] == '\0') ? "GET" : method;
    const char *payload = body == NULL ? "" : body;
    double timeout_sec = timeout_ms > 0 ? ((double)timeout_ms / 1000.0) : 10.0;
    if (timeout_sec < 0.001) {
        timeout_sec = 0.001;
    }
    char timeout_buf[32];
    snprintf(timeout_buf, sizeof(timeout_buf), "%.3f", timeout_sec);
    char *q_method = vibe_shell_single_quote(verb);
    char *q_url = vibe_shell_single_quote(url == NULL ? "" : url);
    char *q_body = vibe_shell_single_quote(payload);
    vibe_string_builder cmd;
    vibe_builder_init(&cmd, strlen(url == NULL ? "" : url) + strlen(payload) + 128);
    vibe_builder_append_bytes(&cmd, "curl -sS -L --max-time ", 23);
    vibe_builder_append_bytes(&cmd, timeout_buf, strlen(timeout_buf));
    vibe_builder_append_bytes(&cmd, " -X ", 4);
    vibe_builder_append_bytes(&cmd, q_method, strlen(q_method));
    if (payload[0] != '\0') {
        vibe_builder_append_bytes(&cmd, " --data-binary ", 15);
        vibe_builder_append_bytes(&cmd, q_body, strlen(q_body));
    }
    vibe_builder_append_bytes(&cmd, " ", 1);
    vibe_builder_append_bytes(&cmd, q_url, strlen(q_url));
    vibe_builder_append_bytes(&cmd, " 2>/dev/null", 12);
    char *out = vibe_exec_capture_stdout(cmd.data);
    free(q_method);
    free(q_url);
    free(q_body);
    free(cmd.data);
    return out;
}

static int64_t vibe_http_request_status_curl(
    const char *method,
    const char *url,
    const char *body,
    int64_t timeout_ms
) {
    const char *verb = (method == NULL || method[0] == '\0') ? "GET" : method;
    const char *payload = body == NULL ? "" : body;
    double timeout_sec = timeout_ms > 0 ? ((double)timeout_ms / 1000.0) : 10.0;
    if (timeout_sec < 0.001) {
        timeout_sec = 0.001;
    }
    char timeout_buf[32];
    snprintf(timeout_buf, sizeof(timeout_buf), "%.3f", timeout_sec);
    char *q_method = vibe_shell_single_quote(verb);
    char *q_url = vibe_shell_single_quote(url == NULL ? "" : url);
    char *q_body = vibe_shell_single_quote(payload);
    vibe_string_builder cmd;
    vibe_builder_init(&cmd, strlen(url == NULL ? "" : url) + strlen(payload) + 160);
    vibe_builder_append_bytes(&cmd, "curl -sS -L --max-time ", 23);
    vibe_builder_append_bytes(&cmd, timeout_buf, strlen(timeout_buf));
    vibe_builder_append_bytes(&cmd, " -o /dev/null -w '%{http_code}' -X ", 35);
    vibe_builder_append_bytes(&cmd, q_method, strlen(q_method));
    if (payload[0] != '\0') {
        vibe_builder_append_bytes(&cmd, " --data-binary ", 15);
        vibe_builder_append_bytes(&cmd, q_body, strlen(q_body));
    }
    vibe_builder_append_bytes(&cmd, " ", 1);
    vibe_builder_append_bytes(&cmd, q_url, strlen(q_url));
    vibe_builder_append_bytes(&cmd, " 2>/dev/null", 12);
    char *raw = vibe_exec_capture_stdout(cmd.data);
    char *end = NULL;
    long code = strtol(raw == NULL ? "" : raw, &end, 10);
    free(raw);
    free(q_method);
    free(q_url);
    free(q_body);
    free(cmd.data);
    if (end == NULL) {
        return 0;
    }
    return code <= 0 ? 0 : (int64_t)code;
}

static char *vibe_http_request_raw_plain(
    const char *method,
    const char *url,
    const char *custom_headers,
    const char *body,
    int64_t timeout_ms
) {
    char host[256];
    memset(host, 0, sizeof(host));
    int64_t port = 0;
    const char *path = "/";
    int is_https = 0;
    if (!vibe_http_parse_url(url, host, sizeof(host), &port, &path, &is_https) || is_https) {
        return vibe_strdup_or_panic("");
    }
    int64_t conn = vibe_net_connect(host, port);
    if (conn == 0) {
        char *resolved = vibe_net_resolve_first(host);
        if (resolved != NULL && resolved[0] != '\0') {
            conn = vibe_net_connect(resolved, port);
        }
        free(resolved);
    }
    if (conn == 0) {
        return vibe_strdup_or_panic("");
    }

    int fd = (int)conn;
    vibe_http_apply_timeout(fd, timeout_ms);
    const char *verb = (method == NULL || method[0] == '\0') ? "GET" : method;
    const char *payload = body == NULL ? "" : body;
    const char *hdrs = custom_headers == NULL ? "" : custom_headers;
    size_t payload_len = strlen(payload);
    size_t hdrs_len = strlen(hdrs);
    size_t cap = strlen(verb) + strlen(path) + strlen(host) + hdrs_len + payload_len + 256;
    vibe_string_builder sb;
    vibe_builder_init(&sb, cap);
    char line_buf[512];
    int n = snprintf(line_buf, sizeof(line_buf), "%s %s HTTP/1.1\r\nHost: %s\r\nConnection: close\r\n", verb, path, host);
    vibe_builder_append_bytes(&sb, line_buf, (size_t)(n > 0 ? n : 0));
    if (hdrs_len > 0) {
        const char *p = hdrs;
        while (*p != '\0') {
            const char *le = p;
            while (*le != '\0' && *le != '\r' && *le != '\n') le++;
            size_t ll = (size_t)(le - p);
            if (ll > 0) {
                vibe_builder_append_bytes(&sb, p, ll);
                vibe_builder_append_bytes(&sb, "\r\n", 2);
            }
            p = le;
            while (*p == '\r' || *p == '\n') p++;
        }
    }
    n = snprintf(line_buf, sizeof(line_buf), "Content-Length: %zu\r\n\r\n", payload_len);
    vibe_builder_append_bytes(&sb, line_buf, (size_t)(n > 0 ? n : 0));
    vibe_builder_append_bytes(&sb, payload, payload_len);
    (void)vibe_net_write(conn, sb.data);
    free(sb.data);
    char *raw_response = vibe_http_read_all_response(fd, 4 * 1024 * 1024);
    (void)vibe_net_close(conn);
    return raw_response;
}

void *vibe_http_send(const char *method, const char *url, const char *headers, const char *body, int64_t timeout_ms) {
    void *record = vibe_record_alloc(3);
    if (url == NULL || url[0] == '\0') {
        vibe_http_fill_response_record(record, 0, "", "");
        return record;
    }
    if (strncmp(url, "https://", 8) == 0) {
        vibe_http_curl_send(method, url, headers, body, timeout_ms, record);
        return record;
    }
    char *raw_response = vibe_http_request_raw_plain(method, url, headers, body, timeout_ms);
    int64_t status = 0;
    const char *resp_hdrs = "";
    const char *resp_body = "";
    vibe_http_parse_raw_response(raw_response, &status, &resp_hdrs, &resp_body);
    vibe_http_fill_response_record(record, status, resp_hdrs, resp_body);
    free(raw_response);
    return record;
}

char *vibe_http_request(const char *method, const char *url, const char *body, int64_t timeout_ms) {
    if (url == NULL || url[0] == '\0') {
        return vibe_strdup_or_panic("");
    }
    if (strncmp(url, "https://", 8) == 0) {
        return vibe_http_request_body_curl(method, url, body, timeout_ms);
    }
    char *raw_response = vibe_http_request_raw_plain(method, url, "", body, timeout_ms);
    if (raw_response == NULL || raw_response[0] == '\0') {
        free(raw_response);
        return vibe_strdup_or_panic("");
    }
    const char *body_ptr = vibe_http_find_body(raw_response);
    char *out = NULL;
    if (body_ptr == NULL) {
        out = vibe_strdup_or_panic(raw_response);
    } else {
        out = vibe_strdup_or_panic(body_ptr);
    }
    free(raw_response);
    return out;
}

int64_t vibe_http_request_status(const char *method, const char *url, const char *body, int64_t timeout_ms) {
    if (url == NULL || url[0] == '\0') {
        return 0;
    }
    if (strncmp(url, "https://", 8) == 0) {
        return vibe_http_request_status_curl(method, url, body, timeout_ms);
    }
    char *raw_response = vibe_http_request_raw_plain(method, url, "", body, timeout_ms);
    int64_t code = vibe_http_parse_status_code(raw_response);
    free(raw_response);
    return code;
}

void *vibe_http_get(const char *url, int64_t timeout_ms) {
    return vibe_http_send("GET", url, "", "", timeout_ms);
}

void *vibe_http_post(const char *url, const char *body, int64_t timeout_ms) {
    return vibe_http_send("POST", url, "", body, timeout_ms);
}

void *vibe_http_send_struct(void *req) {
    long *slots = (long *)req;
    return vibe_http_send((const char *)slots[0], (const char *)slots[1],
                          (const char *)slots[2], (const char *)slots[3], slots[4]);
}

static int64_t vibe_http_extract_i64(const char *text) {
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

static const char *vibe_http_find_body(const char *req) {
    if (req == NULL) {
        return NULL;
    }
    const char *p = strstr(req, "\r\n\r\n");
    return p == NULL ? NULL : (p + 4);
}

static char *vibe_http_read_message(int fd, int64_t max_bytes) {
    if (max_bytes <= 0) {
        return vibe_strdup_or_panic("");
    }
    if (max_bytes > 4 * 1024 * 1024) {
        max_bytes = 4 * 1024 * 1024;
    }
    vibe_string_builder builder;
    vibe_builder_init(&builder, 4096);
    while ((int64_t)builder.len < max_bytes) {
        char tmp[4096];
        size_t remain = (size_t)(max_bytes - (int64_t)builder.len);
        size_t to_read = remain < sizeof(tmp) ? remain : sizeof(tmp);
        ssize_t n = recv(fd, tmp, to_read, 0);
        if (n <= 0) {
            break;
        }
        vibe_builder_append_bytes(&builder, tmp, (size_t)n);
        const char *body = vibe_http_find_body(builder.data);
        if (body != NULL) {
            if (vibe_http_extract_i64(body) != 0 || strstr(body, "0") != NULL) {
                break;
            }
        }
    }
    return builder.data;
}

static void vibe_http_handle_conn(int conn) {
    char *req = vibe_http_read_message(conn, 16384);
    const char *body = vibe_http_find_body(req);
    int64_t value = vibe_http_extract_i64(body);
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
    vibe_net_write((int64_t)conn, header);
    vibe_net_write((int64_t)conn, resp_body);
    vibe_net_close((int64_t)conn);
}

typedef struct vibe_http_server_ctx {
    int listener;
    int64_t total;
} vibe_http_server_ctx;

static void *vibe_http_server_thread(void *arg) {
    vibe_http_server_ctx *ctx = (vibe_http_server_ctx *)arg;
    int handled = 0;
    while ((int64_t)handled < ctx->total) {
        int conn = (int)vibe_net_accept((int64_t)ctx->listener);
        if (conn != 0) {
            vibe_http_handle_conn(conn);
            handled++;
        }
    }
    vibe_net_close((int64_t)ctx->listener);
    return NULL;
}

typedef struct vibe_http_client_ctx {
    int port;
    int64_t value;
    int64_t *out;
} vibe_http_client_ctx;

static void *vibe_http_client_thread(void *arg) {
    vibe_http_client_ctx *ctx = (vibe_http_client_ctx *)arg;
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
        conn = vibe_net_connect("127.0.0.1", (int64_t)ctx->port);
        if (conn != 0) {
            break;
        }
    }
    if (conn == 0) {
        *ctx->out = 0;
        return NULL;
    }
    vibe_net_write(conn, req);
    char *resp = vibe_http_read_message((int)conn, 16384);
    vibe_net_close(conn);
    const char *body_ptr = vibe_http_find_body(resp);
    int64_t parsed = vibe_http_extract_i64(body_ptr);
    free(resp);
    *ctx->out = parsed;
    return NULL;
}

int64_t vibe_http_server_bench(int64_t n) {
#ifdef _WIN32
    (void)n;
    return 0;
#else
    if (n <= 0) {
        return 0;
    }
    int64_t listener_raw = vibe_net_listen("127.0.0.1", 0);
    if (listener_raw == 0) {
        return 0;
    }
    int listener = (int)listener_raw;
    int port = (int)vibe_net_listener_port(listener_raw);
    if (port <= 0) {
        vibe_net_close(listener_raw);
        return 0;
    }

    vibe_http_server_ctx server_ctx = {
        .listener = listener,
        .total = n,
    };
    pthread_t server_thread;
    if (pthread_create(&server_thread, NULL, vibe_http_server_thread, &server_ctx) != 0) {
        vibe_net_close(listener_raw);
        return 0;
    }

    pthread_t *threads = (pthread_t *)calloc((size_t)n, sizeof(pthread_t));
    vibe_http_client_ctx *ctxs =
        (vibe_http_client_ctx *)calloc((size_t)n, sizeof(vibe_http_client_ctx));
    int64_t *results = (int64_t *)calloc((size_t)n, sizeof(int64_t));
    if (threads == NULL || ctxs == NULL || results == NULL) {
        vibe_panic("failed to allocate http-server bench resources");
    }
    for (int64_t i = 0; i < n; i++) {
        ctxs[i].port = port;
        ctxs[i].value = i + 1;
        ctxs[i].out = &results[i];
        (void)pthread_create(&threads[i], NULL, vibe_http_client_thread, &ctxs[i]);
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

// --- secp256k1 (field arithmetic + point multiplication) ---

typedef struct vibe_fe256 {
    uint64_t v[4]; // little-endian limbs
} vibe_fe256;

static const uint64_t VIBE_SECP_P[4] = {
    0xFFFFFFFEFFFFFC2Full,
    0xFFFFFFFFFFFFFFFFull,
    0xFFFFFFFFFFFFFFFFull,
    0xFFFFFFFFFFFFFFFFull,
};

static int vibe_fe_ge_p(const vibe_fe256 *a) {
    for (int i = 3; i >= 0; i--) {
        if (a->v[i] > VIBE_SECP_P[i]) return 1;
        if (a->v[i] < VIBE_SECP_P[i]) return 0;
    }
    return 1;
}

static void vibe_fe_sub_p(vibe_fe256 *a) {
    __uint128_t borrow = 0;
    for (int i = 0; i < 4; i++) {
        __uint128_t av = (__uint128_t)a->v[i];
        __uint128_t bv = (__uint128_t)VIBE_SECP_P[i] + borrow;
        __uint128_t r = av - bv;
        a->v[i] = (uint64_t)r;
        borrow = av < bv ? 1u : 0u;
    }
}

static void vibe_fe_add(vibe_fe256 *out, const vibe_fe256 *a, const vibe_fe256 *b) {
    __uint128_t carry = 0;
    for (int i = 0; i < 4; i++) {
        __uint128_t s = (__uint128_t)a->v[i] + b->v[i] + carry;
        out->v[i] = (uint64_t)s;
        carry = s >> 64;
    }
    if (carry || vibe_fe_ge_p(out)) {
        vibe_fe_sub_p(out);
    }
}

static void vibe_fe_sub(vibe_fe256 *out, const vibe_fe256 *a, const vibe_fe256 *b) {
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
            __uint128_t s = (__uint128_t)out->v[i] + VIBE_SECP_P[i] + carry;
            out->v[i] = (uint64_t)s;
            carry = s >> 64;
        }
    }
}

static void vibe_fe_mul_small_add_shift32(vibe_fe256 *io, uint64_t hi) {
    // Fold hi * 2^256 using p = 2^256 - 2^32 - 977.
    // 2^256 ≡ 2^32 + 977 (mod p) => add (hi<<32) and (hi*977).
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
    if (vibe_fe_ge_p(io)) {
        vibe_fe_sub_p(io);
    }
}

static void vibe_fe_reduce512(vibe_fe256 *out, const uint64_t in[8]) {
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
        vibe_fe_mul_small_add_shift32(out, high);
    } else if (vibe_fe_ge_p(out)) {
        vibe_fe_sub_p(out);
    }
}

static void vibe_fe_mul(vibe_fe256 *out, const vibe_fe256 *a, const vibe_fe256 *b) {
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
    vibe_fe_reduce512(out, prod);
}

static void vibe_fe_sqr(vibe_fe256 *out, const vibe_fe256 *a) {
    vibe_fe_mul(out, a, a);
}

static int vibe_fe_is_zero(const vibe_fe256 *a) {
    return (a->v[0] | a->v[1] | a->v[2] | a->v[3]) == 0;
}

static void vibe_fe_copy(vibe_fe256 *out, const vibe_fe256 *a) {
    out->v[0] = a->v[0];
    out->v[1] = a->v[1];
    out->v[2] = a->v[2];
    out->v[3] = a->v[3];
}

static void vibe_fe_from_u64(vibe_fe256 *out, uint64_t x) {
    out->v[0] = x;
    out->v[1] = 0;
    out->v[2] = 0;
    out->v[3] = 0;
}

static void vibe_fe_pow_pminus2(vibe_fe256 *out, const vibe_fe256 *a) {
    // exponent = p-2
    const uint64_t exp[4] = {
        0xFFFFFFFEFFFFFC2Dull,
        0xFFFFFFFFFFFFFFFFull,
        0xFFFFFFFFFFFFFFFFull,
        0xFFFFFFFFFFFFFFFFull,
    };
    vibe_fe256 result;
    vibe_fe_from_u64(&result, 1);
    vibe_fe256 base;
    vibe_fe_copy(&base, a);
    for (int limb = 3; limb >= 0; limb--) {
        for (int bit = 63; bit >= 0; bit--) {
            vibe_fe_sqr(&result, &result);
            if ((exp[limb] >> bit) & 1ull) {
                vibe_fe_mul(&result, &result, &base);
            }
        }
    }
    vibe_fe_copy(out, &result);
}

typedef struct vibe_jacobian {
    vibe_fe256 x;
    vibe_fe256 y;
    vibe_fe256 z;
} vibe_jacobian;

static void vibe_jacobian_zero(vibe_jacobian *p) {
    vibe_fe_from_u64(&p->x, 0);
    vibe_fe_from_u64(&p->y, 1);
    vibe_fe_from_u64(&p->z, 0);
}

static int vibe_jacobian_is_zero(const vibe_jacobian *p) {
    return vibe_fe_is_zero(&p->z);
}

static void vibe_jacobian_double(vibe_jacobian *out, const vibe_jacobian *p) {
    if (vibe_jacobian_is_zero(p)) {
        *out = *p;
        return;
    }
    vibe_fe256 a, b, c, d, e, f, tmp;
    vibe_fe_sqr(&a, &p->x);
    vibe_fe_sqr(&b, &p->y);
    vibe_fe_sqr(&c, &b);

    vibe_fe_add(&tmp, &p->x, &b);      // x + b
    vibe_fe_sqr(&tmp, &tmp);           // (x+b)^2
    vibe_fe_sub(&tmp, &tmp, &a);
    vibe_fe_sub(&tmp, &tmp, &c);
    vibe_fe_add(&d, &tmp, &tmp);       // 2 * (...)

    vibe_fe_add(&e, &a, &a);
    vibe_fe_add(&e, &e, &a);           // 3*a
    vibe_fe_sqr(&f, &e);               // e^2

    vibe_fe_add(&tmp, &d, &d);         // 2*d
    vibe_fe_sub(&out->x, &f, &tmp);

    vibe_fe_sub(&tmp, &d, &out->x);
    vibe_fe_mul(&tmp, &e, &tmp);
    vibe_fe_add(&c, &c, &c);
    vibe_fe_add(&c, &c, &c);
    vibe_fe_add(&c, &c, &c);           // 8*c
    vibe_fe_sub(&out->y, &tmp, &c);

    vibe_fe_mul(&tmp, &p->y, &p->z);
    vibe_fe_add(&out->z, &tmp, &tmp);  // 2*y*z
}

static void vibe_jacobian_add(vibe_jacobian *out, const vibe_jacobian *p, const vibe_jacobian *q) {
    if (vibe_jacobian_is_zero(q)) {
        *out = *p;
        return;
    }
    if (vibe_jacobian_is_zero(p)) {
        *out = *q;
        return;
    }
    vibe_fe256 z1z1, z2z2, u1, u2, s1, s2, h, r, hh, hhh, v, tmp, x3, y3, z3;
    vibe_fe_sqr(&z1z1, &p->z);
    vibe_fe_sqr(&z2z2, &q->z);
    vibe_fe_mul(&u1, &p->x, &z2z2);
    vibe_fe_mul(&u2, &q->x, &z1z1);
    vibe_fe_mul(&tmp, &q->z, &z2z2);
    vibe_fe_mul(&s1, &p->y, &tmp);
    vibe_fe_mul(&tmp, &p->z, &z1z1);
    vibe_fe_mul(&s2, &q->y, &tmp);
    vibe_fe_sub(&h, &u2, &u1);
    vibe_fe_sub(&r, &s2, &s1);
    if (vibe_fe_is_zero(&h)) {
        if (vibe_fe_is_zero(&r)) {
            vibe_jacobian_double(out, p);
        } else {
            vibe_jacobian_zero(out);
        }
        return;
    }
    vibe_fe_sqr(&hh, &h);
    vibe_fe_mul(&hhh, &h, &hh);
    vibe_fe_mul(&v, &u1, &hh);
    vibe_fe_sqr(&tmp, &r);
    vibe_fe_add(&x3, &v, &v);
    vibe_fe_sub(&x3, &tmp, &x3);
    vibe_fe_sub(&x3, &x3, &hhh);
    vibe_fe_sub(&tmp, &v, &x3);
    vibe_fe_mul(&tmp, &r, &tmp);
    vibe_fe_mul(&y3, &s1, &hhh);
    vibe_fe_sub(&y3, &tmp, &y3);
    vibe_fe_mul(&tmp, &p->z, &q->z);
    vibe_fe_mul(&z3, &tmp, &h);
    out->x = x3;
    out->y = y3;
    out->z = z3;
}

static void vibe_jacobian_mul_scalar(vibe_jacobian *out, const vibe_jacobian *p, const uint64_t scalar[4]) {
    vibe_jacobian acc;
    vibe_jacobian_zero(&acc);
    vibe_jacobian d = *p;
    for (int limb = 0; limb < 4; limb++) {
        uint64_t w = scalar[limb];
        for (int bit = 0; bit < 64; bit++) {
            if (w & 1ull) {
                vibe_jacobian tmp;
                vibe_jacobian_add(&tmp, &acc, &d);
                acc = tmp;
            }
            vibe_jacobian tmpd;
            vibe_jacobian_double(&tmpd, &d);
            d = tmpd;
            w >>= 1;
        }
    }
    *out = acc;
}

static void vibe_jacobian_to_affine(vibe_fe256 *x, vibe_fe256 *y, const vibe_jacobian *p) {
    if (vibe_jacobian_is_zero(p)) {
        vibe_fe_from_u64(x, 0);
        vibe_fe_from_u64(y, 0);
        return;
    }
    vibe_fe256 invz, invz2, invz3;
    vibe_fe_pow_pminus2(&invz, &p->z);
    vibe_fe_sqr(&invz2, &invz);
    vibe_fe_mul(&invz3, &invz2, &invz);
    vibe_fe_mul(x, &p->x, &invz2);
    vibe_fe_mul(y, &p->y, &invz3);
}

static void vibe_fe_to_hex(char out[65], const vibe_fe256 *a) {
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

char *vibe_secp256k1_bench(int64_t n) {
    if (n <= 0) {
        return vibe_strdup_or_panic("");
    }
    const uint64_t priv[4] = {
        0xd9132d70b5bc7693ull,
        0xb64592e3fe0ab0daull,
        0x4fca3ef970ff4d38ull,
        0x2dee927079283c3cull,
    };
    vibe_jacobian p;
    p.x.v[0] = 0x59f2815b16f81798ull;
    p.x.v[1] = 0x029bfcdb2dce28d9ull;
    p.x.v[2] = 0x55a06295ce870b07ull;
    p.x.v[3] = 0x79be667ef9dcbbacull;
    p.y.v[0] = 0x9c47d08ffb10d4b8ull;
    p.y.v[1] = 0xfd17b448a6855419ull;
    p.y.v[2] = 0x5da4fbfc0e1108a8ull;
    p.y.v[3] = 0x483ada7726a3c465ull;
    vibe_fe_from_u64(&p.z, 1);

    for (int64_t i = 0; i < n; i++) {
        vibe_jacobian next;
        vibe_jacobian_mul_scalar(&next, &p, priv);
        p = next;
    }

    vibe_fe256 ax, ay;
    vibe_jacobian_to_affine(&ax, &ay, &p);
    char hx[65];
    char hy[65];
    vibe_fe_to_hex(hx, &ax);
    vibe_fe_to_hex(hy, &ay);

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

// --- edigits (digits of e) ---

#define VIBE_BIG_BASE 1000000000u

typedef struct vibe_big {
    uint32_t *d; // little-endian base 1e9 limbs
    size_t len;
    size_t cap;
} vibe_big;

static void vibe_big_trim(vibe_big *a) {
    while (a->len > 0 && a->d[a->len - 1] == 0) {
        a->len--;
    }
}

static void vibe_big_reserve(vibe_big *a, size_t cap) {
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

static void vibe_big_init_u32(vibe_big *a, uint32_t v) {
    a->d = NULL;
    a->len = 0;
    a->cap = 0;
    if (v == 0) {
        return;
    }
    vibe_big_reserve(a, 1);
    a->d[0] = v;
    a->len = 1;
}

static void vibe_big_copy(vibe_big *out, const vibe_big *a) {
    out->d = NULL;
    out->len = 0;
    out->cap = 0;
    if (a->len == 0) {
        return;
    }
    vibe_big_reserve(out, a->len);
    memcpy(out->d, a->d, a->len * sizeof(uint32_t));
    out->len = a->len;
}

static void vibe_big_free(vibe_big *a) {
    free(a->d);
    a->d = NULL;
    a->len = 0;
    a->cap = 0;
}

static int vibe_big_cmp(const vibe_big *a, const vibe_big *b) {
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

static void vibe_big_add(vibe_big *out, const vibe_big *a, const vibe_big *b) {
    size_t n = a->len > b->len ? a->len : b->len;
    vibe_big_reserve(out, n + 1);
    uint64_t carry = 0;
    for (size_t i = 0; i < n; i++) {
        uint64_t av = i < a->len ? a->d[i] : 0;
        uint64_t bv = i < b->len ? b->d[i] : 0;
        uint64_t s = av + bv + carry;
        out->d[i] = (uint32_t)(s % VIBE_BIG_BASE);
        carry = s / VIBE_BIG_BASE;
    }
    if (carry) {
        out->d[n] = (uint32_t)carry;
        out->len = n + 1;
    } else {
        out->len = n;
    }
    vibe_big_trim(out);
}

static void vibe_big_add_inplace(vibe_big *a, const vibe_big *b) {
    vibe_big tmp = {0};
    vibe_big_add(&tmp, a, b);
    vibe_big_free(a);
    *a = tmp;
}

static void vibe_big_sub_inplace(vibe_big *a, const vibe_big *b) {
    // requires a >= b
    uint64_t borrow = 0;
    for (size_t i = 0; i < a->len; i++) {
        uint64_t av = a->d[i];
        uint64_t bv = (i < b->len ? b->d[i] : 0) + borrow;
        if (av < bv) {
            a->d[i] = (uint32_t)(VIBE_BIG_BASE + av - bv);
            borrow = 1;
        } else {
            a->d[i] = (uint32_t)(av - bv);
            borrow = 0;
        }
    }
    vibe_big_trim(a);
}

static void vibe_big_mul_small_inplace(vibe_big *a, uint32_t m) {
    if (a->len == 0 || m == 1) {
        return;
    }
    if (m == 0) {
        a->len = 0;
        return;
    }
    vibe_big_reserve(a, a->len + 1);
    uint64_t carry = 0;
    for (size_t i = 0; i < a->len; i++) {
        uint64_t cur = (uint64_t)a->d[i] * m + carry;
        a->d[i] = (uint32_t)(cur % VIBE_BIG_BASE);
        carry = cur / VIBE_BIG_BASE;
    }
    if (carry) {
        a->d[a->len++] = (uint32_t)carry;
    }
    vibe_big_trim(a);
}

static uint32_t vibe_big_div_small_inplace(vibe_big *a, uint32_t v) {
    uint64_t rem = 0;
    for (size_t i = a->len; i > 0; i--) {
        uint64_t cur = a->d[i - 1] + rem * VIBE_BIG_BASE;
        a->d[i - 1] = (uint32_t)(cur / v);
        rem = cur % v;
    }
    vibe_big_trim(a);
    return (uint32_t)rem;
}

static void vibe_big_mul(vibe_big *out, const vibe_big *a, const vibe_big *b) {
    if (a->len == 0 || b->len == 0) {
        out->len = 0;
        return;
    }
    vibe_big_reserve(out, a->len + b->len);
    memset(out->d, 0, (a->len + b->len) * sizeof(uint32_t));
    for (size_t i = 0; i < a->len; i++) {
        uint64_t carry = 0;
        for (size_t j = 0; j < b->len || carry; j++) {
            uint64_t cur = out->d[i + j] +
                           (uint64_t)a->d[i] * (j < b->len ? b->d[j] : 0) + carry;
            out->d[i + j] = (uint32_t)(cur % VIBE_BIG_BASE);
            carry = cur / VIBE_BIG_BASE;
        }
    }
    out->len = a->len + b->len;
    vibe_big_trim(out);
}

static void vibe_big_mul_small(vibe_big *out, const vibe_big *a, uint32_t m) {
    vibe_big_copy(out, a);
    vibe_big_mul_small_inplace(out, m);
}

static void vibe_big_shift_base_add(vibe_big *r, uint32_t digit) {
    vibe_big_reserve(r, r->len + 1);
    if (r->len > 0) {
        memmove(r->d + 1, r->d, r->len * sizeof(uint32_t));
    }
    r->d[0] = digit;
    r->len += 1;
    vibe_big_trim(r);
}

static void vibe_big_div(vibe_big *q, const vibe_big *a1, const vibe_big *b1) {
    // q = a1 / b1, integers, b1 > 0
    if (b1->len == 0) {
        vibe_panic("division by zero bigint");
    }
    if (a1->len == 0) {
        q->len = 0;
        return;
    }
    if (vibe_big_cmp(a1, b1) < 0) {
        q->len = 0;
        return;
    }
    uint32_t b_msd = b1->d[b1->len - 1];
    uint32_t norm = (uint32_t)(VIBE_BIG_BASE / ((uint64_t)b_msd + 1));

    vibe_big a = {0};
    vibe_big b = {0};
    vibe_big_copy(&a, a1);
    vibe_big_copy(&b, b1);
    if (norm != 1) {
        vibe_big_mul_small_inplace(&a, norm);
        vibe_big_mul_small_inplace(&b, norm);
    }

    vibe_big r = {0};
    q->len = 0;
    vibe_big_reserve(q, a.len);
    memset(q->d, 0, a.len * sizeof(uint32_t));
    q->len = a.len;

    for (size_t i = a.len; i > 0; i--) {
        vibe_big_shift_base_add(&r, a.d[i - 1]);
        uint32_t s1 = r.len <= b.len ? 0 : r.d[b.len];
        uint32_t s2 = r.len <= b.len - 1 ? 0 : r.d[b.len - 1];
        uint64_t d_est = ((uint64_t)s1 * VIBE_BIG_BASE + s2) / b.d[b.len - 1];
        if (d_est >= VIBE_BIG_BASE) {
            d_est = VIBE_BIG_BASE - 1;
        }

        vibe_big bd = {0};
        vibe_big_mul_small(&bd, &b, (uint32_t)d_est);
        while (vibe_big_cmp(&r, &bd) < 0) {
            vibe_big_free(&bd);
            d_est -= 1;
            vibe_big_mul_small(&bd, &b, (uint32_t)d_est);
        }
        vibe_big_sub_inplace(&r, &bd);
        vibe_big_free(&bd);
        q->d[i - 1] = (uint32_t)d_est;
    }
    vibe_big_trim(q);
    if (norm != 1) {
        (void)vibe_big_div_small_inplace(&r, norm);
    }
    vibe_big_free(&a);
    vibe_big_free(&b);
    vibe_big_free(&r);
}

static char *vibe_big_to_dec_str(const vibe_big *a) {
    if (a->len == 0) {
        return vibe_strdup_or_panic("0");
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

typedef struct vibe_pq {
    vibe_big p;
    vibe_big q;
} vibe_pq;

static vibe_pq vibe_sum_terms(uint32_t a, uint32_t b) {
    if (b == a + 1) {
        vibe_pq base = {0};
        vibe_big_init_u32(&base.p, 1);
        vibe_big_init_u32(&base.q, b);
        return base;
    }
    uint32_t mid = (a + b) / 2;
    vibe_pq left = vibe_sum_terms(a, mid);
    vibe_pq right = vibe_sum_terms(mid, b);

    vibe_big p_left_q_right = {0};
    vibe_big_mul(&p_left_q_right, &left.p, &right.q);
    vibe_big_add_inplace(&p_left_q_right, &right.p);

    vibe_big q_left_q_right = {0};
    vibe_big_mul(&q_left_q_right, &left.q, &right.q);

    vibe_big_free(&left.p);
    vibe_big_free(&left.q);
    vibe_big_free(&right.p);
    vibe_big_free(&right.q);

    vibe_pq out = {0};
    out.p = p_left_q_right;
    out.q = q_left_q_right;
    return out;
}

static uint32_t vibe_edigits_find_k(int64_t n) {
    // Find k such that log10(k!) >= n + 50 via Stirling approximation.
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

static char *vibe_edigits_calculate(int64_t n) {
    if (n <= 0) {
        return vibe_strdup_or_panic("");
    }
    uint32_t k = vibe_edigits_find_k(n);
    vibe_pq pq = vibe_sum_terms(0, k - 1);
    vibe_big_add_inplace(&pq.p, &pq.q); // p += q

    // multiply p by 10^(n-1)
    int64_t exp = n - 1;
    uint32_t shift = (uint32_t)(exp / 9);
    uint32_t rem = (uint32_t)(exp % 9);
    uint32_t pow10 = 1;
    for (uint32_t i = 0; i < rem; i++) {
        pow10 *= 10u;
    }
    vibe_big_mul_small_inplace(&pq.p, pow10);
    if (shift > 0 && pq.p.len > 0) {
        vibe_big_reserve(&pq.p, pq.p.len + shift);
        memmove(pq.p.d + shift, pq.p.d, pq.p.len * sizeof(uint32_t));
        memset(pq.p.d, 0, shift * sizeof(uint32_t));
        pq.p.len += shift;
    }
    vibe_big_trim(&pq.p);

    vibe_big answer = {0};
    vibe_big_div(&answer, &pq.p, &pq.q);
    char *s = vibe_big_to_dec_str(&answer);

    vibe_big_free(&pq.p);
    vibe_big_free(&pq.q);
    vibe_big_free(&answer);

    // Ensure length n (left-pad with zeros if necessary)
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

char *vibe_edigits(int64_t n) {
    if (n <= 0) {
        n = 27;
    }
    char *digits = vibe_edigits_calculate(n);
    vibe_string_builder builder;
    vibe_builder_init(&builder, (size_t)n + 64);
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
        vibe_builder_append_bytes(&builder, line, 10);
        vibe_builder_append_bytes(&builder, suffix, (size_t)(suffix_len > 0 ? suffix_len : 0));
    }
    free(digits);
    return builder.data;
}

/* -------------------------------------------------------------------------- */
/* std.crypto: SHA-256, HMAC-SHA256, CSPRNG, UUID v4, constant-time compare   */
/* -------------------------------------------------------------------------- */

typedef struct {
    uint32_t state[8];
    uint64_t bitcount;
    uint8_t buf[64];
    size_t buflen;
} vibe_sha256_ctx;

#define VIBE_SHA256_ROTR(n, x) (((x) >> (n)) | ((x) << (32 - (n))))
#define VIBE_SHA256_CH(x, y, z) (((x) & (y)) ^ (~(x) & (z)))
#define VIBE_SHA256_MAJ(x, y, z) (((x) & (y)) ^ ((x) & (z)) ^ ((y) & (z)))
#define VIBE_SHA256_EP0(x) (VIBE_SHA256_ROTR(2, x) ^ VIBE_SHA256_ROTR(13, x) ^ VIBE_SHA256_ROTR(22, x))
#define VIBE_SHA256_EP1(x) (VIBE_SHA256_ROTR(6, x) ^ VIBE_SHA256_ROTR(11, x) ^ VIBE_SHA256_ROTR(25, x))
#define VIBE_SHA256_SIG0(x) (VIBE_SHA256_ROTR(7, x) ^ VIBE_SHA256_ROTR(18, x) ^ ((x) >> 3))
#define VIBE_SHA256_SIG1(x) (VIBE_SHA256_ROTR(17, x) ^ VIBE_SHA256_ROTR(19, x) ^ ((x) >> 10))

static void vibe_sha256_transform(uint32_t state[8], const uint8_t block[64]) {
    static const uint32_t K[64] = {
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    };
    uint32_t W[64];
    for (int i = 0; i < 16; i++) {
        W[i] = ((uint32_t)block[i * 4] << 24) | ((uint32_t)block[i * 4 + 1] << 16) |
               ((uint32_t)block[i * 4 + 2] << 8) | ((uint32_t)block[i * 4 + 3]);
    }
    for (int i = 16; i < 64; i++) {
        W[i] = VIBE_SHA256_SIG1(W[i - 2]) + W[i - 7] + VIBE_SHA256_SIG0(W[i - 15]) + W[i - 16];
    }

    uint32_t a = state[0];
    uint32_t b = state[1];
    uint32_t c = state[2];
    uint32_t d = state[3];
    uint32_t e = state[4];
    uint32_t f = state[5];
    uint32_t g = state[6];
    uint32_t h = state[7];

    for (int i = 0; i < 64; i++) {
        uint32_t t1 = h + VIBE_SHA256_EP1(e) + VIBE_SHA256_CH(e, f, g) + K[i] + W[i];
        uint32_t t2 = VIBE_SHA256_EP0(a) + VIBE_SHA256_MAJ(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }

    state[0] += a;
    state[1] += b;
    state[2] += c;
    state[3] += d;
    state[4] += e;
    state[5] += f;
    state[6] += g;
    state[7] += h;
}

static void vibe_sha256_init(vibe_sha256_ctx *ctx) {
    ctx->state[0] = 0x6a09e667;
    ctx->state[1] = 0xbb67ae85;
    ctx->state[2] = 0x3c6ef372;
    ctx->state[3] = 0xa54ff53a;
    ctx->state[4] = 0x510e527f;
    ctx->state[5] = 0x9b05688c;
    ctx->state[6] = 0x1f83d9ab;
    ctx->state[7] = 0x5be0cd19;
    ctx->bitcount = 0;
    ctx->buflen = 0;
}

static void vibe_sha256_update(vibe_sha256_ctx *ctx, const uint8_t *data, size_t len) {
    ctx->bitcount += (uint64_t)len * 8;
    while (len > 0) {
        size_t space = 64 - ctx->buflen;
        size_t take = len < space ? len : space;
        memcpy(ctx->buf + ctx->buflen, data, take);
        ctx->buflen += take;
        data += take;
        len -= take;
        if (ctx->buflen == 64) {
            vibe_sha256_transform(ctx->state, ctx->buf);
            ctx->buflen = 0;
        }
    }
}

static void vibe_sha256_final(vibe_sha256_ctx *ctx, uint8_t hash[32]) {
    uint64_t orig_bits = ctx->bitcount;
    uint8_t pad[128];
    size_t n = ctx->buflen;
    memcpy(pad, ctx->buf, n);
    pad[n++] = 0x80;
    while ((n % 64) != 56) {
        pad[n++] = 0;
    }
    for (int i = 7; i >= 0; i--) {
        pad[n++] = (uint8_t)((orig_bits >> (i * 8)) & 0xff);
    }
    for (size_t off = 0; off < n; off += 64) {
        vibe_sha256_transform(ctx->state, pad + off);
    }
    for (int i = 0; i < 8; i++) {
        hash[i * 4] = (uint8_t)(ctx->state[i] >> 24);
        hash[i * 4 + 1] = (uint8_t)(ctx->state[i] >> 16);
        hash[i * 4 + 2] = (uint8_t)(ctx->state[i] >> 8);
        hash[i * 4 + 3] = (uint8_t)(ctx->state[i]);
    }
}

static char *vibe_crypto_bytes_to_hex_lower(const uint8_t *buf, size_t len) {
    static const char hex[] = "0123456789abcdef";
    char *out = (char *)calloc(len * 2 + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("vibe_crypto_bytes_to_hex_lower: allocation failed");
    }
    for (size_t i = 0; i < len; i++) {
        out[i * 2] = hex[(buf[i] >> 4) & 0x0f];
        out[i * 2 + 1] = hex[buf[i] & 0x0f];
    }
    return out;
}

char *vibe_crypto_sha256(const char *data) {
    const unsigned char *p = (const unsigned char *)(data == NULL ? "" : data);
    size_t len = strlen((const char *)p);
    vibe_sha256_ctx ctx;
    vibe_sha256_init(&ctx);
    vibe_sha256_update(&ctx, p, len);
    uint8_t h[32];
    vibe_sha256_final(&ctx, h);
    return vibe_crypto_bytes_to_hex_lower(h, 32);
}

/*
 * Same algorithm as vibe_crypto_sha256, but the input length comes from the
 * vibe_bytes struct's explicit `len` field rather than strlen, so an embedded
 * 0x00 byte is hashed like any other byte instead of truncating the input. A
 * NULL handle is treated as zero-length Bytes, matching the published SHA-256
 * digest of empty input.
 */
char *vibe_crypto_sha256_bytes(void *handle) {
    const vibe_bytes *b = (const vibe_bytes *)handle;
    int64_t len = b == NULL ? 0 : b->len;
    const uint8_t *p = b == NULL ? NULL : b->data;
    vibe_sha256_ctx ctx;
    vibe_sha256_init(&ctx);
    vibe_sha256_update(&ctx, p, (size_t)len);
    uint8_t h[32];
    vibe_sha256_final(&ctx, h);
    return vibe_crypto_bytes_to_hex_lower(h, 32);
}

char *vibe_crypto_hmac_sha256(const char *key, const char *data) {
    const char *k = key == NULL ? "" : key;
    const char *d = data == NULL ? "" : data;
    size_t key_len = strlen(k);
    size_t data_len = strlen(d);
    uint8_t key_block[64];
    memset(key_block, 0, sizeof(key_block));
    if (key_len > 64) {
        vibe_sha256_ctx kctx;
        vibe_sha256_init(&kctx);
        vibe_sha256_update(&kctx, (const uint8_t *)k, key_len);
        uint8_t kh[32];
        vibe_sha256_final(&kctx, kh);
        memcpy(key_block, kh, 32);
        key_len = 32;
    } else {
        memcpy(key_block, k, key_len);
    }
    uint8_t ipad[64];
    uint8_t opad[64];
    for (int i = 0; i < 64; i++) {
        ipad[i] = (uint8_t)(key_block[i] ^ 0x36u);
        opad[i] = (uint8_t)(key_block[i] ^ 0x5cu);
    }
    vibe_sha256_ctx ictx;
    vibe_sha256_init(&ictx);
    vibe_sha256_update(&ictx, ipad, 64);
    vibe_sha256_update(&ictx, (const uint8_t *)d, data_len);
    uint8_t ihash[32];
    vibe_sha256_final(&ictx, ihash);

    vibe_sha256_ctx octx;
    vibe_sha256_init(&octx);
    vibe_sha256_update(&octx, opad, 64);
    vibe_sha256_update(&octx, ihash, 32);
    uint8_t ohash[32];
    vibe_sha256_final(&octx, ohash);
    return vibe_crypto_bytes_to_hex_lower(ohash, 32);
}

static int vibe_crypto_fill_random(uint8_t *buf, size_t n) {
#if defined(__linux__) || defined(__APPLE__)
    size_t off = 0;
    while (off < n) {
        size_t chunk = n - off;
        if (chunk > 256) {
            chunk = 256;
        }
        if (getentropy(buf + off, chunk) != 0) {
            return -1;
        }
        off += chunk;
    }
    return 0;
#elif defined(_WIN32)
    {
        NTSTATUS st = BCryptGenRandom(NULL, buf, (ULONG)n, BCRYPT_USE_SYSTEM_PREFERRED_RNG);
        return NT_SUCCESS(st) ? 0 : -1;
    }
#else
    (void)buf;
    (void)n;
    return -1;
#endif
}

char *vibe_crypto_random_bytes(int64_t n) {
    if (n <= 0) {
        return vibe_strdup_or_panic("");
    }
    if (n > (int64_t)(16 * 1024 * 1024)) {
        vibe_panic("vibe_crypto_random_bytes: n too large");
    }
    size_t len = (size_t)n;
    uint8_t *raw = (uint8_t *)malloc(len);
    if (raw == NULL) {
        vibe_panic("vibe_crypto_random_bytes: allocation failed");
    }
    if (vibe_crypto_fill_random(raw, len) != 0) {
        free(raw);
        vibe_panic("vibe_crypto_random_bytes: CSPRNG failed");
    }
    char *hex = vibe_crypto_bytes_to_hex_lower(raw, len);
    memset(raw, 0, len);
    free(raw);
    return hex;
}

char *vibe_crypto_uuid_v4(void) {
    uint8_t b[16];
    if (vibe_crypto_fill_random(b, 16) != 0) {
        vibe_panic("vibe_crypto_uuid_v4: CSPRNG failed");
    }
    b[6] = (uint8_t)((b[6] & 0x0fu) | 0x40u);
    b[8] = (uint8_t)((b[8] & 0x3fu) | 0x80u);
    static const char hex[] = "0123456789abcdef";
    char *out = (char *)calloc(37, sizeof(char));
    if (out == NULL) {
        vibe_panic("vibe_crypto_uuid_v4: allocation failed");
    }
    size_t k = 0;
    for (int i = 0; i < 16; i++) {
        if (i == 4 || i == 6 || i == 8 || i == 10) {
            out[k++] = '-';
        }
        out[k++] = hex[(b[i] >> 4) & 0x0f];
        out[k++] = hex[b[i] & 0x0f];
    }
    out[36] = '\0';
    return out;
}

int64_t vibe_crypto_constant_time_eq(const char *a, const char *b) {
    const unsigned char *pa = (const unsigned char *)(a == NULL ? "" : a);
    const unsigned char *pb = (const unsigned char *)(b == NULL ? "" : b);
    size_t la = strlen((const char *)pa);
    size_t lb = strlen((const char *)pb);
    if (la != lb) {
        return 0;
    }
    volatile uint8_t diff = 0;
    for (size_t i = 0; i < la; i++) {
        diff |= (uint8_t)(pa[i] ^ pb[i]);
    }
    return diff == 0 ? 1 : 0;
}

/* -------------------------------------------------------------------------- */
/* std.ws: SHA-1 (minimal, RFC 3174 style) + WebSocket server framing        */
/* -------------------------------------------------------------------------- */

typedef struct {
    uint64_t bitlen;
    uint32_t h[5];
    uint8_t buf[64];
    size_t buflen;
} vibe_sha1_ctx;

#define VIBE_SHA1_ROL(x, n) (((uint32_t)(x) << (n)) | ((uint32_t)(x) >> (32 - (n))))

static void vibe_sha1_transform(uint32_t state[5], const uint8_t block[64]) {
    uint32_t w[80];
    int i;
    for (i = 0; i < 16; i++) {
        w[i] = ((uint32_t)block[i * 4] << 24) | ((uint32_t)block[i * 4 + 1] << 16) |
               ((uint32_t)block[i * 4 + 2] << 8) | ((uint32_t)block[i * 4 + 3]);
    }
    for (; i < 80; i++) {
        uint32_t x = w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16];
        w[i] = VIBE_SHA1_ROL(x, 1);
    }
    uint32_t a = state[0];
    uint32_t b = state[1];
    uint32_t c = state[2];
    uint32_t d = state[3];
    uint32_t e = state[4];
    for (i = 0; i < 80; i++) {
        uint32_t f;
        uint32_t k;
        if (i < 20) {
            f = (b & c) | ((~b) & d);
            k = 0x5a827999u;
        } else if (i < 40) {
            f = b ^ c ^ d;
            k = 0x6ed9eba1u;
        } else if (i < 60) {
            f = (b & c) | (b & d) | (c & d);
            k = 0x8f1bbcdcu;
        } else {
            f = b ^ c ^ d;
            k = 0xca62c1d6u;
        }
        uint32_t temp = VIBE_SHA1_ROL(a, 5) + f + e + k + w[i];
        e = d;
        d = c;
        c = VIBE_SHA1_ROL(b, 30);
        b = a;
        a = temp;
    }
    state[0] += a;
    state[1] += b;
    state[2] += c;
    state[3] += d;
    state[4] += e;
}

static void vibe_sha1_init(vibe_sha1_ctx *ctx) {
    ctx->bitlen = 0;
    ctx->h[0] = 0x67452301u;
    ctx->h[1] = 0xefcdab89u;
    ctx->h[2] = 0x98badcfeu;
    ctx->h[3] = 0x10325476u;
    ctx->h[4] = 0xc3d2e1f0u;
    ctx->buflen = 0;
}

static void vibe_sha1_update(vibe_sha1_ctx *ctx, const uint8_t *data, size_t len) {
    ctx->bitlen += (uint64_t)len * 8;
    while (len > 0) {
        size_t space = 64 - ctx->buflen;
        size_t take = len < space ? len : space;
        memcpy(ctx->buf + ctx->buflen, data, take);
        ctx->buflen += take;
        data += take;
        len -= take;
        if (ctx->buflen == 64) {
            vibe_sha1_transform(ctx->h, ctx->buf);
            ctx->buflen = 0;
        }
    }
}

static void vibe_sha1_final(vibe_sha1_ctx *ctx, uint8_t out[20]) {
    uint64_t orig_bits = ctx->bitlen;
    uint8_t pad[128];
    size_t n = ctx->buflen;
    memcpy(pad, ctx->buf, n);
    pad[n++] = 0x80;
    while ((n % 64) != 56) {
        pad[n++] = 0;
    }
    for (int i = 7; i >= 0; i--) {
        pad[n++] = (uint8_t)((orig_bits >> (i * 8)) & 0xff);
    }
    for (size_t off = 0; off < n; off += 64) {
        vibe_sha1_transform(ctx->h, pad + off);
    }
    for (int i = 0; i < 5; i++) {
        out[i * 4] = (uint8_t)(ctx->h[i] >> 24);
        out[i * 4 + 1] = (uint8_t)(ctx->h[i] >> 16);
        out[i * 4 + 2] = (uint8_t)(ctx->h[i] >> 8);
        out[i * 4 + 3] = (uint8_t)(ctx->h[i]);
    }
}

static char *vibe_ws_base64_encode_bytes(const uint8_t *data, size_t len) {
    static const char b64[] =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    size_t out_len = ((len + 2) / 3) * 4;
    char *out = (char *)calloc(out_len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("vibe_ws_base64_encode_bytes: allocation failed");
    }
    size_t in_idx = 0;
    size_t out_idx = 0;
    while (in_idx < len) {
        size_t rem = len - in_idx;
        uint32_t octet_a = data[in_idx++];
        uint32_t octet_b = rem > 1 ? data[in_idx++] : 0;
        uint32_t octet_c = rem > 2 ? data[in_idx++] : 0;
        uint32_t triple = (octet_a << 16) | (octet_b << 8) | octet_c;
        out[out_idx++] = b64[(triple >> 18) & 0x3f];
        out[out_idx++] = b64[(triple >> 12) & 0x3f];
        out[out_idx++] = rem > 1 ? b64[(triple >> 6) & 0x3f] : '=';
        out[out_idx++] = rem > 2 ? b64[triple & 0x3f] : '=';
    }
    out[out_len] = '\0';
    return out;
}

#ifndef _WIN32
static int vibe_ws_recv_exact(int fd, void *buf, size_t n) {
    uint8_t *p = (uint8_t *)buf;
    size_t got = 0;
    while (got < n) {
        ssize_t r = recv(fd, (char *)(p + got), n - got, 0);
        if (r <= 0) {
            return -1;
        }
        got += (size_t)r;
    }
    return 0;
}

static int vibe_ws_send_exact(int fd, const void *buf, size_t n) {
    const uint8_t *p = (const uint8_t *)buf;
    size_t total = 0;
    while (total < n) {
        ssize_t w = send(fd, (const char *)(p + total), n - total, 0);
        if (w <= 0) {
            return -1;
        }
        total += (size_t)w;
    }
    return 0;
}

static int vibe_ws_copy_header_value(const char *req, const char *hdr, char *out, size_t cap) {
    const char *line_start = req == NULL ? "" : req;
    size_t hn = strlen(hdr);
    while (*line_start != '\0') {
        const char *nl = strchr(line_start, '\n');
        const char *line_end = nl != NULL ? nl : line_start + strlen(line_start);
        const char *s = line_start;
        size_t slen = (size_t)(line_end - s);
        while (slen > 0 && s[slen - 1] == '\r') {
            slen -= 1;
        }
        if (slen >= hn && strncasecmp(s, hdr, hn) == 0) {
            const char *v = s + hn;
            while (v < s + slen && (*v == ' ' || *v == '\t')) {
                v += 1;
            }
            size_t vl = (size_t)(s + slen - v);
            if (vl == 0 || vl >= cap) {
                return -1;
            }
            memcpy(out, v, vl);
            out[vl] = '\0';
            return 0;
        }
        if (nl == NULL) {
            break;
        }
        line_start = nl + 1;
    }
    return -1;
}
#endif

int64_t vibe_ws_upgrade(int64_t conn_raw, const char *raw_request) {
#ifdef _WIN32
    (void)conn_raw;
    (void)raw_request;
    return 0;
#else
    int fd = vibe_net_checked_fd(conn_raw);
    if (fd <= 0) {
        return 0;
    }
    char key_buf[160];
    if (vibe_ws_copy_header_value(raw_request, "Sec-WebSocket-Key:", key_buf, sizeof(key_buf)) != 0) {
        return 0;
    }
    static const char magic[] = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    size_t key_len = strlen(key_buf);
    size_t magic_len = sizeof(magic) - 1;
    uint8_t *concat = (uint8_t *)malloc(key_len + magic_len + 1);
    if (concat == NULL) {
        return 0;
    }
    memcpy(concat, key_buf, key_len);
    memcpy(concat + key_len, magic, magic_len);

    vibe_sha1_ctx sha1;
    vibe_sha1_init(&sha1);
    vibe_sha1_update(&sha1, concat, key_len + magic_len);
    free(concat);

    uint8_t digest[20];
    vibe_sha1_final(&sha1, digest);
    char *accept_b64 = vibe_ws_base64_encode_bytes(digest, 20);

    char response[512];
    int nw = snprintf(response, sizeof(response),
                      "HTTP/1.1 101 Switching Protocols\r\n"
                      "Upgrade: websocket\r\n"
                      "Connection: Upgrade\r\n"
                      "Sec-WebSocket-Accept: %s\r\n"
                      "\r\n",
                      accept_b64);
    free(accept_b64);
    if (nw <= 0 || (size_t)nw >= sizeof(response)) {
        return 0;
    }
    if (vibe_ws_send_exact(fd, response, (size_t)nw) != 0) {
        return 0;
    }
    return 1;
#endif
}

char *vibe_ws_read_frame(int64_t conn_raw) {
#ifdef _WIN32
    (void)conn_raw;
    return vibe_strdup_or_panic("");
#else
    int fd = vibe_net_checked_fd(conn_raw);
    if (fd <= 0) {
        return vibe_strdup_or_panic("");
    }
    const uint64_t max_payload = 4ull * 1024ull * 1024ull;
    uint8_t hdr[2];
    if (vibe_ws_recv_exact(fd, hdr, 2) != 0) {
        return vibe_strdup_or_panic("");
    }
    int fin = (hdr[0] & 0x80) != 0;
    (void)fin;
    uint8_t opcode = (uint8_t)(hdr[0] & 0x0f);
    int masked = (hdr[1] & 0x80) != 0;
    uint64_t payload_len = (uint64_t)(hdr[1] & 0x7f);
    if (payload_len == 126) {
        uint8_t ext[2];
        if (vibe_ws_recv_exact(fd, ext, 2) != 0) {
            return vibe_strdup_or_panic("");
        }
        payload_len = ((uint64_t)ext[0] << 8) | (uint64_t)ext[1];
    } else if (payload_len == 127) {
        uint8_t ext[8];
        if (vibe_ws_recv_exact(fd, ext, 8) != 0) {
            return vibe_strdup_or_panic("");
        }
        if ((ext[0] & 0x80) != 0) {
            return vibe_strdup_or_panic("");
        }
        payload_len = 0;
        for (int i = 0; i < 8; i++) {
            payload_len = (payload_len << 8) | (uint64_t)ext[i];
        }
    }
    if (payload_len > max_payload) {
        return vibe_strdup_or_panic("");
    }
    uint8_t mask_key[4] = {0, 0, 0, 0};
    if (masked) {
        if (vibe_ws_recv_exact(fd, mask_key, 4) != 0) {
            return vibe_strdup_or_panic("");
        }
    } else {
        /* Client frames must be masked (RFC 6455). */
        return vibe_strdup_or_panic("");
    }
    size_t pl = (size_t)payload_len;
    uint8_t *rawpl = (uint8_t *)calloc(pl + 1, sizeof(uint8_t));
    if (rawpl == NULL) {
        vibe_panic("vibe_ws_read_frame: payload alloc failed");
    }
    if (pl > 0 && vibe_ws_recv_exact(fd, rawpl, pl) != 0) {
        free(rawpl);
        return vibe_strdup_or_panic("");
    }
    for (size_t i = 0; i < pl; i++) {
        rawpl[i] = (uint8_t)(rawpl[i] ^ mask_key[i % 4]);
    }
    if (opcode == 0x8) {
        free(rawpl);
        return vibe_strdup_or_panic("");
    }
    if (opcode != 0x1 && opcode != 0x0) {
        free(rawpl);
        return vibe_strdup_or_panic("");
    }
    rawpl[pl] = '\0';
    return (char *)rawpl;
#endif
}

void vibe_ws_write_frame(int64_t conn_raw, const char *data) {
#ifdef _WIN32
    (void)conn_raw;
    (void)data;
#else
    int fd = vibe_net_checked_fd(conn_raw);
    if (fd <= 0) {
        return;
    }
    const unsigned char *raw = (const unsigned char *)(data == NULL ? "" : data);
    size_t len = strlen((const char *)raw);
    uint8_t header[14];
    size_t hlen = 0;
    header[hlen++] = (uint8_t)(0x80 | 0x1);
    if (len < 126) {
        header[hlen++] = (uint8_t)len;
    } else if (len < 65536) {
        header[hlen++] = 126;
        header[hlen++] = (uint8_t)((len >> 8) & 0xff);
        header[hlen++] = (uint8_t)(len & 0xff);
    } else {
        header[hlen++] = 127;
        for (int i = 7; i >= 0; i--) {
            header[hlen++] = (uint8_t)((len >> (i * 8)) & 0xff);
        }
    }
    if (vibe_ws_send_exact(fd, header, hlen) != 0) {
        return;
    }
    if (len > 0 && vibe_ws_send_exact(fd, raw, len) != 0) {
        return;
    }
#endif
}

void vibe_ws_close_frame(int64_t conn_raw) {
#ifdef _WIN32
    (void)conn_raw;
#else
    int fd = vibe_net_checked_fd(conn_raw);
    if (fd <= 0) {
        return;
    }
    uint8_t frame[2] = {0x88, 0x00};
    (void)vibe_ws_send_exact(fd, frame, 2);
#endif
}
