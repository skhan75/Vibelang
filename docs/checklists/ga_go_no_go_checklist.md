# V1 GA Go/No-Go Checklist (Canonical)

Date: `<set per cycle>`  
Candidate: `<set per cycle>`

## Hard Go Criteria

- [ ] `P0` technical gates are implemented and validated.
- [ ] `P1` exceptions are documented and approved.
- [ ] GA evidence bundle artifacts exist (phase exit audit, freeze manifest, GA announcement).
- [ ] Hosted RC run URLs are attached (not `local://` placeholders).
- [ ] Hosted release workflows for candidate are linked in checklist/dashboard.
- [ ] Public release payload is published (tag + release notes + signed artifacts).

## Immediate No-Go Triggers

- Any unresolved `P0` blocker.
- Missing hosted workflow evidence URLs for the candidate cycle.
- Missing trust artifacts (checksums/signatures/provenance/SBOM) in published release payload.
- Known limitations or breaking changes absent from release notes.

## Final Decision Rule

- **GO** only if every hard go criterion is checked.
- Otherwise **NO-GO** and carry blocker(s) into next RC cycle.

