#!/usr/bin/env python3
import queue
import threading

def pingpong_server(req_q, resp_q, rounds):
    for r in range(rounds):
        token = req_q.get()
        resp_q.put(token + 1)

rounds = 50000
req_q = queue.Queue(1)
resp_q = queue.Queue(1)
th = threading.Thread(target=pingpong_server, args=(req_q, resp_q, rounds))
th.start()
checksum = 0
token = 1
for r in range(rounds):
    req_q.put(token)
    reply = resp_q.get()
    checksum += reply
    token = reply
th.join()
ops = rounds
print("RESULT")
print(checksum)
print(ops)
