# Task 02: Worktree Executor Implementation

**Version:** v0.5.0  
**Dependency:** 01-modes-pipeline  
**Status:** ✅ Complete

---

## Objective

Implement real code execution within git worktrees so each task runs in isolation.

---

## Tasks

- [ ] Connect `WorktreeExecutor` to dooz-code `CodeExecutor`
- [ ] Pass repository patterns to LLM for code generation
- [ ] Implement file writing within worktree
- [ ] Add commit and push logic
- [ ] Implement conflict detection
- [ ] Add worktree cleanup on failure
- [ ] Add execution metrics collection

---

## Files to Modify

| File | Action | Description |
|------|--------|-------------|
| `src/worktree/executor.rs` | Modify | Real execution logic |
| `src/worktree/pool.rs` | Modify | Better error handling |
| `src/executor/mod.rs` | Modify | Add path-based execution |

---

## Acceptance Criteria

- [ ] Task creates actual code files in worktree
- [ ] Changes are committed to task branch
- [ ] Worktree is cleaned up after completion
- [ ] Failures don't leave orphan worktrees
