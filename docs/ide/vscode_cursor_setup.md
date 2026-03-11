# VS Code/Cursor Setup for VibeLang

Date: 2026-02-22

## Scope

This setup guide covers the Phase 13.1 editor package in `editor-support/vscode`.
It enables syntax highlighting, snippets, LSP navigation/productivity features, and formatting.

## Prerequisites

- `vibe` CLI available on `PATH`
- Node.js 20+ and npm
- VS Code or Cursor

## Build extension package locally

From repository root:

```bash
cd editor-support/vscode
npm install
npm run build
npm run smoke
```

## Run extension in development host

1. Open `editor-support/vscode` in VS Code or Cursor.
2. Run `Run Extension` from the debug panel.
3. Open a `.yb` or `.vibe` file in the extension host window.

## Language features provided

- Syntax highlighting via `syntaxes/vibelang.tmLanguage.json`
- Language configuration via `language-configuration.json`
- Snippets via `snippets/vibelang.code-snippets`
- LSP-backed features:
  - diagnostics
  - definition and references
  - hover contract metadata
  - completion
  - document/workspace symbols
  - rename
  - code actions
  - document/range formatting

## Extension settings

- `vibelang.server.path` (default `vibe`): path to CLI executable
- `vibelang.lsp.transport` (default `jsonrpc`): `jsonrpc` or `legacy`
- `vibelang.formatOnSave` (default `true`): request formatting edits on save

## Format-on-save behavior

Formatting routes through LSP `textDocument/formatting` and shares the exact
formatter implementation from `crates/vibe_fmt`.
No separate editor-only formatter is used.

## Validation commands

```bash
python3 tooling/phase13/validate_textmate_grammar.py
python3 tooling/phase13/protocol_smoke.py
python3 tooling/phase13/check_diagnostics_parity.py
python3 tooling/phase13/benchmark_editor_ux.py --enforce
```

## Cursor agent profiles (optional)

This repo intentionally does **not** version editor state under `.cursor/` (it is ignored via `.gitignore`),
but we do keep **public templates** for agent profiles that contributors can opt into locally.

- **Template location**: `docs/ide/cursor_agents/vibelang-lead.md`
- **To use in Cursor**: copy it to `.cursor/agents/vibelang-lead.md` in your working tree.

