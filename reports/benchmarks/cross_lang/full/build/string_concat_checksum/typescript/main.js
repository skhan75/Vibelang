// @ts-nocheck
const iterations = 50000;
let checksum = 0;
for (let i = 0; i < iterations; i++) {
    const si = String(i);
    const sj = String(i + 7);
    const pi = parseInt(si, 10);
    const pj = parseInt(sj, 10);
    checksum += pi + pj;
}
const ops = iterations;
console.log("RESULT");
console.log(checksum);
console.log(ops);
