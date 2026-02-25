// @ts-nocheck
function i64(v: bigint): bigint {
  const mask = (1n << 64n) - 1n;
  let w = (v & mask);
  if (w >= 0x8000000000000000n) w -= 1n << 64n;
  return w;
}
const size = 120000;
let x = 17n;
let top1 = 0n;
let top2 = 0n;
let top3 = 0n;
let top4 = 0n;
for (let i = 0; i < size; i++) {
  x = i64(x * 73n + 19n);
  if (x > 100000n) x = i64(x - 100000n);
  if (x > top1) {
    top4 = top3;
    top3 = top2;
    top2 = top1;
    top1 = x;
  } else if (x > top2) {
    top4 = top3;
    top3 = top2;
    top2 = x;
  } else if (x > top3) {
    top4 = top3;
    top3 = x;
  } else if (x > top4) {
    top4 = x;
  }
}
const sum = top1 + top2 + top3 + top4;
const checksum = i64(sum);
const ops = size;
console.log("RESULT");
console.log(Number(checksum));
console.log(ops);
