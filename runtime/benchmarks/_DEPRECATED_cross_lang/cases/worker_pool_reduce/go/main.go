package main

import "fmt"

func main() {
  workers := int64(4)
  limit := int64(60000)
  out := make(chan int64, workers)
  for w := int64(0); w < workers; w++ {
    go func(start int64) {
      localSum := int64(0)
      for i := start; i < limit; i += workers {
        localSum += i + 1
      }
      out <- localSum
    }(w)
  }
  checksum := int64(0)
  for w := int64(0); w < workers; w++ {
    checksum += <-out
  }
  ops := limit
  fmt.Println("RESULT")
  fmt.Println(checksum)
  fmt.Println(ops)
}
