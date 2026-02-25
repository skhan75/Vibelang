use std::sync::mpsc;
use std::thread;

fn main() {
  let workers: i64 = 4;
  let limit: i64 = 60000;
  let (tx, rx) = mpsc::channel();
  for w in 0..workers {
    let tx = tx.clone();
    thread::spawn(move || {
      let mut local_sum: i64 = 0;
      let mut i = w;
      while i < limit {
        local_sum += i + 1;
        i += workers;
      }
      tx.send(local_sum).unwrap();
    });
  }
  drop(tx);
  let mut checksum: i64 = 0;
  for _ in 0..workers {
    checksum += rx.recv().unwrap();
  }
  let ops = limit;
  println!("RESULT");
  println!("{}", checksum);
  println!("{}", ops);
}
