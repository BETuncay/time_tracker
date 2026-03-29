#!/usr/bin/env bash
# work.sh — invoke Claude Code to read AGENT.md and work one session

set -euo pipefail

PROMPT="Read AGENT.md in the current directory and follow the Agent Work Loop defined there:
1. Read TODO.md and pick the next open [ ] task
2. Mark it [~] in progress in TODO.md
3. Implement it
4. Mark it [x] done in TODO.md
5. Commit all changes with a clear commit message
Stop after completing one task unless the task was trivial (e.g. only a config or file change), in which case continue to the next one."

claude "$PROMPT" -p --dangerously-skip-permissions
