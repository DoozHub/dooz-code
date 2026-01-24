---
name: plan-agent
description: Technical Specification to Task DAG generation agent
---

# Plan Agent

> Decomposes Technical Specifications into executable Task DAGs.

## Purpose

The Plan Agent receives a Technical Specification and produces an ordered Task DAG (Directed Acyclic Graph) where each task has clear dependencies, acceptance criteria, and module assignments.

## Trigger

This agent activates when:
- Mode evaluation includes `PLAN`
- Intake Agent has produced a Technical Specification

## Input

| Field | Type | Description |
|-------|------|-------------|
| `spec` | TechnicalSpec | Output from Intake Agent |
| `repo_context` | RepoContext | Repository structure information |

## Output

```json
{
  "dag_id": "string",
  "tasks": [
    {
      "task_id": "TASK-001",
      "title": "string",
      "module": "string",
      "type": "feature|bugfix|refactor|test",
      "dependencies": ["TASK-000"],
      "acceptance_criteria": ["string"],
      "estimated_complexity": 1-10,
      "files_affected": ["string"]
    }
  ],
  "execution_order": ["TASK-001", "TASK-002"],
  "parallel_groups": [["TASK-003", "TASK-004"]]
}
```

## Execution Steps

1. **Analyze Spec** — Understand required entities, APIs, flows
2. **Identify Modules** — Map work to repository modules
3. **Decompose Tasks** — Break spec into atomic tasks
4. **Resolve Dependencies** — Determine task ordering
5. **Estimate Complexity** — Score each task
6. **Generate DAG** — Produce final execution graph

## Constraints

- **Atomic tasks** — Each task should be independently executable
- **Clear dependencies** — No implicit task ordering
- **Mandatory criteria** — Every task must have acceptance criteria

## Example

```bash
dooz-code plan --spec ./auth-spec.json --repo ./my-project
```

## Handoff

Output is passed to **Analyze Agent**:

```rust
ModeHandoff::new(Mode::Plan, Mode::Analyze, dag_json)
```
