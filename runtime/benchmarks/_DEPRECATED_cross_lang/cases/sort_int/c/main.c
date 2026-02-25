#include <stdio.h>
#include <stdint.h>

int main(void) {
  int64_t size = 120000;
  int64_t x = 17;
  int64_t top1 = 0;
  int64_t top2 = 0;
  int64_t top3 = 0;
  int64_t top4 = 0;
  for (int64_t i = 0; i < size; i++) {
    x = x * 73 + 19;
    if (x > 100000)
      x = x - 100000;
    if (x > top1) {
      top4 = top3;
      top3 = top2;
      top2 = top1;
      top1 = x;
    } else if (x > top2) {
      top4 = top3;
      top3 = top2;
      top2 = x;
    } else if (x > top3) {
      top4 = top3;
      top3 = x;
    } else if (x > top4) {
      top4 = x;
    }
  }
  int64_t checksum = top1 + top2 + top3 + top4;
  int64_t ops = size;
  printf("RESULT\n");
  printf("%ld\n", (long)checksum);
  printf("%ld\n", (long)ops);
  return 0;
}
