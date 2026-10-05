# Agent-owned storage

Every agent keeps its own data in its own place, so adding or changing one agent can't break
Nomi or another agent.

## The rules

1. **An agent owns its tables.** A built-in agent that needs storage gets its own tables, named
   with its agent type as the prefix (`money_*`, `reminders`), in its own migration
   (`NNNN_<agent>_*.sql`). Only that agent's crate and its own page routes write them.
2. **Core tables stay core.** `sessions`, `messages`, `agent_sessions`, `agent_delegations`,
   `scheduled_jobs`, `turn_jobs` and the other engine tables are written by the engine and
   workers, never directly by an agent's tools. An agent that needs something from them goes
   through the engine (delegation, reminders scheduling, content blocks).
3. **Agents without tables get private records.** Every dynamic agent, and any built-in that
   sets `SubAgent::uses_records`, gets four tools: `save_record`, `list_records`,
   `update_record` and `delete_record`. They read and write `agent_records`, scoped by the
   engine to the calling agent and the user, so no agent can see or change another's records.
   No migration is needed.
4. **Tools: new or existing.** An agent either brings its own tools (its crate's
   `SubAgent::tools`) or is granted existing ones from the `ToolCatalog` (dynamic agents pick
   these in the admin form). Tools that only touch the agent's own tables can skip the
   approval card with `SubAgent::tool_needs_approval`.
5. **Renaming without breaking.** When a table moves into an agent's namespace, keep the old name
   working (Money kept `mock_transactions` as a view over `money_transactions`).

## Today

| Agent | Tables | Tools |
|---|---|---|
| Money | `money_transactions`, `money_budgets` | `list_transactions`, `summarize_budget`, `log_transaction`, `set_budget`, `list_budgets` |
| Reminders | `reminders` (fired by `reminders_worker`, no model call) | `add_reminder`, `show_reminders`, `edit_reminder`, `complete_reminder`, `remove_reminder` |
| Files | none: reads attachments and voice notes in the message, then answers or delegates to the agent that owns the data | `delegate` (no tools of its own) |
| Dynamic agents | `agent_records` (scoped per agent) | record tools + whatever the admin grants |

Agent tasks scheduled for later ("every Monday, summarize my spending") remain the core
scheduler's job (`scheduled_jobs`), separate from the Reminders agent's reminders.
