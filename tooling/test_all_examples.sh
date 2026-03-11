#!/usr/bin/env bash
# Copyright 2025-2026 VibeLang Contributors
# SPDX-License-Identifier: Apache-2.0
#
# Run every example listed in examples/examples_manifest.json and compare
# the actual outcome against the expected status.
#
# Usage:
#   bash tooling/test_all_examples.sh            # text summary
#   bash tooling/test_all_examples.sh --json      # JSON summary
#
# Exit codes:
#   0  all outcomes match expectations
#   1  at least one unexpected failure or unexpected pass (regression / fix)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$REPO_ROOT/examples/examples_manifest.json"
JSON_MODE=false

if [[ "${1:-}" == "--json" ]]; then
  JSON_MODE=true
fi

if [[ ! -f "$MANIFEST" ]]; then
  echo "error: manifest not found at $MANIFEST" >&2
  exit 2
fi

TOTAL=0
PASS=0
EXPECTED_FAIL=0
UNEXPECTED_FAIL=0
UNEXPECTED_PASS=0
ASSERTION_FAIL=0

declare -a UNEXPECTED_FAIL_FILES=()
declare -a UNEXPECTED_PASS_FILES=()
declare -a ASSERTION_FAIL_FILES=()

keys=$(python3 -c "
import json, sys
with open('$MANIFEST') as f:
    m = json.load(f)
for k in sorted(m):
    if not k.startswith('_'):
        print(k)
")

while IFS= read -r file; do
  full_path="$REPO_ROOT/$file"
  if [[ ! -f "$full_path" ]]; then
    continue
  fi

  expect=$(python3 -c "
import json
with open('$MANIFEST') as f:
    m = json.load(f)
print(m['$file']['expect'])
")

  TOTAL=$((TOTAL + 1))

  output=$(vibe run "$full_path" 2>&1) && rc=0 || rc=$?

  if [[ "$expect" == "pass" ]]; then
    if [[ $rc -eq 0 ]]; then
      PASS=$((PASS + 1))
      if ! $JSON_MODE; then
        printf "  PASS       %s\n" "$file"
      fi

      test_output=$(vibe test "$full_path" --json 2>&1) || true
      ex_failed=$(echo "$test_output" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('examples_failed',0))" 2>/dev/null || echo "0")
      if [[ "$ex_failed" != "0" ]]; then
        ASSERTION_FAIL=$((ASSERTION_FAIL + 1))
        ASSERTION_FAIL_FILES+=("$file")
        if ! $JSON_MODE; then
          printf "  ASSERT-FAIL %s  (%s @examples failed)\n" "$file" "$ex_failed"
        fi
      fi
    else
      UNEXPECTED_FAIL=$((UNEXPECTED_FAIL + 1))
      UNEXPECTED_FAIL_FILES+=("$file")
      if ! $JSON_MODE; then
        printf "  UNEXPECTED-FAIL %s  (expected pass, got exit=%d)\n" "$file" "$rc"
      fi
    fi
  else
    if [[ $rc -ne 0 ]]; then
      EXPECTED_FAIL=$((EXPECTED_FAIL + 1))
      if ! $JSON_MODE; then
        printf "  EXPECTED-FAIL %s\n" "$file"
      fi
    else
      UNEXPECTED_PASS=$((UNEXPECTED_PASS + 1))
      UNEXPECTED_PASS_FILES+=("$file")
      if ! $JSON_MODE; then
        printf "  UNEXPECTED-PASS %s  (expected %s, but passed)\n" "$file" "$expect"
      fi
    fi
  fi
done <<< "$keys"

if $JSON_MODE; then
  uf_json=$(printf '%s\n' "${UNEXPECTED_FAIL_FILES[@]+"${UNEXPECTED_FAIL_FILES[@]}"}" | python3 -c "import sys,json; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))")
  up_json=$(printf '%s\n' "${UNEXPECTED_PASS_FILES[@]+"${UNEXPECTED_PASS_FILES[@]}"}" | python3 -c "import sys,json; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))")
  af_json=$(printf '%s\n' "${ASSERTION_FAIL_FILES[@]+"${ASSERTION_FAIL_FILES[@]}"}" | python3 -c "import sys,json; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))")
  cat <<ENDJSON
{
  "total": $TOTAL,
  "pass": $PASS,
  "expected_fail": $EXPECTED_FAIL,
  "unexpected_fail": $UNEXPECTED_FAIL,
  "unexpected_pass": $UNEXPECTED_PASS,
  "assertion_fail": $ASSERTION_FAIL,
  "unexpected_fail_files": $uf_json,
  "unexpected_pass_files": $up_json,
  "assertion_fail_files": $af_json
}
ENDJSON
else
  echo ""
  echo "========================================"
  echo "  Examples Test Summary"
  echo "========================================"
  printf "  Total:            %d\n" "$TOTAL"
  printf "  Pass:             %d\n" "$PASS"
  printf "  Expected fail:    %d\n" "$EXPECTED_FAIL"
  printf "  Unexpected fail:  %d\n" "$UNEXPECTED_FAIL"
  printf "  Unexpected pass:  %d\n" "$UNEXPECTED_PASS"
  printf "  Assertion fail:   %d\n" "$ASSERTION_FAIL"
  echo "========================================"

  if [[ ${#UNEXPECTED_FAIL_FILES[@]} -gt 0 ]]; then
    echo ""
    echo "Unexpected failures (regressions):"
    for f in "${UNEXPECTED_FAIL_FILES[@]}"; do
      echo "  - $f"
    done
  fi

  if [[ ${#UNEXPECTED_PASS_FILES[@]} -gt 0 ]]; then
    echo ""
    echo "Unexpected passes (gaps now fixed -- update manifest):"
    for f in "${UNEXPECTED_PASS_FILES[@]}"; do
      echo "  - $f"
    done
  fi

  if [[ ${#ASSERTION_FAIL_FILES[@]} -gt 0 ]]; then
    echo ""
    echo "Assertion failures (@examples mismatch):"
    for f in "${ASSERTION_FAIL_FILES[@]}"; do
      echo "  - $f"
    done
  fi
fi

if [[ $UNEXPECTED_FAIL -gt 0 ]] || [[ $ASSERTION_FAIL -gt 0 ]]; then
  exit 1
fi
exit 0
