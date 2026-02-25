package main

import "fmt"

func main() {
  n := int64(200000)
  a := int64(0)
  b := int64(1)
  for i := int64(0); i < n; i++ {
    next := a + b
    if next > 1000000000 {
      next -= 1000000000
    }
    a = b
    b = next
  }
  checksum := b
  ops := n
  fmt.Println("RESULT")
  fmt.Println(checksum)
  fmt.Println(ops)
}
