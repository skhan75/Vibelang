#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <ctype.h>

#define PAYLOAD "{ \"value\" : 12345 }"
#define MIN_BUF 64

static void minify(const char *in, char *out, size_t cap) {
  size_t j = 0;
  for (const char *p = in; *p && j < cap - 1; p++) {
    if (!isspace((unsigned char)*p)) {
      out[j++] = *p;
    }
  }
  out[j] = '\0';
}

static int is_valid_shape(const char *s) {
  if (!s || !*s) return 0;
  const char *start = s;
  int brace = 0;
  for (; *s; s++) {
    if (*s == '{') brace++;
    else if (*s == '}') brace--;
    if (brace < 0) return 0;
  }
  return brace == 0 && strstr(start, "value") != NULL && strstr(start, "12345") != NULL;
}

int main(void) {
  int64_t iterations = 120000;
  const char *payload = PAYLOAD;
  int64_t checksum = 0;
  char buf[MIN_BUF];
  for (int64_t i = 0; i < iterations; i++) {
    minify(payload, buf, sizeof(buf));
    if (is_valid_shape(buf))
      checksum += 12345;
  }
  int64_t ops = iterations;
  printf("RESULT\n");
  printf("%ld\n", (long)checksum);
  printf("%ld\n", (long)ops);
  return 0;
}
