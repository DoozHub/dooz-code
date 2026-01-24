---
description: Full pipeline workflow from prompt to production code
---

# Full Pipeline Workflow

Execute the complete dooz-code pipeline: `EVALUATE → INTAKE → PLAN → ANALYZE → EXECUTE → REVIEW → VERIFY`

## Steps

1. **Evaluate Prompt**
   ```bash
   # Determine required modes
   dooz-code evaluate --prompt "<user_prompt>"
   ```

2. **Run Intake Agent** (if INTAKE mode required)
   ```bash
   dooz-code intake --prompt "<user_prompt>"
   # Output: technical-spec.json
   ```

3. **Run Plan Agent**
   ```bash
   dooz-code plan --spec ./technical-spec.json --repo .
   # Output: task-dag.json
   ```

4. **For each task in DAG:**
   ```bash
   # Analyze context
   dooz-code analyze --task <task-id> --repo .
   
   # Execute task
   dooz-code execute --task <task-id> --context ./context.json
   
   # Review artifacts
   dooz-code review --artifacts ./artifacts/ --criteria ./criteria.json
   ```

5. **Final Verification**
   ```bash
   dooz-code verify --artifacts ./artifacts/ --spec ./technical-spec.json
   ```

## Error Handling

- If REVIEW fails → retry EXECUTE (max 3 iterations)
- If VERIFY fails → return to user with blocking issues
