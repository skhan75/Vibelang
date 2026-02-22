#!/usr/bin/env node
const fs = require("node:fs");
const path = require("node:path");

function fail(message) {
  console.error(`phase13 extension smoke failed: ${message}`);
  process.exit(1);
}

const root = path.resolve(__dirname, "..");
const packagePath = path.join(root, "package.json");
if (!fs.existsSync(packagePath)) {
  fail("missing package.json");
}

const pkg = JSON.parse(fs.readFileSync(packagePath, "utf8"));
const language = (pkg.contributes?.languages ?? []).find((item) => item.id === "vibelang");
if (!language) {
  fail("language contribution for id=vibelang is missing");
}
const grammar = (pkg.contributes?.grammars ?? []).find(
  (item) => item.language === "vibelang"
);
if (!grammar) {
  fail("grammar contribution for vibelang is missing");
}
if (!Array.isArray(pkg.activationEvents) || pkg.activationEvents.length === 0) {
  fail("activationEvents must be configured");
}

const requiredFiles = [
  "language-configuration.json",
  "syntaxes/vibelang.tmLanguage.json",
  "snippets/vibelang.code-snippets",
  "src/extension.ts",
];
for (const rel of requiredFiles) {
  const full = path.join(root, rel);
  if (!fs.existsSync(full)) {
    fail(`required file missing: ${rel}`);
  }
}

const distFile = path.join(root, "dist", "extension.js");
if (!fs.existsSync(distFile)) {
  fail("compiled extension output dist/extension.js is missing");
}

console.log("phase13 extension smoke passed");

