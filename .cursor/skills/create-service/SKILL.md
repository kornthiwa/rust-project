---
name: create-service
description: Creates a new Rust *-service with mandatory axum, sqlx (PostgreSQL), migrations, and dotenv baseline setup, plus onion architecture folders, wiring, error mapping, and verification steps. Use when the user asks to create a new service, bootstrap a microservice, or set up service scaffolding end-to-end.
---

# Create Service

## Goal

Create a new `*-service` that can run immediately and follows this repository's architecture, security, and testing conventions.

## When to Use

- User asks to create a new service.
- User asks for service scaffolding, bootstrap, or initial setup.
- User wants "พร้อมใช้งานเลย" (ready-to-run with baseline wiring).

## Required Inputs

Collect before editing:

- Service name (must end with `-service`)
- Main capability (one sentence)
- HTTP base path (example: `/users`, `/billing`)

If any required input is missing, ask a short clarifying question first.

## Workflow

1. **Create project**
   - Create crate at repo root: `<name>-service/`.
   - Use Rust 2024 edition.
   - Add mandatory dependencies: `axum`, `sqlx` (features: `runtime-tokio`, `postgres`, `migrate`, `macros` — align with existing services; add TLS features only if required), `tokio`, `dotenv`, `serde`, `tracing`, `tracing-subscriber`, `jsonwebtoken` when auth is needed.

2. **Create baseline structure**
   - `src/domain/` for pure business models and rules.
   - `src/application/` for use-cases, ports, service orchestration.
   - `src/infrastructure/` for db/clients/adapters.
   - `src/presentation/` for transport (HTTP handlers, bot commands, etc.).
   - `src/main.rs` and extra bootstrap file only if needed (e.g. `src/app.rs`).

3. **Wire runtime**
   - Load config via `dotenv` + environment-backed config module (no inline secrets).
   - Include JWT config from env (`JWT_SECRET`; optional `JWT_EXPIRATION_SECONDS` if issuing tokens).
   - Build app state and dependency graph in one bootstrap location.
   - Add graceful startup log with service name and bind address.
   - Build Axum router with fallback route and panic-safe middleware.
   - Add `presentation/http/middleware/jwt.rs` and `mod.rs` with bearer-token validation middleware.
   - Protect private routes by applying JWT middleware with `axum::middleware::from_fn_with_state`.
   - Return `401` + `WWW-Authenticate: Bearer` on missing/invalid token.
   - Connect **`sqlx::PgPool`** to PostgreSQL via `PgPoolOptions::connect` using env-backed `DATABASE_URL`.
   - Add **`migrations/`** at the crate root with an initial SQL migration (`CREATE TABLE ...`); run `sqlx::migrate!("./migrations").run(&pool).await?` once after connect (same pattern as `auth-service`, `users-service`, `messages-service`).
   - Implement repositories with `sqlx::query` / `query_as` + `FromRow` row types in `infrastructure` only; map rows to domain entities at the repository boundary.
   - Keep DB URL in env-backed config (for example `DATABASE_URL`), not hardcoded literals.

4. **Add error strategy**
   - Add typed application error with stable `code`.
   - Map infra errors to app errors with safe context.
   - Map app errors to transport-safe HTTP responses (no internal leaks).

5. **Add minimum tests**
   - At least one success test and one failure test for main use-case.
   - Add one regression test if service is created to fix a known bug.

6. **Verify**
   - Run formatter and compile check for the new service.
   - Run tests for the new service.
   - Report commands run, outcomes, and any manual follow-up needed.

## Guardrails

- Do not read or modify `.env` unless user explicitly requests it.
- Do not leak secrets/tokens in code samples or logs.
- Keep `sqlx` / `PgPool` / row types out of `domain`.
- Avoid `unwrap()`/`expect()` in production paths.
- Do not edit unrelated services unless user asks.
- Do not scaffold alternate persistence stacks unless user explicitly overrides this baseline.

## Output Format

Use this response structure after implementation:

```markdown
Created new service scaffold and baseline wiring.

Changed files:

- <service>/Cargo.toml: dependencies and package metadata
- <service>/src/main.rs: runtime entrypoint
- <service>/src/...: domain/application/infrastructure/presentation scaffolding
- <service>/migrations/: initial SQL migration

What is ready:

- Runnable startup path
- Axum + sqlx (PostgreSQL) + migrations + dotenv wiring
- Typed error flow and safe mapping
- JWT middleware for protected routes (if applicable)
- Baseline tests

Verification:

- `cargo fmt --all`
- `cargo check -p <service-name>`
- `cargo test -p <service-name>`
```
