#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>

static bool is_prime(int64_t n) {
  for (int64_t d = 2; d * d <= n; d++) {
    int64_t rem = n - (n / d) * d;
    if (rem == 0)
      return false;
  }
  return true;
}

int main(void) {
  int64_t limit = 12000;
  int64_t count = 0;
  int64_t sum = 0;
  for (int64_t n = 2; n <= limit; n++) {
    if (is_prime(n)) {
      count++;
      sum += n;
    }
  }
  int64_t checksum = count * 1000000 + sum;
  int64_t ops = limit;
  printf("RESULT\n");
  printf("%ld\n", (long)checksum);
  printf("%ld\n", (long)ops);
  return 0;
}
