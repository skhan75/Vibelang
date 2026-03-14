// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Prompt templates for AI sidecar analysis.
//! Each prompt version is tracked for cache invalidation.

pub const PROMPT_VERSION: u32 = 1;

pub const SYSTEM_PROMPT: &str = r#"You are a VibeLang code analysis assistant. VibeLang is a statically typed, natively compiled language with first-class contracts.

Key VibeLang annotations:
- `@intent "..."` — declares what a function should do in natural language
- `@require expr` — precondition checked at function entry
- `@ensure expr` — postcondition checked at function exit; `.` refers to the return value, `old(x)` snapshots pre-call state
- `@examples { f(args) => expected }` — executable test cases
- `@effect tag` — side effects: io, alloc, mut_state, concurrency, nondet

You analyze VibeLang functions and respond ONLY with valid JSON. No markdown, no explanation outside the JSON."#;

pub fn drift_detection_prompt(
    function_name: &str,
    intent_text: &str,
    source_code: &str,
    effects_declared: &[String],
    contracts: &str,
) -> String {
    let effects = if effects_declared.is_empty() {
        "none declared".to_string()
    } else {
        effects_declared.join(", ")
    };

    format!(
        r#"Analyze whether this VibeLang function's implementation matches its stated @intent.

Function: `{function_name}`
Intent: "{intent_text}"
Declared effects: {effects}
Existing contracts:
{contracts}

Source code:
```
{source_code}
```

Respond with this exact JSON structure:
{{
  "aligned": <true if implementation matches intent, false if it drifts>,
  "confidence": <0.0 to 1.0 — how confident you are in your assessment>,
  "rationale": "<one sentence explaining your reasoning>",
  "drift_description": "<if not aligned, describe the specific drift; empty string if aligned>"
}}"#
    )
}

pub fn contract_suggestion_prompt(
    function_name: &str,
    source_code: &str,
    existing_contracts: &str,
) -> String {
    format!(
        r#"Suggest @require (precondition) and @ensure (postcondition) contracts for this VibeLang function.

Function: `{function_name}`
Existing contracts:
{existing_contracts}

Source code:
```
{source_code}
```

Rules for contracts:
- @require expressions are checked at function entry
- @ensure expressions are checked at function exit
- In @ensure, `.` refers to the return value, `old(x)` snapshots the pre-call value of x
- Only suggest contracts that are meaningful and non-trivial
- Do not repeat existing contracts

Respond with this exact JSON structure:
{{
  "suggestions": [
    {{
      "type": "require" or "ensure",
      "expression": "<valid VibeLang expression>",
      "rationale": "<why this contract is useful>",
      "confidence": <0.0 to 1.0>
    }}
  ]
}}"#
    )
}

pub fn example_suggestion_prompt(function_name: &str, source_code: &str) -> String {
    format!(
        r#"Suggest @examples test cases for this VibeLang function.

Function: `{function_name}`

Source code:
```
{source_code}
```

Rules for @examples:
- Format: `function_name(arg1, arg2) => expected_result`
- Cover normal cases, edge cases, and boundary conditions
- Use concrete values, not variables
- Suggest 2-4 examples

Respond with this exact JSON structure:
{{
  "examples": [
    {{
      "input": "<function_name(args)>",
      "expected": "<expected return value>",
      "rationale": "<what this example tests>",
      "confidence": <0.0 to 1.0>
    }}
  ]
}}"#
    )
}

pub fn intent_suggestion_prompt(function_name: &str, source_code: &str) -> String {
    format!(
        r#"Write a concise @intent annotation for this VibeLang function.

Function: `{function_name}`

Source code:
```
{source_code}
```

Rules for @intent:
- One sentence describing what the function achieves (not how)
- Specific enough that a reviewer could detect drift
- Should mention key constraints or guarantees
- Do not use vague words like "stuff", "something", "handle"

Respond with this exact JSON structure:
{{
  "intent_text": "<the intent string without quotes>",
  "confidence": <0.0 to 1.0>,
  "rationale": "<why this intent captures the function's purpose>"
}}"#
    )
}
