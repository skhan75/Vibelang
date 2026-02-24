// @ts-nocheck
const iterations = 200000;
const buckets = 257;
const freq = {};
for (let i = 0; i < iterations; i++) {
    const k = i - Math.floor(i / buckets) * buckets;
    freq[k] = (freq[k] || 0) + 1;
}
let checksum = 0;
for (let k = 0; k < buckets; k++) {
    checksum += (freq[k] || 0) * (k + 1);
}
const ops = iterations;
console.log("RESULT");
console.log(checksum);
console.log(ops);
