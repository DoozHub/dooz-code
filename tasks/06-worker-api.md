# Task 06: Worker API Completion

**Version:** v0.6.0  
**Dependency:** 05-worker-dispatcher  
**Status:** ✅ Complete

---

## Objective

Complete the HTTP API with WebSocket streaming and authentication.

---

## Tasks

- [ ] Add WebSocket endpoint for progress streaming
- [ ] Implement job cancellation endpoint
- [ ] Add basic API key authentication
- [ ] Add rate limiting
- [ ] Add CORS configuration
- [ ] Add OpenAPI documentation

---

## Files to Modify

| File | Action |
|------|--------|
| `dooz-worker/src/api.rs` | Add endpoints |
| `dooz-worker/src/monitor.rs` | WebSocket impl |

---

## Acceptance Criteria

- [ ] WebSocket streams progress in real-time
- [ ] Jobs can be cancelled
- [ ] API requires authentication
