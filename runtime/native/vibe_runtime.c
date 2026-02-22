#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <strings.h>
#include <ctype.h>
#include <errno.h>
#include <pthread.h>
#include <sys/stat.h>
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

enum {
    VIBE_CONTAINER_LIST_I64 = 1,
    VIBE_CONTAINER_MAP_I64_I64 = 2,
    VIBE_CONTAINER_MAP_STR_I64 = 3,
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
} vibe_map_str_i64;

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
    for (int64_t i = 0; i < map->len; i++) {
        if (map->entries[i].key == key) {
            map->entries[i].value = value;
            return 0;
        }
    }
    vibe_map_i64_ensure_capacity(map, map->len + 1);
    map->entries[map->len].key = key;
    map->entries[map->len].value = value;
    map->len += 1;
    return 0;
}

int64_t vibe_map_get_i64_i64(void *handle, int64_t key) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map.get(i64) called on non-map handle");
    }
    for (int64_t i = 0; i < map->len; i++) {
        if (map->entries[i].key == key) {
            return map->entries[i].value;
        }
    }
    return 0;
}

int64_t vibe_map_contains_i64_i64(void *handle, int64_t key) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map.contains(i64) called on non-map handle");
    }
    for (int64_t i = 0; i < map->len; i++) {
        if (map->entries[i].key == key) {
            return 1;
        }
    }
    return 0;
}

int64_t vibe_map_remove_i64_i64(void *handle, int64_t key) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map.remove(i64) called on non-map handle");
    }
    for (int64_t i = 0; i < map->len; i++) {
        if (map->entries[i].key == key) {
            for (int64_t j = i + 1; j < map->len; j++) {
                map->entries[j - 1] = map->entries[j];
            }
            map->len -= 1;
            return 1;
        }
    }
    return 0;
}

int64_t vibe_map_len_i64_i64(void *handle) {
    vibe_map_i64_i64 *map = (vibe_map_i64_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_I64_I64) {
        vibe_panic("map.len called on non-map handle");
    }
    return map->len;
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
    for (int64_t i = 0; i < map->len; i++) {
        if (strcmp(map->entries[i].key, safe_key) == 0) {
            map->entries[i].value = value;
            return 0;
        }
    }
    vibe_map_str_ensure_capacity(map, map->len + 1);
    map->entries[map->len].key = vibe_strdup_or_panic(safe_key);
    map->entries[map->len].value = value;
    map->len += 1;
    return 0;
}

int64_t vibe_map_get_str_i64(void *handle, const char *key) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map.get(Str) called on non-map handle");
    }
    const char *safe_key = key == NULL ? "" : key;
    for (int64_t i = 0; i < map->len; i++) {
        if (strcmp(map->entries[i].key, safe_key) == 0) {
            return map->entries[i].value;
        }
    }
    return 0;
}

int64_t vibe_map_contains_str_i64(void *handle, const char *key) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map.contains(Str) called on non-map handle");
    }
    const char *safe_key = key == NULL ? "" : key;
    for (int64_t i = 0; i < map->len; i++) {
        if (strcmp(map->entries[i].key, safe_key) == 0) {
            return 1;
        }
    }
    return 0;
}

int64_t vibe_map_remove_str_i64(void *handle, const char *key) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map.remove(Str) called on non-map handle");
    }
    const char *safe_key = key == NULL ? "" : key;
    for (int64_t i = 0; i < map->len; i++) {
        if (strcmp(map->entries[i].key, safe_key) == 0) {
            free(map->entries[i].key);
            for (int64_t j = i + 1; j < map->len; j++) {
                map->entries[j - 1] = map->entries[j];
            }
            map->len -= 1;
            return 1;
        }
    }
    return 0;
}

int64_t vibe_map_len_str_i64(void *handle) {
    vibe_map_str_i64 *map = (vibe_map_str_i64 *)handle;
    if (map == NULL || map->tag != VIBE_CONTAINER_MAP_STR_I64) {
        vibe_panic("map.len called on non-map handle");
    }
    return map->len;
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
    vibe_panic("container remove(Str) is only valid for Map<Str, Int>");
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
    vibe_panic("container key_at(Str) is only valid for Map<Str, Int>");
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

int64_t vibe_fs_create_dir(const char *path) {
    if (path == NULL || path[0] == '\0') {
        return 0;
    }
    int rc = mkdir(path, 0777);
    if (rc == 0) {
        return 1;
    }
    return errno == EEXIST ? 1 : 0;
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

int64_t vibe_json_is_valid(const char *raw) {
    if (raw == NULL) {
        return 0;
    }
    const char *start = vibe_trim_start(raw);
    const char *end = vibe_trim_end_ptr(start);
    if (end <= start) {
        return 0;
    }
    size_t len = (size_t)(end - start);
    if ((start[0] == '{' && end[-1] == '}') || (start[0] == '[' && end[-1] == ']') ||
        (start[0] == '"' && end[-1] == '"')) {
        return 1;
    }
    if ((len == 4 && strncmp(start, "true", 4) == 0) ||
        (len == 5 && strncmp(start, "false", 5) == 0) ||
        (len == 4 && strncmp(start, "null", 4) == 0)) {
        return 1;
    }
    char *parse_end = NULL;
    (void)strtoll(start, &parse_end, 10);
    return parse_end == end ? 1 : 0;
}

int64_t vibe_json_parse_i64(const char *raw) {
    if (raw == NULL) {
        return 0;
    }
    const char *start = vibe_trim_start(raw);
    char *parse_end = NULL;
    int64_t value = strtoll(start, &parse_end, 10);
    if (parse_end == start) {
        return 0;
    }
    while (parse_end != NULL && *parse_end != '\0' && isspace((unsigned char)*parse_end)) {
        parse_end += 1;
    }
    if (parse_end != NULL && *parse_end != '\0') {
        return 0;
    }
    return value;
}

char *vibe_json_stringify_i64(int64_t value) {
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%lld", (long long)value);
    return vibe_strdup_or_panic(buffer);
}

char *vibe_json_minify(const char *raw) {
    if (raw == NULL) {
        return vibe_strdup_or_panic("");
    }
    size_t len = strlen(raw);
    char *out = (char *)calloc(len + 1, sizeof(char));
    if (out == NULL) {
        vibe_panic("failed to allocate json.minify output");
    }
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
