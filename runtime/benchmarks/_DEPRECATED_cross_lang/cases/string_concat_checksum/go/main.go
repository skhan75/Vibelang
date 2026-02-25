package main

import (
  "fmt"
  "strconv"
)

func main() {
  iterations := int64(50000)
  checksum := int64(0)
  for i := int64(0); i < iterations; i++ {
    si := strconv.FormatInt(i, 10)
    sj := strconv.FormatInt(i+7, 10)
    pi, _ := strconv.ParseInt(si, 10, 64)
    pj, _ := strconv.ParseInt(sj, 10, 64)
    checksum += pi + pj
  }
  ops := iterations
  fmt.Println("RESULT")
  fmt.Println(checksum)
  fmt.Println(ops)
}
