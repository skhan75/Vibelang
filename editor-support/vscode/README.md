# VibeLang VS Code/Cursor Extension

Language support for VibeLang in VS Code and Cursor.

## Features

- `.yb` and `.vibe` language registration
- Syntax highlighting via TextMate grammar
- Language-aware editor behavior (pairs, comments, indentation)
- Snippets for common VibeLang constructs
- LSP-powered diagnostics/navigation/completion (`vibe lsp`)
- Formatting via the same formatter used by the CLI/runtime tooling

## Prerequisites

- VS Code or Cursor
- `vibe` CLI available on `PATH`
- Node.js 20+ and npm (only required to build/package the extension locally)

Public VibeLang release binaries are available at:

- `https://github.com/skhan75/VibeLang/releases/tag/v1.0.0`

## Install In VS Code/Cursor (VSIX)

The extension is currently intended for local/VSIX installation.

From repository root:

```bash
cd editor-support/vscode
npm install
npm run build
npx --yes @vscode/vsce package
```

This produces a file like `vibelang-editor-support-0.13.1.vsix`.

Install it:

```bash
code --install-extension vibelang-editor-support-0.13.1.vsix
cursor --install-extension vibelang-editor-support-0.13.1.vsix
```

If CLI install is unavailable, use command palette:

- `Extensions: Install from VSIX...`

## Development Host Workflow

To iterate on extension code:

1. Open `editor-support/vscode` in VS Code or Cursor.
2. Run `Run VibeLang Extension` from the debug panel.
3. In the Extension Development Host window, open a `.yb` or `.vibe` file.

The launch config lives in `.vscode/launch.json`.

## Extension Settings

- `vibelang.server.path` (default: `vibe`)
  - Path to the CLI binary used to start the language server.
- `vibelang.lsp.transport` (default: `jsonrpc`)
  - LSP transport passed to `vibe lsp --transport`.
- `vibelang.formatOnSave` (default: `true`)
  - Requests LSP formatting when documents are saved.

## Validation Commands

From repo root:

```bash
python3 tooling/phase13/validate_textmate_grammar.py
python3 tooling/phase13/protocol_smoke.py
python3 tooling/phase13/check_diagnostics_parity.py
python3 tooling/phase13/benchmark_editor_ux.py --enforce
```

## Troubleshooting

- No syntax highlighting:
  - Confirm language mode is `VibeLang`.
  - Confirm file extension is `.yb` or `.vibe`.
- LSP features missing (diagnostics/formatting/completion):
  - Run `vibe --version` in the same shell environment.
  - Set `vibelang.server.path` explicitly if `vibe` is not on `PATH`.
- Format on save not running:
  - Ensure `vibelang.formatOnSave` is `true`.
  - Ensure the extension is activated for the current workspace file.

## Repository Layout

- `package.json` - extension contributions, activation rules, settings
- `src/extension.ts` - runtime bootstrap and LSP client wiring
- `syntaxes/vibelang.tmLanguage.json` - syntax grammar
- `language-configuration.json` - editor behavior
- `snippets/vibelang.code-snippets` - snippets

