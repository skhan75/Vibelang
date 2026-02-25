// @ts-nocheck
let n = 200000;
let a = 0;
let b = 1;
for (let i = 0; i < n; i++) {
    let nextVal = a + b;
    if (nextVal > 1000000000) {
        nextVal = nextVal - 1000000000;
    }
    a = b;
    b = nextVal;
}
const checksum = b;
const ops = n;
console.log("RESULT");
console.log(checksum);
console.log(ops);
