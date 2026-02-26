# Third-Party Benchmark Summary

- profile: `full`
- generated_at_utc: `2026-02-26T07:21:50Z`
- budget_status: `fail`

## Runtime Geomean Ratios (VibeLang vs Baselines)

| baseline | vibelang_ratio |
| --- | ---: |
| c | 0.093 |
| cpp | 0.095 |
| elixir | 0.003 |
| go | 0.037 |
| kotlin | 1.943 |
| python | 0.010 |
| rust | n/a |
| swift | n/a |
| typescript | 0.014 |
| zig | n/a |

Interpretation: ratio > 1.0 means VibeLang is slower on average; ratio < 1.0 means faster.

## Compile Cold Ratios (VibeLang vs Baselines)

| baseline | vibelang_cold_ratio |
| --- | ---: |
| c | 1.112 |
| cpp | 1.123 |
| elixir | 0.317 |
| go | 1.041 |
| kotlin | 0.305 |
| python | 0.542 |
| rust | n/a |
| swift | n/a |
| typescript | 0.784 |
| zig | n/a |

## Category Snapshot

| language | memory_mean_bytes | incremental_compile_ms | coro_prime_sieve_ms |
| --- | ---: | ---: | ---: |
| vibelang | 4561306 | 1578.083 | 1.643 |
| c | 3619840 | 1570.563 | n/a |
| cpp | 2048000 | 1562.087 | n/a |
| rust | n/a | n/a | n/a |
| go | 9943122 | 1628.561 | 12.341 |
| zig | n/a | n/a | n/a |
| swift | n/a | n/a | n/a |
| kotlin | n/a | 1577.541 | 1.285 |
| elixir | 83461266 | 1668.887 | 329.597 |
| python | 28289210 | 1556.459 | 316.813 |
| typescript | 78087936 | 1578.054 | 155.549 |

## AI-Native Proxy Signals

- vibelang_runtime_relative_stddev: `0.082682`
- vibelang_incremental_compile_mean_ms: `1578.083`
- note: AI-native productivity is proxied by incremental compile feedback and runtime stability; replace with direct agent-task benchmarks when available.

## Runtime Mean Time by Problem (ms)

| problem | vibelang | c | cpp | rust | go | zig | swift | kotlin | elixir | python | typescript |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| binarytrees | 1.531 | n/a | n/a | n/a | 199.144 | n/a | n/a | 1.587 | n/a | 372.292 | 134.301 |
| coro-prime-sieve | 1.643 | n/a | n/a | n/a | 12.341 | n/a | n/a | 1.285 | 329.597 | 316.813 | 155.549 |
| edigits | 1.630 | n/a | n/a | n/a | 29.987 | n/a | n/a | n/a | n/a | 226.932 | n/a |
| fannkuch-redux | 1.582 | n/a | 14.678 | n/a | 75.280 | n/a | n/a | n/a | n/a | n/a | n/a |
| fasta | 32.336 | n/a | n/a | n/a | 16.625 | n/a | n/a | n/a | n/a | 270.390 | 240.842 |
| helloworld | 0.817 | 0.988 | 0.856 | n/a | 1.768 | n/a | n/a | n/a | 269.005 | 33.077 | 37.261 |
| http-server | 2.285 | n/a | n/a | n/a | 88.917 | n/a | n/a | 1.230 | n/a | 1700.013 | 307.746 |
| json-serde | 17.306 | n/a | n/a | n/a | 113.156 | n/a | n/a | 1.144 | n/a | 137.241 | 168.752 |
| knucleotide | 1.823 | 25.320 | n/a | n/a | 89.262 | n/a | n/a | n/a | n/a | 165.069 | n/a |
| lru | 1.464 | n/a | n/a | n/a | 51.313 | n/a | n/a | 1.132 | n/a | 271.522 | 150.999 |
| mandelbrot | 1.632 | n/a | n/a | n/a | 83.095 | n/a | n/a | n/a | n/a | n/a | n/a |
| merkletrees | 1.638 | n/a | n/a | n/a | 331.607 | n/a | n/a | 1.202 | n/a | 1.131 | 176.777 |
| nbody | 2.142 | 27.703 | 16.524 | n/a | 29.168 | n/a | n/a | 1.345 | n/a | 1268.777 | 77.491 |
| nsieve | 1.636 | 35.607 | 50.235 | n/a | 52.268 | n/a | n/a | n/a | n/a | 710.775 | n/a |
| pidigits | 1.535 | n/a | n/a | n/a | 206.725 | n/a | n/a | 1.101 | 627.014 | 240.615 | 1208.610 |
| regex-redux | 1.726 | n/a | n/a | n/a | 1323.761 | n/a | n/a | 1.209 | n/a | 331.965 | n/a |
| secp256k1 | 4.380 | n/a | n/a | n/a | 20.252 | n/a | n/a | 1.107 | n/a | 459.116 | 416.381 |
| spectral-norm | 1.397 | 43.001 | 76.503 | n/a | 102.471 | n/a | n/a | n/a | n/a | 2077.996 | 226.581 |

## Wins

- Runtime: faster than c (ratio=0.093)
- Runtime: faster than cpp (ratio=0.095)
- Runtime: faster than go (ratio=0.037)
- Runtime: faster than elixir (ratio=0.003)
- Runtime: faster than python (ratio=0.010)
- Runtime: faster than typescript (ratio=0.014)
- Compile: faster than kotlin (ratio=0.305)
- Compile: faster than elixir (ratio=0.317)
- Compile: faster than python (ratio=0.542)
- Compile: faster than typescript (ratio=0.784)

## Gaps and Improvement Opportunities

- Runtime: slower than kotlin (ratio=1.943)
- Compile: slower than cpp (ratio=1.123)
- Compile: slower than c (ratio=1.112)
- Compile: slower than go (ratio=1.041)

## Simple-language analysis

- VibeLang still has performance gaps versus some baselines. Focus next on the worst ratios first.
- There are measurable strengths that can be highlighted in public benchmark notes.
- Keep fairness caveats explicit: toolchain versions, host environment, and benchmark semantics affect results.

## Budget Gate Output

- mode: `publication-strict`
- status: `fail`
- violations:
  - required runtime language `rust` not available (status=unavailable)
  - required runtime language `zig` not available (status=unavailable)
  - required runtime language `swift` not available (status=unavailable)
  - required compile language `rust` not available (status=unavailable)
  - required compile language `zig` not available (status=unavailable)
  - required compile language `swift` not available (status=unavailable)
  - tooling.publication_mode must be true for publication validation
  - tooling.docker_enabled must be true for publication validation
  - preflight.status must be `ok` for publication validation
  - publication.mode must be `strict`
  - adapter parity validator failed: stdout={
  "format": "vibe-parity-validation-v1",
  "status": "fail",
  "publication_mode": true,
  "required_problem_count": 18,
  "manifest_problem_count": 18,
  "noncanonical_count": 4,
  "warnings": [
    "heuristic proxy signal in `edigits` (literal_print_count=24, matched_signatures=[])",
    "heuristic proxy signal in `json-serde` (literal_print_count=3, matched_signatures=['9f8a9edb47ee2f885325cdc8a18591f4', '80bf2dee6461725c8200bfced3c695b7'])",
    "heuristic proxy signal in `secp256k1` (literal_print_count=2, matched_signatures=['bac4db182bd8e59d'])"
  ],
  "violations": [
    "publication mode requires all problems canonical; noncanonical entries: edigits, http-server, json-serde, secp256k1"
  ],
  "analysis": {
    "helloworld": {
      "literal_print_count": 1,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "nsieve": {
      "literal_print_count": 0,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "binarytrees": {
      "literal_print_count": 5,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "merkletrees": {
      "literal_print_count": 6,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "nbody": {
      "literal_print_count": 3,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "spectral-norm": {
      "literal_print_count": 2,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "pidigits": {
      "literal_print_count": 0,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "edigits": {
      "literal_print_count": 24,
      "matched_proxy_signatures": [],
      "suspicious": true
    },
    "mandelbrot": {
      "literal_print_count": 4,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "fannkuch-redux": {
      "literal_print_count": 3,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "fasta": {
      "literal_print_count": 1,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "knucleotide": {
      "literal_print_count": 2,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "regex-redux": {
      "literal_print_count": 1,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "json-serde": {
      "literal_print_count": 3,
      "matched_proxy_signatures": [
        "9f8a9edb47ee2f885325cdc8a18591f4",
        "80bf2dee6461725c8200bfced3c695b7"
      ],
      "suspicious": true
    },
    "coro-prime-sieve": {
      "literal_print_count": 0,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "http-server": {
      "literal_print_count": 2,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "lru": {
      "literal_print_count": 0,
      "matched_proxy_signatures": [],
      "suspicious": false
    },
    "secp256k1": {
      "literal_print_count": 2,
      "matched_proxy_signatures": [
        "bac4db182bd8e59d"
      ],
      "suspicious": true
    }
  }
} stderr=adapter parity validation failed: publication mode requires all problems canonical; noncanonical entries: edigits, http-server, json-serde, secp256k1
  - [publication-mode] warning promoted to failure: runtime ratio missing/zero for baseline `rust`
  - [publication-mode] warning promoted to failure: runtime ratio missing/zero for baseline `zig`
  - [publication-mode] warning promoted to failure: runtime ratio missing/zero for baseline `swift`
  - [publication-mode] warning promoted to failure: compile ratio missing/zero for baseline `rust`
  - [publication-mode] warning promoted to failure: compile ratio missing/zero for baseline `zig`
  - [publication-mode] warning promoted to failure: compile ratio missing/zero for baseline `swift`

