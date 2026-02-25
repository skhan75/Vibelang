use std::collections::HashMap;

fn main() {
  let iterations: i64 = 200000;
  let buckets: i64 = 257;
  let mut freq: HashMap<i64, i64> = HashMap::new();
  for i in 0..iterations {
    let k = i - (i / buckets) * buckets;
    *freq.entry(k).or_insert(0) += 1;
  }
  let mut checksum: i64 = 0;
  for k in 0..buckets {
    checksum += freq.get(&k).unwrap_or(&0) * (k + 1);
  }
  let ops = iterations;
  println!("RESULT");
  println!("{}", checksum);
  println!("{}", ops);
}
