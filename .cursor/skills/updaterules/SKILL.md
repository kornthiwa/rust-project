---
name: updaterules
description: Rewrites and optimizes Cursor rules in .cursor/rules with clear scope, minimal overlap, and enforceable guidance. Use when the user asks to create, update, refactor, or improve project rules, .mdc files, or AI guidance policy.
---

# Update Rules

## Goal

Produce high-signal, low-overlap rules that are easy for agents to follow and cheap in token cost.

## When to Use

- User asks to create/update/rewrite rules.
- User mentions `.cursor/rules`, `.mdc`, policy, conventions, or AI behavior guidance.
- Existing rules are duplicated, conflicting, vague, or too verbose.

## Workflow

1. Inventory current rules in `.cursor/rules/*.mdc`.
2. Classify each rule:
   - `alwaysApply: true` for universal constraints.
   - `alwaysApply: false` + `globs` for file/domain-specific guidance.
3. Detect and remove overlap:
   - Keep one owner per concern (security, workflow, architecture, errors, testing).
   - Merge duplicate guidance into the most relevant rule.
4. Rewrite for enforceability:
   - Prefer short imperative bullets.
   - Replace vague wording ("should try") with clear expectations.
   - Keep each rule focused on one concern.
5. Verify structure:
   - Valid frontmatter (`description`, plus `alwaysApply` and optional `globs`).
   - Consistent terminology and no contradictory statements.
   - No secret-handling violations (do not read/write `.env` unless explicitly requested).
6. Report outcome:
   - List changed files and intent.
   - Call out any assumptions and optional next tightening pass.

## Authoring Standard

- Keep `SKILL.md` concise and practical.
- Prefer 5-10 strong bullets over long prose.
- Use stable wording for repeatability.
- Do not edit unrelated project code when task is rule maintenance only.

## Output Template

Use this structure in responses after rule updates:

```markdown
Updated rules for clarity and execution reliability.

Changed files:
- .cursor/rules/<file-1>.mdc: <intent>
- .cursor/rules/<file-2>.mdc: <intent>

Quality improvements:
- Reduced overlap/conflicts across rules
- Improved enforceability and scope boundaries
- Preserved security baseline constraints

Optional next step:
- Tighten file-specific globs per service/module
```
