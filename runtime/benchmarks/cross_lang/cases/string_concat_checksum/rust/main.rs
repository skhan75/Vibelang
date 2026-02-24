fn main() {
  let iterations: i64 = 50000;
  let mut checksum: i64 = 0;
  for i in 0..iterations {
    let si = i.to_string();
    let sj = (i + 7).to_string();
    let pi: i64 = si.parse().unwrap();
    let pj: i64 = sj.parse().unwrap();
    checksum += pi + pj;
  }
  let ops = iterations;
  println!("RESULT");
  println!("{}", checksum);
  println!("{}", ops);
}
