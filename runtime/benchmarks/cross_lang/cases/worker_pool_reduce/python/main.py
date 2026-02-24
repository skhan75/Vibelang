#!/usr/bin/env python3
import threading

def worker(start, step, limit, result_list, index):
    local_sum = 0
    i = start
    while i < limit:
        local_sum += i + 1
        i += step
    result_list[index] = local_sum

workers_count = 4
limit = 60000
partials = [0] * workers_count
threads = []
for w in range(workers_count):
    t = threading.Thread(target=worker, args=(w, workers_count, limit, partials, w))
    threads.append(t)
    t.start()
for t in threads:
    t.join()
checksum = sum(partials)
ops = limit
print("RESULT")
print(checksum)
print(ops)
