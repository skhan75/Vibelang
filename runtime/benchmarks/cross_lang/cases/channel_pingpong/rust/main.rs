use std::sync::mpsc;
use std::thread;

fn main() {
  let rounds: i64 = 50000;
  let (req_tx, req_rx) = mpsc::channel();
  let (resp_tx, resp_rx) = mpsc::channel();
  let server = thread::spawn(move || {
    for _ in 0..rounds {
      let token = req_rx.recv().unwrap();
      resp_tx.send(token + 1).unwrap();
    }
  });
  let mut checksum: i64 = 0;
  let mut token: i64 = 1;
  for _ in 0..rounds {
    req_tx.send(token).unwrap();
    let reply = resp_rx.recv().unwrap();
    checksum += reply;
    token = reply;
  }
  server.join().unwrap();
  let ops = rounds;
  println!("RESULT");
  println!("{}", checksum);
  println!("{}", ops);
}
