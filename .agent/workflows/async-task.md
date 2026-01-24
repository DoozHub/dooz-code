---
description: Submit async task to dooz-worker queue (Jules-style)
---

# Async Task Workflow

Submit long-running tasks to dooz-worker for background execution.

## Steps

1. **Submit Task**
   ```bash
   curl -X POST http://localhost:8080/api/tasks \
     -H "Content-Type: application/json" \
     -d '{"prompt": "<user_prompt>", "repo_url": "<git_url>"}'
   ```

2. **Monitor Progress**
   ```bash
   # Poll status
   curl http://localhost:8080/api/tasks/<task_id>/status
   
   # Or subscribe to WebSocket
   wscat -c ws://localhost:8080/api/tasks/<task_id>/stream
   ```

3. **Retrieve Results**
   ```bash
   curl http://localhost:8080/api/tasks/<task_id>/artifacts
   ```

## Webhook Callback

Configure callback URL for completion notification:
```json
{
  "prompt": "...",
  "callback_url": "https://your-server.com/webhook"
}
```
