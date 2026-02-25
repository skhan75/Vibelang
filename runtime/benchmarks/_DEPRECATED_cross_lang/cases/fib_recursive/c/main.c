#include <stdio.h>
#include <stdint.h>

int main(void) {
  int64_t n = 200000;
  int64_t a = 0;
  int64_t b = 1;
  for (int64_t i = 0; i < n; i++) {
    int64_t next = a + b;
    if (next > 1000000000) {
      next = next - 1000000000;
    }
    a = b;
    b = next;
  }
  int64_t checksum = b;
  int64_t ops = n;
  printf("RESULT\n");
  printf("%ld\n", (long)checksum);
  printf("%ld\n", (long)ops);
  return 0;
}
