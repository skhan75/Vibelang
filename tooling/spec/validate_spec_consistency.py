#!/usr/bin/env python3
import sys
from pathlib import Path


def fail(msg: str) -> None:
    print(f"spec consistency validation failed: {msg}")
    sys.exit(1)


def require_file(path: Path) -> None:
    if not path.exists():
        fail(f"missing required file: {path}")


def require_contains(text: str, needle: str, context: str) -> None:
    if needle not in text:
        fail(f"missing `{needle}` in {context}")


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    spec_dir = repo_root / "docs" / "spec"

    required = [
        spec_dir / "README.md",
        spec_dir / "syntax.md",
        spec_dir / "semantics.md",
        spec_dir / "grammar_v0_1.ebnf",
        spec_dir / "grammar_v1_0.ebnf",
        spec_dir / "phase1_resolved_decisions.md",
        spec_dir / "type_system.md",
        spec_dir / "numeric_model.md",
        spec_dir / "mutability_model.md",
        spec_dir / "strings_and_text.md",
        spec_dir / "containers.md",
        spec_dir / "control_flow.md",
        spec_dir / "concurrency_and_scheduling.md",
        spec_dir / "async_await_and_threads.md",
        spec_dir / "ownership_sendability.md",
        spec_dir / "memory_model_and_gc.md",
        spec_dir / "error_model.md",
        spec_dir / "module_and_visibility.md",
        spec_dir / "abi_and_ffi.md",
        spec_dir / "spec_coverage_matrix.md",
    ]
    for path in required:
        require_file(path)

    syntax = (spec_dir / "syntax.md").read_text()
    semantics = (spec_dir / "semantics.md").read_text()
    grammar_v1 = (spec_dir / "grammar_v1_0.ebnf").read_text()
    phase1 = (spec_dir / "phase1_resolved_decisions.md").read_text()
    readme = (spec_dir / "README.md").read_text()

    # Grammar-level required forms for v1 target.
    grammar_needles = [
        "match_stmt",
        "break_stmt",
        "continue_stmt",
        "thread_stmt",
        "async_opt",
        "unary_prefix",
        "\"await\"",
        "nullable_suffix_opt",
        "\"none\"",
    ]
    for needle in grammar_needles:
        require_contains(grammar_v1, needle, "grammar_v1_0.ebnf")

    # Syntax-level alignment with control flow and optional typing.
    syntax_needles = [
        "`match`",
        "`break`",
        "`continue`",
        "`thread`",
        "`await`",
        "`T?`",
        "`none`",
        "Contracts MUST appear",
    ]
    for needle in syntax_needles:
        require_contains(syntax, needle, "syntax.md")

    # Semantics-level alignment for optional and control flow references.
    semantics_needles = [
        "`T?`",
        "`match`",
        "`break`",
        "`continue`",
        "async",
        "thread",
    ]
    for needle in semantics_needles:
        require_contains(semantics, needle, "semantics.md")

    # Compatibility appendix must exist.
    require_contains(
        phase1,
        "Compatibility Appendix",
        "phase1_resolved_decisions.md",
    )

    # Spec index must reference v1 grammar and archived v0.1 freeze.
    require_contains(readme, "grammar_v1_0.ebnf", "docs/spec/README.md")
    require_contains(readme, "grammar_v0_1.ebnf", "docs/spec/README.md")

    print("spec consistency validation passed")


if __name__ == "__main__":
    main()
