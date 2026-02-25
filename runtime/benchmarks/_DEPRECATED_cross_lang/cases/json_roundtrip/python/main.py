#!/usr/bin/env python3
def minify(s):
    return "".join(c for c in s if not c.isspace())

def is_valid_shape(s):
    if not s:
        return False
    brace = 0
    for c in s:
        if c == "{":
            brace += 1
        elif c == "}":
            brace -= 1
        if brace < 0:
            return False
    return brace == 0 and "value" in s and "12345" in s

iterations = 120000
payload = '{ "value" : 12345 }'
checksum = 0
for i in range(iterations):
    buf = minify(payload)
    if is_valid_shape(buf):
        checksum += 12345
ops = iterations
print("RESULT")
print(checksum)
print(ops)
