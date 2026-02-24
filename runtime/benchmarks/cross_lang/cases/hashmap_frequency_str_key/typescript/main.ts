// @ts-nocheck
const iterations = 200000;
const buckets = 257;
const freq: Record<string, number> = {};
for (let i = 0; i < iterations; i++) {
  const k = i - Math.floor(i / buckets) * buckets;
  const key = String(k);
  freq[key] = (freq[key] || 0) + 1;
}
let checksum = 0;
for (let k = 0; k < buckets; k++) {
  const key = String(k);
  checksum += (freq[key] || 0) * (k + 1);
}
const ops = iterations;
console.log("RESULT");
console.log(checksum);
console.log(ops);
