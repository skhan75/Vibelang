# VibeLang Observability Contracts

Date: 2026-02-22

## Purpose

Define deterministic contracts for structured logs, metrics, and traces in
runtime and tooling workflows.

## Structured Log Envelope

```json
{
  "ts_unix_ms": 0,
  "level": "INFO",
  "component": "runtime.scheduler",
  "event": "task_spawn",
  "message": "spawned task",
  "trace_id": "trace-0001",
  "span_id": "span-0001",
  "fields": {
    "queue_depth": 3
  }
}
```

Contract rules:

- `level`, `component`, `event`, and `message` are required.
- `trace_id`/`span_id` are required for trace-correlated logs.
- `fields` payload keys must be deterministic for identical event type.

## Metrics Envelope

```json
{
  "metric": "runtime.channel.blocking_duration_ms",
  "value": 12,
  "kind": "histogram",
  "labels": {
    "channel": "jobs",
    "profile": "release"
  }
}
```

Contract rules:

- `metric`, `value`, and `kind` are required.
- Label keys are stable and lower_snake_case.
- Units are encoded in metric name suffix.

## Trace Span Envelope

```json
{
  "trace_id": "trace-0001",
  "span_id": "span-0002",
  "parent_span_id": "span-0001",
  "name": "runtime.select.wait",
  "start_unix_ms": 0,
  "end_unix_ms": 1,
  "attributes": {
    "case_count": 3
  }
}
```

Contract rules:

- `trace_id`, `span_id`, and `name` are required.
- Timing fields are monotonic for a span.
- Attribute keys are deterministic for a span type.

## Collection Surfaces

- Runtime metrics baseline: `runtime/concurrency/design.md` observability section.
- Release metrics artifacts: `reports/v1/release_benchmarks.json`.
- Allocation visibility artifact: `reports/v1/allocation_visibility_smoke.json`.

## Validation

Observability contract smoke evidence is collected by:

- `tooling/observability/collect_observability_primitives_report.py`
- release gate job `observability_contracts_gate`.
