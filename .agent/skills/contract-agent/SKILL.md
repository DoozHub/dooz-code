---
name: contract-agent
description: Sub-work dispatch to specialized external agents
---

# Contract Agent

> Dispatches specialized sub-tasks to external agents.

## Purpose

The Contract Agent handles complex tasks that require specialized expertise by splitting work and dispatching sub-contracts to external agents (domain specialists, language experts, etc.).

## Trigger

This agent activates when:
- Task complexity exceeds threshold
- Specialized domain knowledge required
- Explicit sub-contract request in task

## Input

| Field | Type | Description |
|-------|------|-------------|
| `task` | Task | Complex task requiring splitting |
| `available_agents` | Vec<Agent> | Registered external agents |
| `constraints` | Constraints | Budget, time, quality constraints |

## Output

```json
{
  "contracts": [
    {
      "contract_id": "string",
      "agent_id": "string",
      "sub_task": {},
      "budget_usd": 0.0,
      "deadline": "ISO8601",
      "status": "pending|active|completed|failed"
    }
  ],
  "aggregation_strategy": "merge|replace|review",
  "quality_threshold": 0.0-1.0
}
```

## Agent Registry

External agents register with capabilities:

```json
{
  "agent_id": "security-expert-001",
  "name": "Security Auditor",
  "capabilities": ["security-review", "penetration-testing"],
  "cost_per_task": 0.05,
  "average_latency_ms": 30000,
  "quality_rating": 0.95
}
```

## Execution Steps

1. **Analyze Complexity** — Determine if sub-contracting needed
2. **Split Tasks** — Divide into sub-tasks per specialty
3. **Match Agents** — Find suitable agents for each sub-task
4. **Dispatch Contracts** — Send work to external agents
5. **Monitor Progress** — Track contract completion
6. **Aggregate Results** — Combine outputs
7. **Quality Check** — Verify aggregated result

## Constraints

- **Budget limits** — Cannot exceed allocated budget
- **Quality thresholds** — Results must meet minimum score
- **Timeout handling** — Fallback if agent unresponsive

## Usage

```bash
dooz-code contract --task ./complex-task.json --agents ./registry.json
```
