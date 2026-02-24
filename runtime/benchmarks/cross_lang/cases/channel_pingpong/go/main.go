package main

import "fmt"

func main() {
  rounds := int64(50000)
  req := make(chan int64, 1)
  resp := make(chan int64, 1)
  go func() {
    for r := int64(0); r < rounds; r++ {
      token := <-req
      resp <- token + 1
    }
  }()
  checksum := int64(0)
  token := int64(1)
  for r := int64(0); r < rounds; r++ {
    req <- token
    reply := <-resp
    checksum += reply
    token = reply
  }
  ops := rounds
  fmt.Println("RESULT")
  fmt.Println(checksum)
  fmt.Println(ops)
}
