# Modeling Shapes and OOP Patterns

This folder shows how to model domain "shapes" with the currently stable VibeLang surface.

## What is runnable today

- Nominal type declarations (struct-like records):
  - `74_type_point_basics.yb`
  - `76_type_mixed_fields.yb`
- Enum + match control flow:
  - `75_enum_match_basics.yb`
- Shape-like records via `Map<Str, Int>` (legacy fallback pattern):
  - `71_shape_with_map_record.yb`
  - `72_shape_contracts_and_validation.yb`
- Composition-based behavior layering (recommended over inheritance-style hierarchies in current surface):
  - `73_composition_over_inheritance.yb`

## What is not fully available yet

- Class-based inheritance
- Trait/interface polymorphism

See `examples/FEATURE_GAPS_CHECKLIST.md` for the canonical implementation tracker.
