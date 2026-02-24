#include <stdio.h>
#include <stdint.h>
#include <pthread.h>

typedef struct {
  int64_t start;
  int64_t step;
  int64_t limit;
  int64_t *partial;
} worker_args_t;

static void *worker(void *arg) {
  worker_args_t *a = (worker_args_t *)arg;
  int64_t local_sum = 0;
  for (int64_t i = a->start; i < a->limit; i += a->step) {
    local_sum += (i + 1);
  }
  *a->partial = local_sum;
  return NULL;
}

int main(void) {
  int64_t workers = 4;
  int64_t limit = 60000;
  int64_t partials[4];
  pthread_t th[4];
  worker_args_t args[4];
  for (int64_t w = 0; w < workers; w++) {
    args[w] = (worker_args_t){
        .start = w, .step = workers, .limit = limit, .partial = &partials[w]};
    pthread_create(&th[w], NULL, worker, &args[w]);
  }
  int64_t checksum = 0;
  for (int64_t w = 0; w < workers; w++) {
    pthread_join(th[w], NULL);
    checksum += partials[w];
  }
  int64_t ops = limit;
  printf("RESULT\n");
  printf("%ld\n", (long)checksum);
  printf("%ld\n", (long)ops);
  return 0;
}
