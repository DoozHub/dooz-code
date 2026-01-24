# Task 07: Intake Module

**Version:** v0.7.0  
**Dependency:** 06-worker-api  
**Status:** ✅ Complete

---

## Objective

Implement PRD → Technical Spec conversion module.

---

## Tasks

- [ ] Create `src/intake/mod.rs` module structure
- [ ] Define TechnicalSpec struct
- [ ] Implement LLM-based PRD parsing
- [ ] Extract entities, APIs, UI flows
- [ ] Identify edge cases and assumptions
- [ ] Generate acceptance criteria
- [ ] Add intake CLI command

---

## Files to Create

| File | Description |
|------|-------------|
| `src/intake/mod.rs` | Module entry |
| `src/intake/parser.rs` | PRD parsing |
| `src/intake/spec.rs` | Spec generation |

---

## Acceptance Criteria

- [ ] `dooz-code intake --prd ./prd.md` outputs spec JSON
- [ ] Spec includes entities, APIs, flows
- [ ] Edge cases are identified
