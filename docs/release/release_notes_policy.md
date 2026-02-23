# Release Notes Policy (v1)

Date: 2026-02-22

## Required Sections

Every RC/GA release notes document must include:

- `Highlights`
- `Known Limitations`
- `Breaking Changes`

## Automation

- Source inputs: `reports/v1/release_notes_inputs.json`
- Generator: `tooling/release/generate_release_notes.py`
- Validator: `tooling/release/validate_release_notes.py`
- Output artifact: `reports/v1/release_notes_preview.md`

## Gate Requirement

Release promotion requires release-notes automation gate success and artifact
publication with the candidate evidence bundle.
