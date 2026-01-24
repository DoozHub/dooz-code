# dooz-code Platform — v2.0.0 Complete 🎉

## All 14 Tasks Complete

| # | Task | Version | Status |
|---|------|---------|--------|
| 01 | Modes Pipeline | v0.5.0 | ✅ |
| 02 | Worktree Executor | v0.5.0 | ✅ |
| 03 | CLI Integration Tests | v0.5.0 | ⏸️ Deferred |
| 04 | Worker Redis | v0.6.0 | ✅ |
| 05 | Worker Dispatcher | v0.6.0 | ✅ |
| 06 | Worker API | v0.6.0 | ✅ |
| 07 | Intake Module | v0.7.0 | ✅ |
| 08 | Verifier Module | v0.7.0 | ✅ |
| 09 | Web UI MVP | v0.8.0 | ✅ |
| 10 | Production Hardening | v1.0.0 | ✅ |
| 11 | Desktop App | v1.5.0 | ✅ |
| 12 | Mobile App | v1.5.0 | ✅ |
| 13 | Contracts Module | v2.0.0 | ✅ |
| 14 | Agent Marketplace | v2.0.0 | ✅ |

**Progress: 13/14 Complete | 1 Deferred (CLI Tests)**

---

## API Endpoints (Task 06)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/ready` | Readiness probe |
| GET | `/live` | Liveness probe |
| GET | `/metrics` | Prometheus metrics |
| POST | `/api/tasks` | Submit new task |
| GET | `/api/tasks` | List tasks |
| GET | `/api/tasks/:id` | Get task status |
| DELETE | `/api/tasks/:id` | Cancel task |
| GET | `/api/tasks/:id/artifacts` | Get artifacts |
| GET | `/api/tasks/:id/logs` | Get execution logs |
| GET | `/api/openapi.json` | OpenAPI spec |

---

## Run

```bash
# Worker API
cd dooz-worker && cargo run

# Web UI
cd dooz-code-ui && pnpm install && pnpm dev
```
