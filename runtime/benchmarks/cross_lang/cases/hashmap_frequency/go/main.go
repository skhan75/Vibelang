package main

import "fmt"

func main() {
  iterations := int64(200000)
  buckets := int64(257)
  freq := make(map[int64]int64)
  for i := int64(0); i < iterations; i++ {
    k := i - (i/buckets)*buckets
    freq[k]++
  }
  checksum := int64(0)
  for k := int64(0); k < buckets; k++ {
    checksum += freq[k] * (k + 1)
  }
  ops := iterations
  fmt.Println("RESULT")
  fmt.Println(checksum)
  fmt.Println(ops)
}
