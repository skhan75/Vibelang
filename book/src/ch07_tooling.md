# Chapter 7: Tooling Workflow

Core toolchain flow:

```bash
vibe check app/main.yb
vibe build app/main.yb --profile release
vibe test app/
vibe fmt app/main.yb --check
vibe lint app/ --intent
vibe index app/ --stats
```

Normative command behavior is captured by CLI docs and tests.
