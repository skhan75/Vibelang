#include <stdio.h>
#include <stdint.h>
#include <pthread.h>

typedef struct {
  int64_t val;
  pthread_mutex_t mu;
  pthread_cond_t has_val;
  pthread_cond_t consumed;
  int full;
} chan_t;

static void chan_send(chan_t *c, int64_t v) {
  pthread_mutex_lock(&c->mu);
  while (c->full)
    pthread_cond_wait(&c->consumed, &c->mu);
  c->val = v;
  c->full = 1;
  pthread_cond_signal(&c->has_val);
  pthread_mutex_unlock(&c->mu);
}

static int64_t chan_recv(chan_t *c) {
  pthread_mutex_lock(&c->mu);
  while (!c->full)
    pthread_cond_wait(&c->has_val, &c->mu);
  int64_t v = c->val;
  c->full = 0;
  pthread_cond_signal(&c->consumed);
  pthread_mutex_unlock(&c->mu);
  return v;
}

typedef struct {
  chan_t *req;
  chan_t *resp;
  int64_t rounds;
} worker_args_t;

static void *pingpong_server(void *arg) {
  worker_args_t *a = (worker_args_t *)arg;
  for (int64_t r = 0; r < a->rounds; r++) {
    int64_t token = chan_recv(a->req);
    chan_send(a->resp, token + 1);
  }
  return NULL;
}

int main(void) {
  int64_t rounds = 50000;
  chan_t req = {.val = 0, .mu = PTHREAD_MUTEX_INITIALIZER,
                .has_val = PTHREAD_COND_INITIALIZER,
                .consumed = PTHREAD_COND_INITIALIZER, .full = 0};
  chan_t resp = {.val = 0, .mu = PTHREAD_MUTEX_INITIALIZER,
                 .has_val = PTHREAD_COND_INITIALIZER,
                 .consumed = PTHREAD_COND_INITIALIZER, .full = 0};
  worker_args_t args = {.req = &req, .resp = &resp, .rounds = rounds};
  pthread_t th;
  pthread_create(&th, NULL, pingpong_server, &args);
  int64_t checksum = 0;
  int64_t token = 1;
  for (int64_t r = 0; r < rounds; r++) {
    chan_send(&req, token);
    int64_t reply = chan_recv(&resp);
    checksum += reply;
    token = reply;
  }
  pthread_join(th, NULL);
  int64_t ops = rounds;
  printf("RESULT\n");
  printf("%ld\n", (long)checksum);
  printf("%ld\n", (long)ops);
  return 0;
}
