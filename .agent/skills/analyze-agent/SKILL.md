---
name: analyze-agent
description: Repository context extraction and pattern detection agent
---

# Analyze Agent

> Extracts repository context and patterns for each task.

## Purpose

The Analyze Agent examines the repository to understand existing patterns, conventions, dependencies, and file structures before code generation begins.

## Input

| Field | Type | Description |
|-------|------|-------------|
| `task` | Task | Task requiring context |
| `repo_path` | Path | Repository root |

## Output

```json
{
  "patterns": [],
  "conventions": {},
  "dependencies": [],
  "related_files": [],
  "integration_points": []
}
```

## Handoff

Output passed to **Execute Agent** for code generation.
