package main

import (
  "fmt"
)

func main() {
  size := 120000
  x := int64(17)
  top1 := int64(0)
  top2 := int64(0)
  top3 := int64(0)
  top4 := int64(0)
  for i := 0; i < size; i++ {
    x = x*73 + 19
    if x > 100000 {
      x = x - 100000
    }
    if x > top1 {
      top4 = top3
      top3 = top2
      top2 = top1
      top1 = x
    } else if x > top2 {
      top4 = top3
      top3 = top2
      top2 = x
    } else if x > top3 {
      top4 = top3
      top3 = x
    } else if x > top4 {
      top4 = x
    }
  }
  checksum := top1 + top2 + top3 + top4
  ops := size
  fmt.Println("RESULT")
  fmt.Println(checksum)
  fmt.Println(ops)
}
