#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define BUCKETS 257
#define KEY_BUF 32

typedef struct {
  char *key;
  int64_t val;
} slot_t;

static uint32_t hash_str(const char *s) {
  uint32_t h = 5381;
  int c;
  while ((c = (unsigned char)*s++) != '\0')
    h = ((h << 5) + h) + c;
  return h;
}

static void hm_init(slot_t *slots) {
  for (int i = 0; i < BUCKETS; i++)
    slots[i].key = NULL;
}

static void hm_inc(slot_t *slots, const char *key) {
  uint32_t h = hash_str(key);
  int idx = (int)(h % BUCKETS);
  for (int i = 0; i < BUCKETS; i++) {
    int j = (idx + i) % BUCKETS;
    if (slots[j].key == NULL) {
      slots[j].key = strdup(key);
      slots[j].val = 1;
      return;
    }
    if (strcmp(slots[j].key, key) == 0) {
      slots[j].val++;
      return;
    }
  }
}

static int64_t hm_get(slot_t *slots, const char *key) {
  uint32_t h = hash_str(key);
  int idx = (int)(h % BUCKETS);
  for (int i = 0; i < BUCKETS; i++) {
    int j = (idx + i) % BUCKETS;
    if (slots[j].key == NULL)
      return 0;
    if (strcmp(slots[j].key, key) == 0)
      return slots[j].val;
  }
  return 0;
}

static void hm_free(slot_t *slots) {
  for (int i = 0; i < BUCKETS; i++) {
    free(slots[i].key);
    slots[i].key = NULL;
  }
}

int main(void) {
  int64_t iterations = 200000;
  int64_t buckets = 257;
  char key_buf[KEY_BUF];
  slot_t *slots = (slot_t *)calloc(BUCKETS, sizeof(slot_t));
  hm_init(slots);
  for (int64_t i = 0; i < iterations; i++) {
    int64_t k = i - (i / buckets) * buckets;
    snprintf(key_buf, sizeof(key_buf), "%ld", (long)k);
    hm_inc(slots, key_buf);
  }
  int64_t checksum = 0;
  for (int64_t k = 0; k < buckets; k++) {
    snprintf(key_buf, sizeof(key_buf), "%ld", (long)k);
    checksum += hm_get(slots, key_buf) * (k + 1);
  }
  int64_t ops = iterations;
  printf("RESULT\n");
  printf("%ld\n", (long)checksum);
  printf("%ld\n", (long)ops);
  hm_free(slots);
  free(slots);
  return 0;
}
