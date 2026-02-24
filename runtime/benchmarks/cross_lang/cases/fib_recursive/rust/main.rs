fn main() {
  let n: i64 = 200000;
  let mut a: i64 = 0;
  let mut b: i64 = 1;
  for _ in 0..n {
    let mut next = a + b;
    if next > 1000000000 {
      next -= 1000000000;
    }
    a = b;
    b = next;
  }
  let checksum = b;
  let ops = n;
  println!("RESULT");
  println!("{}", checksum);
  println!("{}", ops);
}
