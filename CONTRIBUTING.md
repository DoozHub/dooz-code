# Contributing to Dooz-Code

Thank you for your interest in contributing to Dooz-Code.

Before contributing, please read and understand the [Governance](GOVERNANCE.md) document. Dooz-Code has strict boundaries that cannot be compromised.

---

## Development Setup

### Prerequisites

- Rust 1.75 or later
- Git
- A Unix-like environment (Linux, macOS, WSL)

### Installation

```bash
# Clone the repository
git clone https://github.com/DoozHub/dooz-code.git
cd dooz-code

# Build the project
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- analyze --repo ./examples/sample-project
```

---

## Project Structure

```
dooz-code/
├── src/
│   ├── lib.rs           # Library root
│   ├── main.rs          # CLI entry point
│   ├── types/           # Core data structures
│   │   ├── mod.rs
│   │   ├── work_package.rs
│   │   ├── context.rs
│   │   └── result.rs
│   ├── analyzer/        # Repository analysis
│   │   ├── mod.rs
│   │   ├── files.rs
│   │   ├── patterns.rs
│   │   └── dependencies.rs
│   ├── planner/         # Implementation planning
│   │   ├── mod.rs
│   │   ├── decompose.rs
│   │   ├── order.rs
│   │   └── rollback.rs
│   ├── executor/        # Code generation
│   │   ├── mod.rs
│   │   ├── step.rs
│   │   ├── generate.rs
│   │   └── apply.rs
│   └── reviewer/        # Self-validation
│       ├── mod.rs
│       ├── validate.rs
│       └── iterate.rs
├── tests/               # Integration tests
├── examples/            # Usage examples
└── docs/                # Documentation
```

---

## Code Standards

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

---

## Commit Guidelines

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | Purpose |
|------|---------|
| `feat` | New feature within boundaries |
| `fix` | Bug fix |
| `docs` | Documentation |
| `test` | Test additions |
| `refactor` | Code refactoring |
| `perf` | Performance improvement |
| `chore` | Maintenance |

### Examples

```
feat(analyzer): add dependency graph extraction

fix(executor): correct file path handling on Windows

docs(readme): clarify installation steps

test(planner): add rollback plan generation tests
```

---

## Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/description`)
3. **Make** your changes
4. **Test** thoroughly (`cargo test`)
5. **Lint** your code (`cargo clippy`)
6. **Commit** with clear messages
7. **Push** to your fork
8. **Open** a pull request

### PR Template

```markdown
## Description

Brief description of changes.

## Type of Change

- [ ] Bug fix
- [ ] New feature (within boundaries)
- [ ] Documentation
- [ ] Performance improvement
- [ ] Test improvement

## Governance Compliance

- [ ] Does not add scope expansion capability
- [ ] Does not bypass governance requirements
- [ ] Maintains determinism guarantees
- [ ] Respects existing patterns

## Testing

- [ ] All existing tests pass
- [ ] New tests added for new functionality
- [ ] Manual testing performed

## Documentation

- [ ] Code comments added where needed
- [ ] README updated if relevant
- [ ] API documentation updated if relevant
```

---

## Areas for Contribution

### Wanted

| Area | Description |
|------|-------------|
| Analyzer | Better pattern detection algorithms |
| Planner | Improved step ordering |
| Tests | Higher coverage, edge cases |
| Docs | Examples, tutorials, clarifications |
| Performance | Faster analysis, lower memory |

### Not Wanted

| Area | Reason |
|------|--------|
| Chat interface | Violates execution-only principle |
| Auto-suggestions | Violates scope boundaries |
| "Smart" features | Violates determinism |
| Productivity tools | Outside mission |

---

## Questions?

- Read the [Philosophy](docs/PHILOSOPHY.md)
- Check the [Architecture](docs/ARCHITECTURE.md)
- Review the [Governance](GOVERNANCE.md)

If still unclear, open an issue with the `question` label.

---

*Contributions that strengthen execution integrity are welcome.*
