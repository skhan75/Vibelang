package main

import "fmt"

func isPrime(n int64) bool {
  d := int64(2)
  for d*d <= n {
    rem := n - (n/d)*d
    if rem == 0 {
      return false
    }
    d++
  }
  return true
}

func main() {
  limit := int64(12000)
  count := int64(0)
  sum := int64(0)
  n := int64(2)
  for n <= limit {
    if isPrime(n) {
      count++
      sum += n
    }
    n++
  }
  checksum := count*1000000 + sum
  ops := limit
  fmt.Println("RESULT")
  fmt.Println(checksum)
  fmt.Println(ops)
}
