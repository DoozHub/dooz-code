# Task 04: Worker Redis Integration

**Version:** v0.6.0  
**Dependency:** 03-cli-integration  
**Status:** ✅ Complete

---

## Objective

Make dooz-worker properly handle Redis connections with fallbacks.

---

## Tasks

- [ ] Add Redis connection pool
- [ ] Implement connection health checks
- [ ] Add graceful degradation (in-memory fallback)
- [ ] Add Redis reconnection logic
- [ ] Add connection metrics
- [ ] Add configuration via environment variables

---

## Files to Modify

| File | Action |
|------|--------|
| `dooz-worker/src/queue.rs` | Add connection pool |
| `dooz-worker/src/lib.rs` | Add config struct |
| `dooz-worker/src/main.rs` | Init from env |

---

## Acceptance Criteria

- [ ] Worker starts without Redis (fallback mode)
- [ ] Worker reconnects after Redis restart
- [ ] Connection metrics available
