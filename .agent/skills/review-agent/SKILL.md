---
name: review-agent
description: Self-validation and iteration agent
---

# Review Agent

> Validates generated artifacts against acceptance criteria.

## Purpose

The Review Agent checks generated code artifacts for correctness, pattern adherence, and acceptance criteria compliance before verification.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `artifacts` | Vec<Artifact> | Generated code |
| `criteria` | Vec<Criterion> | Acceptance criteria |
| `patterns` | Vec<Pattern> | Expected patterns |

## Output

```json
{
  "status": "pass|fail",
  "issues": [],
  "suggestions": []
}
```

## Decision

- **Pass** → Forward to Verify Agent
- **Fail** → Return to Execute Agent with issues for iteration
