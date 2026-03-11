// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

use vibe_diagnostics::{Diagnostic, Diagnostics, Severity};

use crate::effect_propagation::FunctionEffectSummary;

pub fn emit_effect_diagnostics(
    summaries: &[FunctionEffectSummary],
    transitive_effects: &BTreeMap<String, BTreeSet<String>>,
    diagnostics: &mut Diagnostics,
) {
    for summary in summaries {
        let observed = transitive_effects
            .get(&summary.name)
            .cloned()
            .unwrap_or_else(BTreeSet::new);

        for observed_effect in &observed {
            if !summary.declared_effects.contains(observed_effect) {
                diagnostics.push(Diagnostic::new(
                    "E3002",
                    Severity::Warning,
                    format!(
                        "observed effect `{observed_effect}` is not declared in `@effect` (including transitive calls)"
                    ),
                    summary.span,
                ));
            }
        }

        for declared in &summary.declared_effects {
            if !observed.contains(declared) {
                diagnostics.push(Diagnostic::new(
                    "E3003",
                    Severity::Info,
                    format!("declared effect `{declared}` was not observed"),
                    summary.span,
                ));
            }
        }

        for callee in &summary.direct_calls {
            let Some(callee_effects) = transitive_effects.get(callee) else {
                continue;
            };
            for effect in callee_effects {
                if !summary.declared_effects.contains(effect) {
                    diagnostics.push(Diagnostic::new(
                        "E3101",
                        Severity::Warning,
                        format!(
                            "call to `{callee}` requires transitive effect `{effect}` to be declared"
                        ),
                        summary.span,
                    ));
                }
            }
        }
    }
}
