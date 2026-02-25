#!/usr/bin/env python3
MASK64 = (1 << 64) - 1

def i64(v):
    v = int(v) & MASK64
    return v if v < (1 << 63) else v - (1 << 64)

size = 120000
x = 17
top1 = 0
top2 = 0
top3 = 0
top4 = 0
for i in range(size):
    x = i64(x * 73 + 19)
    if x > 100000:
        x = i64(x - 100000)
    if x > top1:
        top4 = top3
        top3 = top2
        top2 = top1
        top1 = x
    elif x > top2:
        top4 = top3
        top3 = top2
        top2 = x
    elif x > top3:
        top4 = top3
        top3 = x
    elif x > top4:
        top4 = x
checksum = i64(top1 + top2 + top3 + top4)
ops = size
print("RESULT")
print(checksum)
print(ops)
