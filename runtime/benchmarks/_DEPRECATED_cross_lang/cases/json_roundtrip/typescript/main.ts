// @ts-nocheck
function minify(s: string): string {
  return s.replace(/\s/g, "");
}
function isValidShape(s: string): boolean {
  if (!s) return false;
  let brace = 0;
  for (const c of s) {
    if (c === "{") brace++;
    else if (c === "}") brace--;
    if (brace < 0) return false;
  }
  return brace === 0 && s.includes("value") && s.includes("12345");
}
const iterations = 120000;
const payload = '{ "value" : 12345 }';
let checksum = 0;
for (let i = 0; i < iterations; i++) {
  const buf = minify(payload);
  if (isValidShape(buf)) checksum += 12345;
}
const ops = iterations;
console.log("RESULT");
console.log(checksum);
console.log(ops);
