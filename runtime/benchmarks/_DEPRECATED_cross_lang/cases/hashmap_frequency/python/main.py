#!/usr/bin/env python3
iterations = 200000
buckets = 257
freq = {}
for i in range(iterations):
    k = i - (i // buckets) * buckets
    freq[k] = freq.get(k, 0) + 1
checksum = 0
for k in range(buckets):
    checksum += freq.get(k, 0) * (k + 1)
ops = iterations
print("RESULT")
print(checksum)
print(ops)
