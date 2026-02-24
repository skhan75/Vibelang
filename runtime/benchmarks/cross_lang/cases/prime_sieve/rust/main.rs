fn is_prime(n: i64) -> bool {
  let mut d: i64 = 2;
  while d * d <= n {
    let rem = n - (n / d) * d;
    if rem == 0 {
      return false;
    }
    d += 1;
  }
  true
}

fn main() {
  let limit: i64 = 12000;
  let mut count: i64 = 0;
  let mut sum: i64 = 0;
  let mut n: i64 = 2;
  while n <= limit {
    if is_prime(n) {
      count += 1;
      sum += n;
    }
    n += 1;
  }
  let checksum = count * 1000000 + sum;
  let ops = limit;
  println!("RESULT");
  println!("{}", checksum);
  println!("{}", ops);
}
