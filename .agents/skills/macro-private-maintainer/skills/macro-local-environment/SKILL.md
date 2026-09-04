---
name: macro-local-environment
description: Local development, macOS Docker runtime, local stack, headless stack, local production validation, and CI parity workflow for KevinLaucn/macro.
---

# Macro Local Environment

Use this skill for any task involving local development, macOS Docker runtime, local stack startup, `run_local`, `run_dev`, `doctor-local`, `status_local`, `stack`, local service image builds, port conflicts, or local production/CI parity checks.

## First Step

Classify the request before running commands:

- `DEV`: attached local developer stack with Vite dev server.
- `DEV_HEADLESS`: detached local stack for agents/CI-style operation.
- `DEV_SHARED`: local binaries pointed at shared dev resources.
- `PROD_LOCAL`: local validation of production-style artifacts.
- `CI_PARITY`: local reproduction of CI or release build behavior.
- `DEBUG_EXISTING_LOCAL`: inspect or repair an already running local stack.

Before acting, state:

- `LOCAL MODE`
- `HOST`
- `TARGET`
- `BUILD PATH`
- `RUNTIME PATH`

Use `git rev-parse --show-toplevel` for the repository root. Do not hard-code a personal absolute checkout path.

## Canonical DEV Commands

From the repository root:

```bash
nix develop
just doctor-local
just run_local --no-doppler
```

For an isolated instance or a port-conflict workaround:

```bash
just doctor-local --instance <name> --port-base <free-port>
just run_local --no-doppler --instance <name> --port-base <free-port>
just status_local --instance <name> --port-base <free-port>
```

The explicit port base `31000` is a useful example only; it is not a project default.

## Canonical Headless Commands

Use `just stack` when the caller needs the stack to return control after startup, with no attached hotkey loop and no Vite dev server:

```bash
just stack up --no-doppler
just stack status --json
just stack update
just stack down
```

All relevant `run_local` flags also apply to `stack`, including `--instance`, `--port-base`, `--no-doppler`, `--no-build`, and `--binaries-dir`.

## Mandatory Rules

- Preserve the upstream local workflow unless the task explicitly asks to change it.
- Do not create a second local infrastructure stack when `xtask_local` already owns the workflow.
- Do not kill unrelated host processes for port conflicts by default; prefer `--instance` and `--port-base`.
- Use the same `--instance` and `--port-base` for run, seed, status, stop, reset, and destroy commands.
- Never use production secrets for normal DEV.
- Do not treat local DEV success as production or CI parity.
- Use CodeGraph for symbol/call relationship analysis when `.codegraph/` exists.
- Use Cargo/Nix/Compose graphs for build dependency analysis instead of guessing from filenames.

## References

Read only what applies:

- `references/macos.md`: macOS host/runtime model and local startup expectations.
- `references/commands.md`: command map and when to use each one.
- `references/troubleshooting.md`: stuck image pulls, port conflicts, stale images, and status checks.
- `references/production-parity.md`: DEV vs production/CI parity boundaries.

