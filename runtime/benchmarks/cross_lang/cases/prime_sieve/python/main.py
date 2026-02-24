#!/usr/bin/env python3
def is_prime(n):
    d = 2
    while d * d <= n:
        rem = n - (n // d) * d
        if rem == 0:
            return False
        d += 1
    return True

limit = 12000
count = 0
total_sum = 0
for n in range(2, limit + 1):
    if is_prime(n):
        count += 1
        total_sum += n
checksum = count * 1000000 + total_sum
ops = limit
print("RESULT")
print(checksum)
print(ops)
