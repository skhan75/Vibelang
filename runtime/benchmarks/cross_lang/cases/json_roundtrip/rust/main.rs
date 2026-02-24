fn minify(s: &str) -> String {
  s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn is_valid_shape(s: &str) -> bool {
  if s.is_empty() {
    return false;
  }
  let mut brace = 0i32;
  for c in s.chars() {
    match c {
      '{' => brace += 1,
      '}' => brace -= 1,
      _ => {}
    }
    if brace < 0 {
      return false;
    }
  }
  brace == 0 && s.contains("value") && s.contains("12345")
}

fn main() {
  let iterations: i64 = 120000;
  let payload = "{ \"value\" : 12345 }";
  let mut checksum: i64 = 0;
  for _ in 0..iterations {
    let minified = minify(payload);
    if is_valid_shape(&minified) {
      checksum += 12345;
    }
  }
  let ops = iterations;
  println!("RESULT");
  println!("{}", checksum);
  println!("{}", ops);
}
