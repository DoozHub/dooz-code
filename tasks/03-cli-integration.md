# Task 03: CLI Integration Tests

**Version:** v0.5.0  
**Dependency:** 02-worktree-executor  
**Status:** ⏸️ Deferred

---

## Objective

Add end-to-end integration tests for the CLI to verify the complete pipeline works.

---

## Tasks

- [ ] Create test fixtures (sample repos, work packages)
- [ ] Add `evaluate` command test
- [ ] Add `plan` command test
- [ ] Add `execute` command test with worktree
- [ ] Add `analyze` command test
- [ ] Add pipeline integration test
- [ ] Add CI workflow for tests

---

## Files to Create

| File | Description |
|------|-------------|
| `tests/integration/` | Integration test directory |
| `tests/fixtures/` | Test repositories and packages |
| `.github/workflows/test.yml` | CI workflow |

---

## Acceptance Criteria

- [ ] All CLI commands have integration tests
- [ ] Tests run in CI
- [ ] Coverage > 60%
