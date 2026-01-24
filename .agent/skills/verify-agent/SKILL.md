---
name: verify-agent
description: Compliance verification and quality gate agent
---

# Verify Agent

> Final compliance check before artifacts are accepted.

## Purpose

The Verify Agent performs comprehensive verification of generated artifacts against specifications, acceptance criteria, and code quality standards. This is the final gate before output is delivered.

## Trigger

This agent activates when:
- Mode evaluation includes `VERIFY`
- Review Agent has validated artifacts

## Input

| Field | Type | Description |
|-------|------|-------------|
| `artifacts` | Vec<Artifact> | Generated code artifacts |
| `spec` | TechnicalSpec | Original specification |
| `criteria` | Vec<AcceptanceCriterion> | Success criteria |

## Output

```json
{
  "status": "PASS|FAIL|WARN",
  "checks": [
    {
      "name": "string",
      "status": "pass|fail|warn",
      "message": "string"
    }
  ],
  "spec_coverage": 0.0-1.0,
  "quality_score": 0.0-1.0,
  "blocking_issues": [],
  "recommendations": []
}
```

## Verification Checks

| Check | Description |
|-------|-------------|
| Spec Mismatch | Artifacts must match specification |
| Missing Features | All spec requirements implemented |
| Hallucination Detection | No invented features |
| Edge Case Coverage | Edge cases from spec addressed |
| Test Coverage | Tests exist for critical paths |
| Pattern Compliance | Follows repository conventions |

## Decision Logic

```
IF blocking_issues.length > 0:
    status = FAIL
    → Reject artifacts, return to Execute Agent
ELSE IF recommendations.length > 0:
    status = WARN
    → Accept with warnings
ELSE:
    status = PASS
    → Accept artifacts
```

## Handoff

On PASS, artifacts are finalized. On FAIL, handoff back to Execute:

```rust
if result.status == VerifyStatus::Fail {
    ModeHandoff::new(Mode::Verify, Mode::Execute, issues_json)
}
```
