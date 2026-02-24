#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define BUF_SZ 24

static void append_int(char *buf, int64_t v) {
  if (v == 0) {
    buf[0] = '0';
    buf[1] = '\0';
    return;
  }
  if (v < 0) {
    buf[0] = '-';
    append_int(buf + 1, -v);
    return;
  }
  char tmp[BUF_SZ];
  int len = 0;
  int64_t n = v;
  while (n > 0) {
    tmp[len++] = (char)('0' + (n % 10));
    n /= 10;
  }
  int i = 0;
  while (len > 0) {
    buf[i++] = tmp[--len];
  }
  buf[i] = '\0';
}

static int64_t parse_int(const char *s) {
  return (int64_t)strtoll(s, NULL, 10);
}

int main(void) {
  int64_t iterations = 50000;
  int64_t checksum = 0;
  char si[BUF_SZ], sj[BUF_SZ];
  for (int64_t i = 0; i < iterations; i++) {
    append_int(si, i);
    append_int(sj, i + 7);
    int64_t pi = parse_int(si);
    int64_t pj = parse_int(sj);
    checksum += pi + pj;
  }
  int64_t ops = iterations;
  printf("RESULT\n");
  printf("%ld\n", (long)checksum);
  printf("%ld\n", (long)ops);
  return 0;
}
