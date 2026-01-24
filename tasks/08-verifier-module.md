# Task 08: Verifier Module

**Version:** v0.7.0  
**Dependency:** 07-intake-module  
**Status:** ✅ Complete

---

## Objective

Implement compliance verification against specifications.

---

## Tasks

- [ ] Create `src/verifier/mod.rs` module
- [ ] Implement spec mismatch detection
- [ ] Add missing feature detection
- [ ] Add hallucination detection
- [ ] Check edge case coverage
- [ ] Verify test existence
- [ ] Add verifier CLI command

---

## Files to Create

| File | Description |
|------|-------------|
| `src/verifier/mod.rs` | Module entry |
| `src/verifier/checks.rs` | Verification checks |
| `src/verifier/report.rs` | Report generation |

---

## Acceptance Criteria

- [ ] `dooz-code verify --spec ./spec.json --artifacts ./` outputs report
- [ ] Detects spec mismatches
- [ ] Identifies missing features
