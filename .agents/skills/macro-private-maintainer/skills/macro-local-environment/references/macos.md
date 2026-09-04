# macOS Development Environment

This guide reflects the current repository behavior in `docs/RUNNING_LOCALLY.md`, `tooling/just/xtask.just`, and `tooling/xtask/crates/xtask_local`.

## Host Model

Primary local development on macOS uses:

- macOS on Apple Silicon / arm64 when running on Kevin's main machine.
- Nix dev shell for CLI tools and the Rust/frontend toolchain.
- Docker Desktop, OrbStack, or Colima for the Docker daemon and Linux container runtime.
- Docker Compose and BuildKit/buildx through the active Docker runtime.

On macOS, the Nix shell supplies Docker CLI tooling, but it does not by itself provide a daemon. Do not try to start a second Docker daemon from Nix.

## Build and Runtime Model

`just run_local` is a hybrid local workflow:

- Rust services are built on the host with `cargo zigbuild`.
- The target is Linux ARM64, currently `aarch64-unknown-linux-gnu`.
- Built service binaries are mounted into a shared runtime image.
- Docker does not compile the normal Rust services during a standard `run_local`.
- Docker runs infrastructure and service containers.
- The frontend runs as a Vite dev server unless disabled.

Do not call the Rust service binaries "native macOS production artifacts"; they are Linux ARM64 binaries produced from the macOS host.

## Local Infrastructure

The fully local stack runs these dependencies in Docker:

- PostgreSQL
- Redis
- LocalStack
- OpenSearch
- Kafka
- FusionAuth
- Mailpit

`bring_up_infra` runs Docker Compose with `up -d --wait` for `postgres`, `redis`, `search`, `kafka`, and `localstack`. FusionAuth is started and polled separately because its kickstart can outlive its early healthcheck retries.

## First-Run Behavior

First startup can be slow because Docker may pull and build large images. Known heavy pieces include:

- `opensearchproject/opensearch` plus the local `macro-local-opensearch` image with `analysis-icu`.
- FusionAuth.
- Kafka.
- Rust/Node base images for repository-built auxiliary services.
- `sync_service`, `lexical_service`, and `websocket_service` when their images are built.

Do not diagnose a long first image pull as a Rust build failure until Docker/buildx activity, image presence, and compose state have been checked.

## No-Doppler Authentication

With `--no-doppler`:

- Local FusionAuth authentication is expected to work.
- Passwordless login codes land in Mailpit, not a real inbox.
- Google login and real Gmail linking are not expected to work unless real integration keys are provided with `--env-file`.
- Stubbed third-party values are enough for the local stack to start.

Basic DEV acceptance:

1. Infrastructure containers are running and expected healthchecks pass.
2. Application services start.
3. Frontend is reachable.
4. Mailpit is reachable.
5. Local passwordless login succeeds.
6. `/app` opens.
7. Email, Contacts, and Search pages open.

