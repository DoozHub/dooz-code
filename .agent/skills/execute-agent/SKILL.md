---
name: execute-agent
description: Code generation and file modification agent
---

# Execute Agent

> Generates code artifacts based on analyzed context and task requirements.

## Purpose

The Execute Agent takes analyzed context (repository patterns, conventions, dependencies) and task specifications, then generates production-ready code artifacts.

## Trigger

This agent activates when:
- Mode evaluation includes `EXECUTE`
- Analyze Agent has prepared context for the task

## Input

| Field | Type | Description |
|-------|------|-------------|
| `task` | Task | Single task from the DAG |
| `context` | AnalyzedContext | Repository context with patterns |
| `constraints` | Vec<Constraint> | Technical/business constraints |

## Output

```json
{
  "artifacts": [
    {
      "path": "string",
      "action": "create|modify|delete",
      "content": "string",
      "language": "string"
    }
  ],
  "execution_log": {
    "steps": [],
    "decisions": [],
    "patterns_followed": []
  }
}
```

## Execution Steps

1. **Load Context** — Retrieve analyzed patterns and conventions
2. **Parse Task** — Understand specific requirements
3. **Plan Files** — Determine which files to create/modify
4. **Generate Code** — Write code following repository patterns
5. **Apply Changes** — Create file artifacts
6. **Log Decisions** — Record all choices made

## Constraints

- **Scope immutability** — Cannot expand beyond task scope
- **Pattern adherence** — Must follow existing conventions
- **No feature invention** — Only implements specified requirements

## LLM Integration

Uses the multi-provider orchestrator:

```rust
let orchestrator = MultiProviderOrchestrator::with_claude(api_key)?;
let result = orchestrator.generate(&unified_request).await?;
```

## Handoff

Output is passed to **Review Agent**:

```rust
ModeHandoff::new(Mode::Execute, Mode::Review, artifacts_json)
```
