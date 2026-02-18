# Runtime Benchmark Plan (v0.1)

## Objectives

Measure and track:

- GC throughput and pause behavior
- Scheduler scalability and overhead
- Channel communication performance
- End-to-end service latency under concurrency

## Benchmark Classes

## GC Benchmarks

1. Short-lived object churn (young generation stress)
2. Mixed-lifetime graph allocations
3. Large object allocation/reclamation

Metrics:

- Alloc/sec
- Minor/major GC count
- Pause p50/p95/p99
- Live heap and promotion rate

## Scheduler Benchmarks

1. Task spawn/join microbenchmark
2. Work-stealing under skewed load
3. Blocking I/O simulation with many parked tasks

Metrics:

- Tasks/sec
- Spawn overhead
- Queue contention
- Tail latency for task completion

## Channel Benchmarks

1. Single producer / single consumer
2. Many producer / one consumer
3. Many producer / many consumer
4. Select-heavy fan-in/fan-out patterns

Metrics:

- Msg/sec
- Send/recv latency
- Blocking ratio under bounded capacities

## Application Benchmarks

1. HTTP-like request processing pipeline
2. Stream aggregation workload
3. Background job worker pool

Metrics:

- Throughput (RPS/jobs per second)
- End-to-end latency
- CPU utilization
- Memory footprint

## Profiles and Build Modes

Each benchmark should run in:

- `dev` profile
- `release` profile

And with:

- contracts enabled
- contracts reduced

## Reference Hardware Baseline

At minimum record results for:

- 8-core developer laptop
- 16+ core server-class machine

## Reporting Format

For each run, persist:

- Toolchain version hash
- Benchmark revision hash
- Raw result JSON
- Aggregated markdown summary

## Regression Gates

CI perf gates (initial):

- No more than 10% regression on key scheduler/channel metrics
- No more than 15% regression on GC p95 pause target

## Future Extensions

- Cross-language comparison suite (Go/Rust baseline)
- Flamegraph and trace collection automation
