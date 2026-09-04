# Local Troubleshooting

## Docker Image Pull Appears Stuck

First distinguish:

- Rust compilation.
- Docker image pull.
- Docker image build.
- Compose startup.
- Health waiting.

Useful checks:

```bash
ps -axo pid,ppid,stat,etime,pcpu,pmem,command | rg -i 'run-local|xtask_local|docker compose|docker-buildx|docker pull'
docker images
docker ps -a
docker buildx ls
docker system events --since 30m --until 0s
```

If `docker pull` or BuildKit sits for many minutes with no image/container events and negligible transfer in Docker proxy logs, treat it as a Docker runtime, registry, mirror, or proxy issue before changing project code.

Large first-run pulls/builds can include OpenSearch, FusionAuth, Kafka, and Rust/Node base layers. Do not restart repeatedly while BuildKit is actively downloading or extracting layers.

## OpenSearch First Pull

The local `search` service builds `macro-local-opensearch` from `infra/local/opensearch/Dockerfile`, based on `opensearchproject/opensearch`, then installs `analysis-icu`.

`Starting infra (docker compose up -d --wait)` can therefore spend a long time in Docker pull/build even though Rust services have already built.

Check:

```bash
docker images --digests | rg -i 'opensearch|macro-local-opensearch'
docker compose --project-directory <repo> -p <project> -f docker/docker-compose.yml -f infra/local/generated/<instance>/docker-compose.override.yml --env-file infra/local/generated/<instance>/local.generated.env ps
```

## macOS Port Conflicts

Run:

```bash
just doctor-local
```

Do not kill unrelated macOS/system services by default. Prefer an isolated instance and a free port window:

```bash
just doctor-local --instance <name> --port-base <free-port>
just run_local --no-doppler --instance <name> --port-base <free-port>
```

Use the same flags later with `status_local`, `seed-scenario`, `stop_local`, `reset_local`, and `destroy_local`.

## Stale Auxiliary Service Images

Normal `run_local` builds Rust service binaries on the host and mounts them into the runtime image. It does not rebuild every repository-built Docker image by default.

The known repository-built auxiliary services are:

- `sync_service`
- `lexical_service`
- `websocket_service`

If changing those services, start with:

```bash
just run_local --build-aux-services
```

or use `just stack update` in headless mode.

## Local Proxy and Network

Docker Desktop networking and the macOS host proxy are separate layers. Before modifying project code for image pull or connection failures, determine whether the failure is:

- Docker Hub or registry mirror access.
- Docker Desktop proxy configuration.
- Local HTTP/SOCKS proxy behavior.
- Image architecture or manifest availability.
- A real application dependency failure.

