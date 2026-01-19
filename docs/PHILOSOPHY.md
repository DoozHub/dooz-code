# Philosophy

> *Execution is not decision. They must remain separate.*

---

## The Problem With AI Coding Tools

Every AI coding tool makes the same fundamental error: **they collapse decision and execution.**

When you ask Copilot to "add authentication," it immediately starts generating code. It decides what authentication means, how it should work, what patterns to use—and generates artifacts simultaneously.

This creates three problems:

### 1. Hidden Decisions

The AI makes architectural decisions implicitly. You don't see them. You don't approve them. They're embedded in the code you receive.

### 2. Scope Drift

Without explicit boundaries, the AI adds features you didn't ask for. "Authentication" becomes "authentication with password reset, OAuth, and email verification" without you ever approving that scope.

### 3. Context Collapse

The AI treats each request as independent. It doesn't understand that this authentication module must integrate with your existing user model, your session management, your audit logging.

---

## The Dooz-Code Principle

**Dooz-Code executes. It does not decide.**

The decision about *what* to build happens elsewhere—in requirements analysis, in governance evaluation, in strategic planning. By the time Dooz-Code receives a work package, the scope is fixed.

This separation enables:

### Explicit Architecture

Architectural decisions are made consciously by roles with that authority (CTO, BA). They're documented, reviewed, and approved before execution begins.

### Scope Integrity

The work package defines boundaries. Dooz-Code cannot add features, cannot expand scope, cannot reinterpret requirements. It implements exactly what was approved.

### System Awareness

Because Dooz-Code operates within an agency, it has access to full system context—not just the current file, but the entire repository, its history, its patterns, its constraints.

---

## Why This Matters

### For Small Teams

You don't have budget for architectural mistakes. Every wrong abstraction costs you weeks. Separated execution means you approve the architecture before code is written.

### For Growing Teams

Knowledge concentrates in fewer heads as you scale. Separated execution means architectural decisions are explicit and documented, not hidden in AI-generated code.

### For Enterprise

Compliance requires audit trails. Separated execution means every decision is traceable—who approved what, when, and why.

---

## The Execution Contract

When Dooz-Code receives a work package, it agrees to:

1. **Analyze** the repository context fully
2. **Respect** existing patterns and conventions
3. **Implement** exactly the specified scope
4. **Test** against the defined acceptance criteria
5. **Document** the rationale for implementation choices
6. **Signal** completion with full audit trail

It will **never**:

1. Add features not in scope
2. Change architecture without approval
3. Override governance decisions
4. Proceed without sufficient context
5. Optimize for speed over correctness

---

## The Moat

Every other AI coding tool is racing to be faster, more helpful, more autonomous.

Dooz-Code is racing to be **more constrained**.

The constraint is the value. The separation is the moat.

A system that can't override governance is a system you can trust. A coder that can't expand scope is a coder that won't create technical debt.

---

## Who Should Use This

- Teams that have been burned by "helpful" AI that created architectural debt
- Organizations where compliance matters
- Founders who want to move fast without breaking things
- Engineers who are tired of cleaning up AI-generated code

---

## Who Should Not Use This

- Developers who want fast autocomplete
- Teams that don't do code review
- Anyone who thinks "more code = more progress"
- People who want an AI that says yes to everything

---

*Dooz-Code is not helpful on demand. It is correct when activated.*
