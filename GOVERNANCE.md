# Governance

VibeLang is an open source project licensed under [Apache 2.0](LICENSE).

## Decision-making

VibeLang currently follows a **BDFL (Benevolent Dictator For Life)** model.
The project lead makes final decisions on language design, compiler
architecture, and release policy. As the contributor base grows, governance
will evolve toward a maintainer council model.

## Roles

| Role | Scope | Current |
|---|---|---|
| **Project lead** | Language design, architecture, release decisions | [@skhan75](https://github.com/skhan75) |
| **Maintainer** | Review and merge PRs, triage issues, enforce quality gates | Project lead (expanding) |
| **Contributor** | Submit PRs, report issues, improve docs and examples | Anyone — see [CONTRIBUTING.md](CONTRIBUTING.md) |

## How changes are proposed

- **Bug fixes and small improvements**: open a PR directly. See [CONTRIBUTING.md](CONTRIBUTING.md).
- **New language features or breaking changes**: open an issue describing the
  motivation, design, alternatives considered, and migration impact. Discussion
  happens on the issue before implementation begins.
- **Stdlib additions**: propose via issue with API surface, use cases, and
  stability tier (experimental, stable). New modules start as experimental.

## Release process

Releases follow the process documented in `docs/release/`. Release candidates
go through the [GA checklist](docs/checklists/ga_go_no_go_checklist.md) before
promotion to stable.

## Evolving governance

As VibeLang grows, this document will be updated to reflect new roles,
voting procedures, and RFC processes. Proposals to change governance follow
the same issue-first workflow described above.
