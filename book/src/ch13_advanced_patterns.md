# Chapter 13: Advanced Patterns and Real-World Programs

Everything you've learned so far — types, contracts, effects, error handling,
concurrency, ownership — comes together when you build real programs. This
chapter walks through production-style VibeLang programs that combine multiple
language features into cohesive designs.

---

## 13.1 Building an Agentic AI Pipeline

The most demanding test of a language's safety features is agentic AI: programs
where an AI model proposes actions, and the program must decide whether to
execute them. This requires risk scoring, budget tracking, retry logic, and
audit trails — all enforced by the type system and contracts.

### The Domain Types

```vibe
module agent_pipeline

type Action {
    name: Str,
    risk_score: Float,
    estimated_cost: Float,
    payload: Str,
}

type ActionResult {
    action: Action,
    outcome: Str,
    actual_cost: Float,
    approved: Bool,
}

type Budget {
    total: Float,
    spent: Float,
    max_single_action: Float,
}

type PipelineConfig {
    max_risk: Float,
    max_retries: Int,
    timeout_ms: Int,
}
```

### The Guardrail Function

```vibe
@intent "Determine whether an action is safe to execute given current budget"
@require action.risk_score >= 0.0
@require action.risk_score <= 1.0
@require budget.spent <= budget.total
pub can_execute(
    action: Action,
    budget: Budget,
    config: PipelineConfig
) -> Result<Bool, Str> {
    remaining := budget.total - budget.spent

    if remaining <= 0.0 {
        return err("budget exhausted")
    }
    if action.estimated_cost > budget.max_single_action {
        return err("action cost exceeds per-action limit")
    }
    if action.estimated_cost > remaining {
        return err("action cost exceeds remaining budget")
    }

    if action.risk_score > config.max_risk { ok(false) } else { ok(true) }
}
```

The contracts encode safety invariants: risk scores are normalized, budget
tracking is consistent, and the return type forces callers to handle all three
outcomes (approved, denied, error).

### The Execution Engine with Retry

```vibe
@intent "Execute a single action with retry logic and budget tracking"
@require config.max_retries > 0
@effect concurrency
@effect net
@effect alloc
pub execute_with_retry(
    action: Action,
    mut budget: Budget,
    config: PipelineConfig,
    result_ch: Chan<ActionResult>
) -> Result<ActionResult, Str> {
    approved := can_execute(action, budget, config)?

    if !approved {
        denied := ActionResult {
            action: action, outcome: "denied: risk too high",
            actual_cost: 0.0, approved: false,
        }
        result_ch.send(denied)
        return ok(denied)
    }

    mut attempts := 0
    mut last_error := ""

    for attempts < config.max_retries {
        response_ch := chan(1)
        go { response_ch.send(perform_action(action)) }

        select {
            case result := response_ch.recv() => {
                match result {
                    ok(outcome) => {
                        budget.spent = budget.spent + action.estimated_cost
                        success := ActionResult {
                            action: action, outcome: outcome,
                            actual_cost: action.estimated_cost, approved: true,
                        }
                        result_ch.send(success)
                        return ok(success)
                    }
                    err(e) => { last_error = e; attempts = attempts + 1 }
                }
            }
            case after config.timeout_ms => {
                last_error = "timeout"; attempts = attempts + 1
            }
        }
    }

    err("action failed after " + config.max_retries.to_string() + " attempts")
}
```

This function combines contracts, effects, error handling with `?`, and
concurrency (channels with select and timeout) in a single cohesive design.

### The Pipeline Orchestrator

```vibe
@intent "Process a batch of AI-proposed actions through the safety pipeline"
@require actions.len() > 0
@ensure result.len() == actions.len()
@effect concurrency
@effect net
@effect alloc
pub run_pipeline(
    actions: List<Action>,
    config: PipelineConfig,
    initial_budget: Budget
) -> List<ActionResult> {
    result_ch := chan(actions.len())
    mut budget := initial_budget

    for action in actions {
        match execute_with_retry(action, budget, config, result_ch) {
            ok(r) => { budget.spent = budget.spent + r.actual_cost }
            err(msg) => {
                result_ch.send(ActionResult {
                    action: action, outcome: "error: " + msg,
                    actual_cost: 0.0, approved: false,
                })
            }
        }
    }

    mut results: List<ActionResult> := []
    mut collected := 0
    for collected < actions.len() {
        results.append(result_ch.recv())
        collected = collected + 1
    }
    results
}
```

The postcondition guarantees every action gets a result — approved, denied, or
failed. Budget is threaded sequentially so spending is tracked accurately.

---

## 13.2 The Guardrail Pattern

The guardrail pattern generalizes the safety checks from Section 13.1. A
guardrail is a function that sits between a request and its execution, deciding
whether the request should proceed.

```vibe
type GuardrailDecision {
    Approve
    Deny(reason: Str)
    Escalate(reason: Str)
}

@intent "Evaluate whether a request should proceed based on policy"
pub check_guardrail(request: Request, policy: Policy) -> GuardrailDecision {
    if request.risk_level > policy.max_risk {
        return Escalate("risk " + request.risk_level.to_string() + " exceeds threshold")
    }
    if request.cost > policy.remaining_budget {
        return Deny("insufficient budget")
    }
    if policy.blocked_actions.contains(request.action_type) {
        return Deny("action type '" + request.action_type + "' is blocked")
    }
    Approve
}
```

### Composing Guardrails

Multiple guardrails chain together — each one narrows the set of allowed
actions:

```vibe
@intent "Run a request through all guardrails, stopping at first denial"
@require guardrails.len() > 0
pub check_all_guardrails(
    request: Request,
    guardrails: List<Guardrail>
) -> GuardrailDecision {
    for guardrail in guardrails {
        match guardrail.check(request) {
            Approve => { }
            Deny(reason) => { return Deny(reason) }
            Escalate(reason) => { return Escalate(reason) }
        }
    }
    Approve
}
```

### Contract-Enforced Safety Boundaries

The real power comes from contracts that encode safety invariants the compiler
checks:

```vibe
@intent "Execute a guarded action, ensuring budget is never exceeded"
@require budget.spent <= budget.total
@ensure budget.spent <= budget.total
@effect net
@effect alloc
pub guarded_execute(
    action: Action,
    mut budget: Budget,
    policy: Policy
) -> Result<ActionResult, Str> {
    match check_guardrail(action_to_request(action), policy) {
        Approve => {
            result := perform_action(action)?
            budget.spent = budget.spent + action.estimated_cost
            ok(ActionResult {
                action: action, outcome: result,
                actual_cost: action.estimated_cost, approved: true,
            })
        }
        Deny(reason) => {
            ok(ActionResult {
                action: action, outcome: "denied: " + reason,
                actual_cost: 0.0, approved: false,
            })
        }
        Escalate(reason) => err("escalation required: " + reason)
    }
}
```

The postcondition `budget.spent <= budget.total` is a critical safety invariant.
If any code path could cause spending to exceed the budget, the contract system
catches it during testing.

---

## 13.3 Pipeline Processing

Pipelines connect processing stages with channels. Each stage runs concurrently,
and data flows through typed channels from one stage to the next.

### A Three-Stage Pipeline

```vibe
module data_pipeline

type RawRecord { id: Str, payload: Str, timestamp: Int }
type ValidatedRecord { id: Str, payload: Str, timestamp: Int, checksum: Str }
type EnrichedRecord {
    id: Str, payload: Str, timestamp: Int,
    checksum: Str, category: Str, priority: Int,
}

@intent "Validate raw records and compute checksums"
@effect concurrency
@effect alloc
pub validate_stage(
    input: Chan<RawRecord>,
    output: Chan<Result<ValidatedRecord, Str>>
) -> Int {
    mut processed := 0
    for record in input {
        if record.payload.len() == 0 {
            output.send(err("empty payload for record " + record.id))
        } else if record.timestamp <= 0 {
            output.send(err("invalid timestamp for record " + record.id))
        } else {
            output.send(ok(ValidatedRecord {
                id: record.id, payload: record.payload,
                timestamp: record.timestamp,
                checksum: compute_checksum(record.payload),
            }))
        }
        processed = processed + 1
    }
    output.close()
    processed
}

@intent "Enrich validated records with category and priority"
@effect concurrency
@effect alloc
pub enrich_stage(
    input: Chan<Result<ValidatedRecord, Str>>,
    output: Chan<Result<EnrichedRecord, Str>>
) -> Int {
    mut enriched := 0
    for item in input {
        match item {
            ok(record) => {
                category := classify_payload(record.payload)
                priority := if category == "critical" { 1 }
                    else if category == "warning" { 2 } else { 3 }
                output.send(ok(EnrichedRecord {
                    id: record.id, payload: record.payload,
                    timestamp: record.timestamp, checksum: record.checksum,
                    category: category, priority: priority,
                }))
                enriched = enriched + 1
            }
            err(e) => output.send(err(e))
        }
    }
    output.close()
    enriched
}
```

The enrichment stage passes errors through unchanged — a common pipeline
pattern where validation errors skip subsequent stages.

### Wiring the Pipeline

```vibe
@intent "Run the full data pipeline with concurrent stages"
@require records.len() > 0
@ensure result.total == records.len()
@effect concurrency
@effect alloc
pub run_data_pipeline(records: List<RawRecord>) -> PipelineReport {
    raw_ch := chan(100)
    validated_ch := chan(100)
    enriched_ch := chan(100)

    go validate_stage(raw_ch, validated_ch)
    go enrich_stage(validated_ch, enriched_ch)

    for record in records {
        raw_ch.send(record)
    }
    raw_ch.close()

    mut successes: List<EnrichedRecord> := []
    mut failures: List<Str> := []
    for item in enriched_ch {
        match item {
            ok(record) => successes.append(record)
            err(msg) => failures.append(msg)
        }
    }

    PipelineReport {
        total: successes.len() + failures.len(),
        succeeded: successes.len(), failed: failures.len(),
        records: successes, errors: failures,
    }
}
```

The postcondition `result.total == records.len()` guarantees nothing is silently
dropped. Backpressure flows naturally through the bounded channels.

---

## 13.4 HTTP Service Pattern

Network services combine effects, error handling, and contracts:

```vibe
type HttpRequest { method: Str, path: Str, headers: Map<Str, Str>, body: Str }
type HttpResponse { status: Int, headers: Map<Str, Str>, body: Str }

@intent "Route an HTTP request to the appropriate handler"
@require request.method.len() > 0
@ensure result.status >= 100
@ensure result.status < 600
@effect io
@effect net
@effect alloc
pub handle_request(request: HttpRequest) -> HttpResponse {
    match request.path {
        "/health" => handle_health(request)
        "/api/users" => handle_users(request)
        _ => HttpResponse { status: 404, headers: {}, body: "{\"error\": \"not found\"}" }
    }
}
```

The postconditions on status codes ensure every handler returns a valid HTTP
status. Effect declarations make the operational footprint explicit.

### Bounded Concurrent Request Processing

The semaphore pattern uses a channel to limit concurrency:

```vibe
@intent "Process incoming requests concurrently with bounded parallelism"
@require max_concurrent > 0
@effect concurrency
@effect io
@effect net
@effect alloc
pub serve(
    requests: Chan<HttpRequest>,
    responses: Chan<HttpResponse>,
    max_concurrent: Int
) -> Int {
    semaphore := chan(max_concurrent)
    mut w := 0
    for w < max_concurrent { semaphore.send(true); w = w + 1 }

    mut served := 0
    for request in requests {
        _ := semaphore.recv()
        go {
            response := handle_request(request)
            responses.send(response)
            semaphore.send(true)
        }
        served = served + 1
    }
    served
}
```

Each request consumes a token before spawning a handler task, and the task
returns the token when done. This bounds memory usage and prevents overload.

---

## 13.5 Data Transformation Pipelines

Data transformation is where contracts shine — each stage has preconditions on
its input and postconditions on its output:

```vibe
@intent "Apply a transformation to each item in parallel, preserving count"
@require items.len() > 0
@require num_workers > 0
@ensure result.len() == items.len()
@effect concurrency
@effect alloc
pub parallel_transform(
    items: List<Input>,
    transform_fn: (Input) -> Output,
    num_workers: Int
) -> List<Output> {
    jobs := chan(items.len())
    results := chan(items.len())

    mut w := 0
    for w < num_workers {
        go { for item in jobs { results.send(transform_fn(item)) } }
        w = w + 1
    }

    for item in items { jobs.send(item) }
    jobs.close()

    mut output: List<Output> := []
    mut collected := 0
    for collected < items.len() {
        output.append(results.recv())
        collected = collected + 1
    }
    output
}
```

The postcondition `result.len() == items.len()` is the critical invariant: the
parallel transformation must produce exactly one output for each input. If a
worker crashes or a channel operation fails, this contract catches it in testing.

---

## 13.6 Testing Strategies for Complex Programs

### Unit Testing with `@examples`

Embed examples directly in contracts for the simplest testing:

```vibe
@intent "Compute the risk score for an action based on its properties"
@examples {
    compute_risk(safe_action) == 0.1
    compute_risk(moderate_action) == 0.5
    compute_risk(dangerous_action) == 0.9
}
@ensure result >= 0.0
@ensure result <= 1.0
pub compute_risk(action: Action) -> Float {
    base := match action.category {
        "read" => 0.1
        "write" => 0.3
        "delete" => 0.7
        "admin" => 0.9
        _ => 0.5
    }
    if action.payload.len() > 10000 { min(base + 0.2, 1.0) } else { base }
}
```

### Integration Testing

For effectful code, write tests that exercise the full stack:

```vibe
pub test_full_pipeline() -> Int {
    @effect concurrency
    @effect alloc

    records := [
        RawRecord { id: "1", payload: "valid data", timestamp: 1000 },
        RawRecord { id: "2", payload: "", timestamp: 2000 },
        RawRecord { id: "3", payload: "more data", timestamp: -1 },
    ]

    report := run_data_pipeline(records)
    assert(report.total == 3, "should process all records")
    assert(report.succeeded == 1, "one record should succeed")
    assert(report.failed == 2, "two should fail")
    0
}
```

### Testing Concurrent Code

Concurrent tests must account for non-deterministic scheduling. Test aggregate
properties (sums, counts, set membership) rather than ordering:

```vibe
pub test_parallel_sum() -> Int {
    @effect concurrency
    @effect alloc

    input := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    result := parallel_transform(input, |x| { x * 2 }, 4)

    mut sum := 0
    for v in result { sum = sum + v }
    assert(sum == 110, "sum of doubled values should be 110")
    assert(result.len() == 10, "should have 10 results")
    0
}
```

Use channels to synchronize expectations, and test timeouts with `select`:

```vibe
pub test_timeout_handling() -> Int {
    @effect concurrency

    slow_ch := chan(1)
    select {
        case _ := slow_ch.recv() => { assert(false, "should not receive") }
        case after 100ms => { assert(true, "timeout fired as expected") }
    }
    0
}
```

Guardrail functions are pure, making them ideal for exhaustive unit testing with
no setup, no mocking, and no teardown.

---

## 13.7 Summary

This chapter demonstrated how VibeLang's features combine in real programs:

- **The agentic AI pipeline** showed contracts enforcing safety on AI-proposed
  actions, channels coordinating execution, and error handling for partial
  failures.
- **The guardrail pattern** generalized safety checks into composable,
  contract-enforced gates.
- **Pipeline processing** connected concurrent stages with typed channels and
  postconditions guaranteeing no records are dropped.
- **The HTTP service pattern** combined effect-tracked I/O and bounded
  concurrency.
- **Data transformation pipelines** used contracts to maintain invariants with
  parallel execution.
- **Testing strategies** showed `@examples` for pure functions, integration
  tests for effectful code, and aggregate assertions for concurrent programs.

The common thread: contracts, effects, and concurrency primitives work together
to make program behavior explicit, verifiable, and auditable.

---

Next: Chapter 14 takes you inside the compiler to understand how VibeLang
source code becomes a native binary.
