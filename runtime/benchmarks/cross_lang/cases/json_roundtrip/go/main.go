package main

import (
  "fmt"
  "strings"
)

func minify(s string) string {
  var b strings.Builder
  for _, c := range s {
    if c != ' ' && c != '\t' && c != '\n' && c != '\r' {
      b.WriteRune(c)
    }
  }
  return b.String()
}

func isValidShape(s string) bool {
  if len(s) == 0 {
    return false
  }
  brace := 0
  for _, c := range s {
    if c == '{' {
      brace++
    } else if c == '}' {
      brace--
    }
    if brace < 0 {
      return false
    }
  }
  return brace == 0 && strings.Contains(s, "value") && strings.Contains(s, "12345")
}

func main() {
  iterations := int64(120000)
  payload := "{ \"value\" : 12345 }"
  checksum := int64(0)
  for i := int64(0); i < iterations; i++ {
    minified := minify(payload)
    if isValidShape(minified) {
      checksum += 12345
    }
  }
  ops := iterations
  fmt.Println("RESULT")
  fmt.Println(checksum)
  fmt.Println(ops)
}
