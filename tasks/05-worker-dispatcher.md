# Task 05: Worker Dispatcher Integration

**Version:** v0.6.0  
**Dependency:** 04-worker-redis  
**Status:** ✅ Complete

---

## Objective

Connect dooz-worker dispatcher to actually call dooz-code for task execution.

---

## Tasks

- [ ] Import dooz-code as library dependency
- [ ] Convert Job to dooz-code Task
- [ ] Call dooz-code execute pipeline
- [ ] Collect artifacts from execution
- [ ] Handle execution errors
- [ ] Add progress callbacks during execution

---

## Files to Modify

| File | Action |
|------|--------|
| `dooz-worker/Cargo.toml` | Add dooz-code dep |
| `dooz-worker/src/dispatcher.rs` | Real execution |

---

## Acceptance Criteria

- [ ] Jobs trigger actual code generation
- [ ] Artifacts are collected from execution
- [ ] Progress updates during execution
