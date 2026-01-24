---
name: intake-agent
description: PRD/prompt to Technical Specification conversion agent
---

# Intake Agent

> Converts PRD documents and natural language prompts into structured Technical Specifications.

## Purpose

The Intake Agent is the entry point of the dooz-code pipeline. It receives raw requirements (PRDs, feature requests, user stories) and produces a structured Technical Specification that downstream agents can consume.

## Trigger

This agent activates when:
- Mode evaluation returns `INTAKE` as the first required mode
- Input contains PRD indicators (requirement, feature request, user story)

## Input

| Field | Type | Description |
|-------|------|-------------|
| `prompt` | string | Raw PRD or requirement text |
| `context` | object | Optional repository context |
| `constraints` | array | Business/technical constraints |

## Output

```json
{
  "spec_id": "string",
  "title": "string",
  "description": "string",
  "entities": [
    {
      "name": "string",
      "type": "model|service|component",
      "attributes": []
    }
  ],
  "apis": [
    {
      "method": "GET|POST|PUT|DELETE",
      "path": "string",
      "request_body": {},
      "response": {}
    }
  ],
  "ui_flows": [
    {
      "name": "string",
      "screens": [],
      "navigation": []
    }
  ],
  "edge_cases": [
    "string"
  ],
  "assumptions": [
    "string"
  ],
  "acceptance_criteria": [
    "string"
  ]
}
```

## Execution Steps

1. **Parse Input** — Extract key requirements from raw text
2. **Identify Entities** — Detect data models, services, components
3. **Map APIs** — Define necessary endpoints
4. **Chart UI Flows** — Outline screen navigation
5. **List Edge Cases** — Identify potential failure modes
6. **Document Assumptions** — Capture implicit requirements
7. **Define Acceptance Criteria** — Measurable success conditions

## Constraints

- **No code generation** — This agent produces specs only
- **Explicit assumptions** — All assumptions must be documented
- **Ambiguity blocking** — Unclear requirements trigger clarification request

## Example Usage

```bash
dooz-code intake --prompt "Add user authentication with email/password and OAuth"
```

## Handoff

Output is passed to the **Plan Agent** via `ModeHandoff`:

```rust
ModeHandoff::new(Mode::Intake, Mode::Plan, spec_json)
```
