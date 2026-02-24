// @ts-nocheck
function isPrime(n: number): boolean {
  for (let d = 2; d * d <= n; d++) {
    const rem = n - Math.floor(n / d) * d;
    if (rem === 0) return false;
  }
  return true;
}
const limit = 12000;
let count = 0;
let sum = 0;
for (let n = 2; n <= limit; n++) {
  if (isPrime(n)) {
    count++;
    sum += n;
  }
}
const checksum = count * 1000000 + sum;
const ops = limit;
console.log("RESULT");
console.log(checksum);
console.log(ops);
