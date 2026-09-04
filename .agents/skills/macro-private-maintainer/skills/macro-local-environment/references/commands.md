# Local Command Guide

Run these commands from the repository root unless a command explicitly says otherwise.

## Preflight

```bash
just doctor-local
just doctor-local --instance <name> --port-base <free-port>
```

`doctor-local` checks Docker daemon access, required tools/toolchain, ports, env sources, and selected images.

## Attached Local Stack

```bash
just run_local --no-doppler
just run_local --no-doppler --instance <name> --port-base <free-port>
```

`run_local` starts local infra, app services, proxy, and a Vite frontend server. It stays attached and supports hotkeys:

- `r`: rebuild changed Rust service binaries and reload services.
- `q`: stop and remove the stack cleanly.

Use `--build-aux-services` only when working on repository-built Docker services that are not rebuilt by default.

## Shared Dev Resources

```bash
just run_dev
```

`run_dev` runs local binaries against shared dev resources. It needs Doppler and real cloud access. It is not the normal self-contained local workflow.

## Headless Stack

```bash
just stack up --no-doppler
just stack status --json
just stack update
just stack update --frontend
just stack down
```

`stack up` is for agents and CI-style local operation. It returns after startup, serves a static frontend bundle through the proxy, and leaves Docker containers running.

## Status and Lifecycle

```bash
just status_local
just status_local --instance <name> --port-base <free-port>
just stop_local --instance <name>
just reset_local --instance <name>
just destroy_local --instance <name>
```

`status_local` does not start or rebuild anything. It reports live endpoints, reachability, container state, health, and host ports.

## Seed Data

Use the same instance and port-base as the running stack:

```bash
just seed-scenario --instance <name> --port-base <free-port> apply --file seed/scenarios/team-perms.json
just seed-scenario --instance <name> --port-base <free-port> status --file seed/scenarios/team-perms.json
```

If a stack started with an explicit `--port-base`, all seed/status commands for that stack must repeat the same `--port-base`.

