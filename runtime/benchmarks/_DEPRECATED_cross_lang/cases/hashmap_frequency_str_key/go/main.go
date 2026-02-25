package main

import (
  "fmt"
  "strconv"
)

func main() {
  iterations := int64(200000)
  buckets := int64(257)
  freq := make(map[string]int64)
  for i := int64(0); i < iterations; i++ {
    k := i - (i/buckets)*buckets
    key := strconv.FormatInt(k, 10)
    freq[key]++
  }
  checksum := int64(0)
  for k := int64(0); k < buckets; k++ {
    key := strconv.FormatInt(k, 10)
    checksum += freq[key] * (k + 1)
  }
  ops := iterations
  fmt.Println("RESULT")
  fmt.Println(checksum)
  fmt.Println(ops)
}
