# Dooz-Code Configuration Guide

> Complete configuration reference for dooz-code v0.2.0

---

## Configuration File Locations

Dooz-Code searches for configuration in the following order:

1. `./dooz-code.yaml` or `./dooz-code.yml` (current directory)
2. `./dooz-code.json` (current directory)
3. `~/.config/dooz-code.yaml` (home directory)

You can also specify a config file explicitly:

```bash
dooz-code execute --config /path/to/config.yaml --package work.yaml --repo .
```

---

## Configuration Structure

### Top-Level Configuration

```yaml
# dooz-code.yaml
llm: {}           # LLM provider settings
executor: {}      # Execution settings
analyzer: {}      # Repository analysis settings
reviewer: {}      # Code review settings
general: {}       # General settings
```

---

## LLM Configuration

### `llm.provider`

**Type:** String  
**Default:** `"stub"`  
**Options:** `"stub"`, `"computer-use"`, `"openai"`, `"anthropic"`

```yaml
llm:
  provider: "computer-use"
```

### `llm.api_url`

**Type:** String (URL)  
**Default:** `http://127.0.0.1:8315`

```yaml
llm:
  api_url: "http://localhost:8080/v1"
```

### `llm.api_key`

**Type:** String  
**Default:** (empty)

> **Security Note:** It's recommended to use environment variables instead:
> ```bash
> export DOOZ_LLM_API_KEY="sk-..."
> ```

### `llm.model`

**Type:** String  
**Default:** `"gemini-2.5-computer-use-preview-10-2025"`

```yaml
llm:
  model: "qwen3-coder-plus"
```

### `llm.max_tokens`

**Type:** Integer  
**Default:** `4096`

```yaml
llm:
  max_tokens: 8192
```

### `llm.temperature`

**Type:** Float (0.0 - 1.0)  
**Default:** `0.2`

```yaml
llm:
  temperature: 0.1  # Lower = more deterministic
```

### `llm.retries`

**Type:** Integer  
**Default:** `3`

```yaml
llm:
  retries: 5
```

### `llm.timeout_seconds`

**Type:** Integer (seconds)  
**Default:** `60`

```yaml
llm:
  timeout_seconds: 120
```

### `llm.fallback_models`

**Type:** Array of Strings  
**Default:** `[]`

```yaml
llm:
  fallback_models:
    - "qwen3-coder-flash"
    - "deepseek-v3.1"
```

---

## Executor Configuration

### `executor.max_artifacts`

**Type:** Integer  
**Default:** `100`

Maximum number of artifacts (files) that can be generated in a single execution.

```yaml
executor:
  max_artifacts: 50
```

### `executor.max_lines_per_file`

**Type:** Integer  
**Default:** `1000`

Maximum number of lines per generated file.

```yaml
executor:
  max_lines_per_file: 500
```

### `executor.dry_run`

**Type:** Boolean  
**Default:** `false`

When `true`, no files are written to disk.

```yaml
executor:
  dry_run: true
```

### `executor.follow_patterns`

**Type:** Boolean  
**Default:** `true`

When `true`, the executor follows detected code patterns from the repository.

```yaml
executor:
  follow_patterns: true
```

### `executor.enable_correction`

**Type:** Boolean  
**Default:** `true`

When `true`, the executor attempts to correct code that fails validation.

```yaml
executor:
  enable_correction: false
```

### `executor.max_corrections`

**Type:** Integer  
**Default:** `3`

Maximum number of correction iterations.

```yaml
executor:
  max_corrections: 5
```

---

## Analyzer Configuration

### `analyzer.include_extensions`

**Type:** Array of Strings  
**Default:** `[]` (all files)

File extensions to include in analysis.

```yaml
analyzer:
  include_extensions:
    - ".rs"
    - ".ts"
    - ".py"
```

### `analyzer.exclude_patterns`

**Type:** Array of Strings  
**Default:** `[]`

File patterns to exclude (supports glob patterns).

```yaml
analyzer:
  exclude_patterns:
    - "**/generated/**"
    - "**/*.min.js"
```

### `analyzer.exclude_dirs`

**Type:** Array of Strings  
**Default:** `["target", "node_modules", ".git"]`

Directories to exclude from analysis.

```yaml
analyzer:
  exclude_dirs:
    - "target"
    - "node_modules"
    - ".git"
    - "build"
    - "dist"
```

### `analyzer.max_file_size`

**Type:** Integer (bytes)  
**Default:** `1048576` (1 MB)

Maximum file size to analyze.

```yaml
analyzer:
  max_file_size: 5242880  # 5 MB
```

### `analyzer.threads`

**Type:** Integer  
**Default:** (number of CPU cores)

Number of parallel threads for analysis.

```yaml
analyzer:
  threads: 4
```

---

## Reviewer Configuration

### `reviewer.security_checks`

**Type:** Boolean  
**Default:** `true`

Enable security vulnerability checks.

```yaml
reviewer:
  security_checks: false
```

### `reviewer.performance_checks`

**Type:** Boolean  
**Default:** `true`

Enable performance recommendations.

```yaml
reviewer:
  performance_checks: false
```

### `reviewer.style_checks`

**Type:** Boolean  
**Default:** `true`

Enable code style checks.

```yaml
reviewer:
  style_checks: false
```

### `reviewer.min_test_coverage`

**Type:** Float (percentage)  
**Default:** `null` (no requirement)

Minimum test coverage percentage required.

```yaml
reviewer:
  min_test_coverage: 80.0
```

### `reviewer.custom_rules`

**Type:** Object  
**Default:** `{}`

Custom linting rules.

```yaml
reviewer:
  custom_rules:
    max_function_length: "50"
    require_doc_comments: "true"
```

---

## General Configuration

### `general.work_dir`

**Type:** String (path)  
**Default:** `null`

Working directory for execution.

```yaml
general:
  work_dir: "/tmp/dooz-work"
```

### `general.output_format`

**Type:** String  
**Default:** `"summary"`

Output format for CLI.

```yaml
general:
  output_format: "json"  # "summary", "json", "minimal"
```

### `general.verbose`

**Type:** Boolean  
**Default:** `false`

Enable verbose output.

```yaml
general:
  verbose: true
```

### `general.log_level`

**Type:** String  
**Default:** `"info"`

Log level.

```yaml
general:
  log_level: "debug"  # "debug", "info", "warn", "error"
```

### `general.colors`

**Type:** Boolean  
**Default:** `true`

Enable colored output.

```yaml
general:
  colors: false
```

---

## Environment Variable Overrides

Environment variables take precedence over config file values:

| Environment Variable | Maps To |
|----------------------|---------|
| `DOOZ_LLM_API_URL` | `llm.api_url` |
| `DOOZ_LLM_API_KEY` | `llm.api_key` |
| `DOOZ_LLM_MODEL` | `llm.model` |
| `DOOZ_OUTPUT_FORMAT` | `general.output_format` |
| `DOOZ_LOG_LEVEL` | `general.log_level` |
| `DOOZ_VERBOSE` | `general.verbose` |

---

## Complete Example

```yaml
# dooz-code.yaml
# Complete configuration example

llm:
  provider: "computer-use"
  api_url: "http://127.0.0.1:8315"
  api_key: ""  # Set via DOOZ_LLM_API_KEY
  model: "gemini-2.5-computer-use-preview-10-2025"
  max_tokens: 4096
  temperature: 0.2
  retries: 3
  timeout_seconds: 60
  fallback_models:
    - "qwen3-coder-flash"
    - "deepseek-v3.1"

executor:
  max_artifacts: 100
  max_lines_per_file: 1000
  dry_run: false
  follow_patterns: true
  enable_correction: true
  max_corrections: 3

analyzer:
  include_extensions: []
  exclude_patterns: []
  exclude_dirs:
    - target
    - node_modules
    - .git
    - build
    - dist
  max_file_size: 1048576
  threads: 4

reviewer:
  security_checks: true
  performance_checks: true
  style_checks: true
  min_test_coverage: null
  custom_rules: {}

general:
  work_dir: null
  output_format: "summary"
  verbose: false
  log_level: "info"
  colors: true
```

---

## Generating a Config Template

Generate a template config file:

```bash
dooz-code generate-config > dooz-code.yaml
```

---

## Validation

Validate your configuration file:

```bash
dooz-code validate-config --config dooz-code.yaml
```

Or use the default locations:

```bash
dooz-code validate-config
```
