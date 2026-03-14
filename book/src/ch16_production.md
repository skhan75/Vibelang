# Chapter 16: Production and Release Engineering

Writing correct VibeLang code is the first step. Getting it into production
reliably, keeping it running, and evolving it with a team requires engineering
discipline beyond the language itself. This chapter covers the practices that
bridge the gap between "it works on my machine" and "it runs in production with
confidence."

---

## 16.1 From Development to Production

### Build Profiles and Their Implications

The choice of build profile has direct consequences for program behavior:

| Aspect | Dev | Test | Release |
|---|---|---|---|
| Optimization | None | None | Full (Cranelift) |
| Contract checks | Runtime | Runtime | Configurable |
| Debug info | Full | Full | Stripped |
| Binary size | Larger | Larger | Smaller |
| Compilation speed | Fast | Fast | Slower |
| `@examples` | Not executed | Executed as tests | Not included |

The critical difference is contract behavior. In dev and test builds, every
`@require` and `@ensure` compiles to a runtime check that panics with a
diagnostic on violation. In release builds, you choose the behavior:

```toml
# vibe.toml
[release]
contracts = "checked"
```

**`checked`** (recommended default): Contracts remain as runtime checks. The
binary is slightly larger and slower, but contract violations in production
produce clear diagnostics instead of silent corruption.

**`unchecked`**: Contracts are used for static analysis during compilation but
removed from the binary. The compiler still verifies that contracts are
well-typed and pure, but no runtime checks execute. Use this when you have high
confidence in your test coverage and need maximum performance.

**`removed`**: Contracts are stripped entirely. The compiler skips contract
validation. Use this only for performance-critical inner loops where you have
proven correctness through other means.

A practical approach: start with `checked` in production. If profiling shows
that contract checks are a measurable bottleneck in a specific hot path,
consider `unchecked` for that module while keeping `checked` everywhere else.

### Performance Optimization

Release builds enable Cranelift's optimization passes:

- **Constant folding:** `2 + 3` becomes `5` at compile time
- **Dead code elimination:** Unreachable branches are removed
- **Function inlining:** Small pure functions are expanded at call sites
- **Loop optimizations:** Invariant computations are hoisted out of loops
- **Register allocation:** Values are kept in registers instead of memory

For most programs, the release profile provides sufficient performance without
any manual optimization. When you do need to optimize:

1. **Profile first.** Use system profiling tools (`perf`, `instruments`) on the
   release binary. Don't guess where the bottleneck is.

2. **Check effects.** Functions with `@effect alloc` in hot paths are the most
   common performance issue. Reducing allocations often has the largest impact.

3. **Reduce channel contention.** If profiling shows tasks blocked on channel
   operations, consider increasing buffer sizes or restructuring the pipeline.

4. **Batch I/O.** Functions with `@effect io` or `@effect net` in loops should
   be restructured to batch operations where possible.

---

## 16.2 Deterministic Builds

### Same Source + Same Flags = Same Binary

VibeLang guarantees that compiling the same source code with the same compiler
version and the same flags produces a bit-identical binary. This is not an
aspiration — it is an enforced property of the compiler.

### Reproducibility Verification

Verify reproducibility with a simple test:

```bash
vibe build --release src/main.yb -o build_a
vibe build --release src/main.yb -o build_b
sha256sum build_a build_b
```

```
e3b0c44298fc1c149afb... build_a
e3b0c44298fc1c149afb... build_b
```

Identical hashes confirm reproducibility. Run this check in CI on every release
to catch any regression in determinism.

### What Can Break Reproducibility

While the compiler itself is deterministic, external factors can affect the
binary:

| Factor | Impact | Mitigation |
|---|---|---|
| Different compiler version | Different binary | Pin compiler version in CI |
| Different target platform | Different binary | Specify `--target` explicitly |
| Different dependency versions | Different binary | Commit `vibe.lock` |
| File system ordering | None (compiler is order-independent) | N/A |
| System time | None (no timestamps in binary) | N/A |
| Environment variables | None (not embedded) | N/A |

The key practice: pin your compiler version and commit your lock file. With
these two controls, reproducibility is guaranteed.

### Why This Matters for Audits and Compliance

Regulated industries (finance, healthcare, government) require traceability from
source code to deployed binary. Deterministic builds provide this:

1. **Audit trail:** Given a deployed binary's hash, you can identify the exact
   source code and compiler version that produced it.

2. **Third-party verification:** An auditor can rebuild from source and verify
   that their binary matches the deployed one.

3. **Tamper detection:** If a deployed binary's hash doesn't match the expected
   value from a reproducible build, it has been modified.

4. **Rollback confidence:** When rolling back to a previous version, you know
   the rebuilt binary is identical to what was previously deployed.

---

## 16.3 CI/CD Integration

### Example CI Pipeline

Here is a complete CI pipeline in GitHub Actions style:

```yaml
name: VibeLang CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install VibeLang
        run: |
          # Prefer pinning your toolchain in CI (release artifact or source build).
          # This example builds the VibeLang CLI from source at the checked-out revision.
          cargo build --release -p vibe_cli
          sudo install -m 0755 target/release/vibe /usr/local/bin/vibe

      - name: Format check
        run: vibe fmt --check .

      - name: Type check
        run: vibe check src/

      - name: Run tests
        run: vibe test src/

      - name: Lint
        run: vibe lint .

      - name: Intent analysis (changed files)
        run: vibe lint --intent --changed .

      - name: Build release
        run: vibe build --release src/main.yb -o myapp

      - name: Verify reproducibility
        run: |
          vibe build --release src/main.yb -o myapp_verify
          sha256sum myapp myapp_verify | awk '{print $1}' | sort -u | wc -l | grep -q '^1$'

      - name: Audit dependencies
        run: vibe pkg audit

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: myapp-linux-x86_64
          path: myapp
```

### Pipeline Stage Ordering

The stages are ordered by speed and specificity:

1. **`vibe fmt --check`** — Fastest. Catches formatting issues in milliseconds.
   Fail early if code isn't formatted.

2. **`vibe check`** — Fast. Catches type errors, effect violations, and contract
   issues without building. Most errors are caught here.

3. **`vibe test`** — Medium. Runs all `@examples` and test functions. Catches
   logic errors and contract violations.

4. **`vibe lint`** — Medium. Catches code quality issues. Less critical than
   type errors but important for maintainability.

5. **`vibe lint --intent`** — Slower when AI is enabled (requires
   `ANTHROPIC_API_KEY`). Catches intent drift via Claude. Local heuristic
   checks run without a key. Use `--changed` and `--mode hybrid` in CI.

6. **`vibe build --release`** — Slowest. Produces the release artifact. Only
   runs after all checks pass.

7. **Reproducibility verification** — Builds twice and compares hashes. Run on
   release builds to maintain the determinism guarantee.

8. **`vibe pkg audit`** — Checks dependencies for known vulnerabilities. Run on
   every build to catch new advisories.

### Branch Protection Rules

Configure your repository to require these checks before merging:

- `vibe fmt --check` must pass
- `vibe check` must pass
- `vibe test` must pass with zero failures
- `vibe lint` must pass with zero errors (warnings are acceptable)

Optional but recommended:

- `vibe lint --intent` must pass (no intent drift warnings)
- `vibe pkg audit` must pass (no known vulnerabilities)

---

## 16.4 Release Engineering

### Versioning Strategy

VibeLang projects use semantic versioning (SemVer):

```
MAJOR.MINOR.PATCH
```

- **MAJOR:** Breaking changes to public API (changed function signatures,
  removed exports, changed contract semantics)
- **MINOR:** New features that don't break existing callers (new functions, new
  types, relaxed contracts)
- **PATCH:** Bug fixes that don't change the API (implementation corrections,
  performance improvements)

Version is declared in `vibe.toml`:

```toml
[project]
name = "myapp"
version = "1.3.2"
```

### Changelog and Distribution

Maintain a `CHANGELOG.md` that references contract and effect changes, not just
code changes. When a `@require` or `@ensure` changes, that is a semantic change
callers need to know about.

Release binaries are standalone executables:

```bash
vibe build --release src/main.yb -o myapp-linux-x86_64
sha256sum myapp-linux-x86_64 > myapp-linux-x86_64.sha256
```

VibeLang binaries are statically linked by default, so they run in `scratch`
Docker containers without any base image:

```dockerfile
FROM scratch
COPY myapp /myapp
ENTRYPOINT ["/myapp"]
```

---

## 16.5 Monitoring and Observability

### Contract Violations in Production

When running with `contracts = "checked"` in release mode, contract violations
produce structured diagnostics:

```
CONTRACT VIOLATION — ABORTING
  function: process_order
  file:     src/orders.yb:45
  require:  order.total >= 0
  actual:   order.total = -150

  Stack trace:
    src/orders.yb:45    process_order
    src/handler.yb:22   handle_request
    src/main.yb:10      main
```

This diagnostic contains everything needed to reproduce the issue:

1. **Which contract failed** — the exact `@require` or `@ensure` expression
2. **The actual values** — what the violating input was
3. **The call stack** — how execution reached the violation

### Effect-Based Monitoring Boundaries

Effect declarations inform your monitoring strategy. Functions with specific
effects need specific monitoring:

| Effect | What to monitor |
|---|---|
| `io` | File system errors, disk space, I/O latency |
| `net` | Network errors, latency, timeout rates |
| `concurrency` | Task count, channel backpressure, scheduling latency |
| `alloc` | Heap size, GC pause frequency, allocation rate |
| `nondet` | Entropy source availability |

When adding monitoring to a VibeLang service, start by listing all functions
with `@effect net` or `@effect io` — these are your external dependency
boundaries and the most likely sources of production failures.

### Deterministic Crash Reproduction

VibeLang's deterministic builds simplify crash investigation: identify the
deployed binary's hash, look up the commit that produced it, rebuild locally
(the binary is identical), and replay the input from the contract violation
diagnostic. No "works on my machine" issues caused by different compilation.

For concurrent programs, exact scheduling reproduction is non-deterministic, but
contract violations provide enough context (violating values and call stack) to
identify bugs without exact replay.

---

## 16.6 Team Adoption

### Onboarding New Developers

A structured onboarding path for new team members:

**Week 1: Language fundamentals**

- Days 1–2: Chapters 1–4 (installation, first program, syntax, types)
- Days 3–4: Chapters 5–7 (control flow, contracts, effects)
- Day 5: Write a small program with contracts and effects, get code review

**Week 2: Advanced features and team practices**

- Days 1–2: Chapters 8–11 (error handling, modules, concurrency)
- Days 3–4: Chapters 12–13 (ownership, advanced patterns)
- Day 5: Contribute a small feature to the team's codebase with full contracts

The key insight: new developers should write contracts from day one. Don't let
them learn "code first, contracts later" — that habit is hard to break.

### Code Review with Contracts

Contracts change how code review works. Instead of reviewing only the
implementation, reviewers also evaluate the specification:

**Review the contracts first:**

1. Does the `@intent` accurately describe what the function does?
2. Are the `@examples` sufficient? Do they cover edge cases?
3. Are the `@require` preconditions the right boundary between caller
   responsibility and function responsibility?
4. Are the `@ensure` postconditions strong enough to be useful but not so
   strong that they over-constrain the implementation?
5. Are the `@effect` declarations accurate and minimal?

**Then review the implementation:**

6. Does the implementation satisfy the contracts?
7. Is the implementation the simplest way to satisfy the contracts?
8. Are there code paths that could violate the postconditions?

This two-phase review is more effective than reviewing implementation alone
because the contracts provide a specification to review against. A reviewer who
understands the contracts can catch implementation bugs without tracing every
code path.

### Gradual Adoption Strategy

If you're introducing VibeLang to an existing team or migrating from another
language, adopt gradually:

**Phase 1: Toolchain adoption**

- Set up CI with `vibe fmt --check`, `vibe check`, and `vibe test`
- Establish formatting as non-negotiable (eliminates style debates immediately)
- Run `vibe lint` but treat warnings as advisory

**Phase 2: Contract discipline**

- Require `@intent` on all public functions
- Require `@examples` on all public functions with non-trivial logic
- Add `@require` and `@ensure` to critical business logic
- Run `vibe lint --intent` on changed files

**Phase 3: Effect discipline**

- Require accurate `@effect` declarations on all functions
- Use effect information to classify tests (unit vs. integration)
- Refactor to push effects to the edges where possible

**Phase 4: Full adoption**

- Require contracts on all functions (public and private)
- Use `vibe lint --intent` on all files
- Enforce contract quality rubric in code review
- Monitor contract violations in production

Each phase builds on the previous one. Don't skip to Phase 4 — the team needs
time to internalize each practice before adding the next.

### Common Adoption Mistakes

- **Writing contracts after implementation** — they tend to describe what the
  code does rather than what it should do. Write contracts first.
- **Over-constraining** — postconditions that restate the algorithm add
  complexity without value. Capture invariants, not algorithms.
- **Ignoring effects** — if `@effect` is treated as optional, the system
  provides no value. Effects must be accurate and reviewed.
- **Skipping `vibe fmt`** — adopt it on day one. Formatting debates waste
  enormous team energy.

---

## 16.7 Summary

Production VibeLang requires engineering discipline beyond writing correct code:

- **Build profiles** control the trade-off between safety and performance.
  Start with `contracts = "checked"` in production and relax only where
  profiling justifies it.

- **Deterministic builds** guarantee that the same source produces the same
  binary. Pin your compiler version, commit your lock file, and verify
  reproducibility in CI.

- **CI/CD pipelines** should run checks in order of speed: format, check, test,
  lint, build. Each stage gates the next. Reproducibility verification and
  dependency auditing run on every release.

- **Release engineering** follows semantic versioning, maintains changelogs that
  reference contract changes, and distributes standalone binaries or minimal
  container images.

- **Monitoring** leverages effect declarations to identify external dependency
  boundaries and contract violations to produce actionable crash diagnostics.
  Deterministic builds enable exact reproduction of production issues.

- **Team adoption** follows a phased approach: toolchain first, then contracts,
  then effects, then full discipline. New developers write contracts from day
  one. Code review evaluates contracts before implementation.

The combination of VibeLang's language features (types, contracts, effects) and
engineering practices (deterministic builds, CI gates, phased adoption) produces
systems that are correct, auditable, and maintainable — even as teams grow and
codebases evolve.

---

Next: Chapter 17 covers building real applications — CLI tools and services —
using VibeLang's standard library.
