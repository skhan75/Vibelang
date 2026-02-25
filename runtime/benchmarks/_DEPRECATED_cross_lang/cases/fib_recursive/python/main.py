#!/usr/bin/env python3
n = 200000
a = 0
b = 1
for i in range(n):
    next_val = a + b
    if next_val > 1000000000:
        next_val = next_val - 1000000000
    a = b
    b = next_val
checksum = b
ops = n
print("RESULT")
print(checksum)
print(ops)
