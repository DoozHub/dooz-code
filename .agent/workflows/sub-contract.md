---
description: Dispatch sub-work to specialized external agents
---

# Sub-Contract Workflow

Delegate complex or specialized tasks to external agents.

## Steps

1. **Identify Complex Task**
   - Complexity score > 7
   - Requires specialized domain (security, ML, etc.)
   - Explicit contract request

2. **Query Agent Registry**
   ```bash
   dooz-code agents list --capability security-audit
   ```

3. **Create Contract**
   ```bash
   dooz-code contract create \
     --task ./sub-task.json \
     --agent security-expert-001 \
     --budget 0.10
   ```

4. **Monitor Contract**
   ```bash
   dooz-code contract status <contract_id>
   ```

5. **Aggregate Results**
   ```bash
   dooz-code contract merge <contract_id> --into ./artifacts/
   ```

## Agent Registration

External agents register via API:
```bash
curl -X POST http://localhost:8080/api/agents/register \
  -d '{
    "name": "Security Auditor",
    "capabilities": ["security-audit"],
    "endpoint": "https://agent.example.com/execute"
  }'
```
