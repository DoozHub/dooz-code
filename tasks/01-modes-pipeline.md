# Task 01: Modes Pipeline Integration

**Version:** v0.5.0  
**Dependency:** None  
**Status:** ✅ Complete

---

## Objective

Wire the multi-mode pipeline so that task prompts flow through EVALUATE → INTAKE → PLAN → ANALYZE → EXECUTE → REVIEW → VERIFY sequentially.

---

## Tasks

- [ ] Enhance `ModeEvaluator` with LLM-based prompt analysis
- [ ] Create `ModePipeline` orchestrator that chains mode agents
- [ ] Implement `ModeAgent` trait for consistent agent interface
- [ ] Create mode-specific agents (IntakeAgent, PlanAgent, etc.)
- [ ] Add pipeline state machine for mode transitions
- [ ] Wire `execute()` in lib.rs to use mode pipeline
- [ ] Add pipeline tests

---

## Files to Modify

| File | Action | Description |
|------|--------|-------------|
| `src/modes/mod.rs` | Modify | Add ModePipeline, ModeAgent trait |
| `src/modes/pipeline.rs` | Create | Pipeline orchestration logic |
| `src/modes/agents/` | Create | Mode-specific agent implementations |
| `src/lib.rs` | Modify | Wire pipeline to execute() |

---

## Acceptance Criteria

- [ ] `dooz-code evaluate "Add auth"` returns mode sequence
- [ ] `dooz-code execute --pipeline` runs all modes sequentially
- [ ] Each mode produces handoff data for next mode
- [ ] Pipeline state is observable
