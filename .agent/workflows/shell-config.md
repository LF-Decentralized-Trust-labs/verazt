---
description: Shell configuration for agent commands
---

# Shell Configuration

Commands run by the AI agent should use a separate history session to avoid cluttering the user's main shell history while still being referenceable.

// turbo-all

## Fish Shell

Set the `fish_history` variable to use a separate history file:
```fish
set -gx fish_history ai-agent
```

History is stored in `~/.local/share/fish/ai-agent_history`.

To view this history later:
```fish
set fish_history ai-agent
history
```

---

## Bash

Set `HISTFILE` to use a separate history file:
```bash
export HISTFILE=~/.bash_history_ai_agent
```

To view this history later:
```bash
cat ~/.bash_history_ai_agent
# or
HISTFILE=~/.bash_history_ai_agent history -r && history
```

---

## Zsh

Set `HISTFILE` to use a separate history file:
```zsh
export HISTFILE=~/.zsh_history_ai_agent
```

To view this history later:
```zsh
cat ~/.zsh_history_ai_agent
# or
fc -R ~/.zsh_history_ai_agent && fc -l
```
