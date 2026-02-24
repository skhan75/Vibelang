#!/usr/bin/env python3
iterations = 50000
checksum = 0
for i in range(iterations):
    si = str(i)
    sj = str(i + 7)
    pi = int(si)
    pj = int(sj)
    checksum += pi + pj
ops = iterations
print("RESULT")
print(checksum)
print(ops)
