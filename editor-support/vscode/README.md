# VibeLang VS Code/Cursor Extension

This package hosts VS Code/Cursor language support for VibeLang.

## Scope

- `.yb` and `.vibe` language registration
- TextMate grammar and language configuration
- Snippets
- Language server client wiring to `vibe lsp`
- Formatter integration through LSP

## Structure

- `package.json` - extension contributions and activation rules
- `src/extension.ts` - extension activation/runtime client bootstrap
- `syntaxes/vibelang.tmLanguage.json` - syntax grammar
- `language-configuration.json` - editor behavior config
- `snippets/vibelang.code-snippets` - starter snippets

