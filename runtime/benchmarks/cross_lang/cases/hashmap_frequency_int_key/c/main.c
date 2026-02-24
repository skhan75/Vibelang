#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define BUCKETS 257
#define EMPTY -2
#define TOMB  -1

typedef struct {
  int64_t key;
  int64_t val;
} slot_t;

static void hm_init(slot_t *slots) {
  for (int i = 0; i < BUCKETS; i++)
    slots[i].key = EMPTY;
}

static void hm_inc(slot_t *slots, int64_t k) {
  int idx = (int)(k % BUCKETS);
  if (idx < 0)
    idx += BUCKETS;
  for (int i = 0; i < BUCKETS; i++) {
    int j = (idx + i) % BUCKETS;
    if (slots[j].key == EMPTY || slots[j].key == TOMB) {
      slots[j].key = k;
      slots[j].val = 1;
      return;
    }
    if (slots[j].key == k) {
      slots[j].val++;
      return;
    }
  }
}

static int64_t hm_get(slot_t *slots, int64_t k) {
  int idx = (int)(k % BUCKETS);
  if (idx < 0)
    idx += BUCKETS;
  for (int i = 0; i < BUCKETS; i++) {
    int j = (idx + i) % BUCKETS;
    if (slots[j].key == EMPTY)
      return 0;
    if (slots[j].key == k)
      return slots[j].val;
  }
  return 0;
}

int main(void) {
  int64_t iterations = 200000;
  int64_t buckets = 257;
  slot_t *slots = (slot_t *)calloc(BUCKETS, sizeof(slot_t));
  hm_init(slots);
  for (int64_t i = 0; i < iterations; i++) {
    int64_t k = i - (i / buckets) * buckets;
    hm_inc(slots, k);
  }
  int64_t checksum = 0;
  for (int64_t k = 0; k < buckets; k++) {
    checksum += hm_get(slots, k) * (k + 1);
  }
  int64_t ops = iterations;
  printf("RESULT\n");
  printf("%ld\n", (long)checksum);
  printf("%ld\n", (long)ops);
  free(slots);
  return 0;
}
