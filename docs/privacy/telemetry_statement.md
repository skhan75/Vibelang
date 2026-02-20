# Telemetry and Privacy Statement

Date: 2026-02-20

## Scope

This statement covers optional telemetry emitted by VibeLang tooling, including
AI-related linting paths.

## Principles

- Core compile correctness does not depend on telemetry.
- Telemetry is opt-in for AI-related outputs.
- Local-first behavior is the default.

## What May Be Collected

- Performance counters (latency, request counts, budget status)
- Aggregate lint diagnostics counts and categories
- Tooling run metadata needed for quality trend reporting

## What Is Not Required for Core Compilation

- Source upload to external services
- Cloud connectivity for parse/type/codegen/link
- Mandatory paid AI services

## Controls

- Opt-in telemetry output via CLI flags (for example `--telemetry-out` where supported)
- Budget controls for AI-sidecar operations (latency/request caps)
- Offline-first modes remain available

## Data Handling

- Retention policy and storage location must be documented per deployment context.
- Teams should avoid storing sensitive source excerpts in telemetry outputs.

## Release Requirement

Every release candidate must confirm:

- Telemetry behavior matches this statement.
- Privacy-impacting changes are called out in release notes.
