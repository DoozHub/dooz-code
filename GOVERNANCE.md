# Governance

> Any change that weakens execution boundaries or adds autonomous feature creation will be rejected.

---

## Constitutional Laws

The following are immutable. They cannot be modified, weakened, or bypassed:

1. **Scope is immutable during execution** — Cannot expand, reduce, or reinterpret scope
2. **Execution is deterministic** — Same input → same output, no randomness
3. **Governance cannot be bypassed** — Approval must precede execution
4. **Patterns are respected** — Existing conventions take precedence
5. **Context is required** — Cannot execute without full repository analysis
6. **Audit is complete** — Every decision is logged

---

## Contribution Policy

### Allowed Contributions

- Analyzer improvements
- Planner optimizations
- Pattern detection enhancements
- Self-review logic improvements
- Documentation clarifications
- Bug fixes that don't weaken boundaries
- Test coverage expansion
- Performance improvements

### Forbidden Contributions

- Autonomous feature creation
- Scope expansion logic
- Governance bypass mechanisms
- "Helpful" behavior that ignores constraints
- Chat interfaces
- Productivity shortcuts
- Output optimization at cost of correctness
- Non-deterministic execution paths

---

## Pull Request Requirements

Every PR must:

1. Not weaken execution boundaries
2. Not add scope expansion capability
3. Maintain determinism guarantees
4. Include relevant tests
5. Document any new behavior
6. Pass all existing tests

---

## Review Criteria

Maintainers will reject PRs that:

- Add features that weren't in scope
- Reduce pattern adherence
- Introduce non-determinism
- Bypass required approval
- Improve "user experience" at cost of execution integrity
- Add autonomy beyond approved scope

---

## Versioning

- **Patch versions** — Bug fixes, documentation, minor improvements
- **Minor versions** — New analysis capabilities, improved planning
- **Major versions** — Fundamental changes (rare, requires RFC)

Constitutional laws do not change across versions.

---

## Maintainer Philosophy

The maintainers believe:

- Less autonomy is better
- Constraints enable trust
- User convenience is secondary to execution integrity
- Following patterns matters more than "improving" them
- The work package defines reality

PRs are evaluated against this philosophy.

---

## Dispute Resolution

If a contribution is rejected:

1. The rejection reason will cite specific governance violations
2. Appeal requires demonstrating constitutional compliance
3. Maintainer decision is final on philosophy matters

---

*This project exists to execute, not to create.*
