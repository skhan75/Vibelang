fn main() {
  let size: i64 = 120000;
  let mut x: i64 = 17;
  let mut top1: i64 = 0;
  let mut top2: i64 = 0;
  let mut top3: i64 = 0;
  let mut top4: i64 = 0;
  for _ in 0..size {
    x = x * 73 + 19;
    if x > 100000 {
      x = x - 100000;
    }
    if x > top1 {
      top4 = top3;
      top3 = top2;
      top2 = top1;
      top1 = x;
    } else if x > top2 {
      top4 = top3;
      top3 = top2;
      top2 = x;
    } else if x > top3 {
      top4 = top3;
      top3 = x;
    } else if x > top4 {
      top4 = x;
    }
  }
  let checksum: i64 = top1 + top2 + top3 + top4;
  let ops = size;
  println!("RESULT");
  println!("{}", checksum);
  println!("{}", ops);
}
