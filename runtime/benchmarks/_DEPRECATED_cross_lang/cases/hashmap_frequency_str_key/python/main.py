#!/usr/bin/env python3
iterations = 200000
buckets = 257
freq = {}
for i in range(iterations):
    k = i - (i // buckets) * buckets
    key = str(k)
    freq[key] = freq.get(key, 0) + 1
checksum = 0
for k in range(buckets):
    key = str(k)
    checksum += freq.get(key, 0) * (k + 1)
ops = iterations
print("RESULT")
print(checksum)
print(ops)
