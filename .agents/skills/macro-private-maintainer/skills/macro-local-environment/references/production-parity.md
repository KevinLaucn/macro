# Production and CI Parity

Local DEV success is useful, but it is not production or CI parity.

## DEV PASS Means

For `just run_local --no-doppler`, a pass means:

- Host-side `cargo zigbuild` produced Linux ARM64 service binaries.
- Local Docker infrastructure and service containers started.
- Local migrations/provisioning completed.
- The app can be exercised through the local frontend/proxy.

It does not prove Nix/crane production builds, release Dockerfiles, or CI workflows pass.

## PROD_LOCAL Must Use Production-Like Artifacts

When the task asks for production-local validation, self-host validation, or release parity, first inspect the current Nix, Dockerfile, and workflow definitions. Do not infer production behavior from `run_local`.

Validate against the exact artifact path the task cares about, such as:

- Nix flake outputs.
- crane-built Rust artifacts.
- official production Dockerfiles.
- self-host Email profile artifacts.
- compose files used for production deployment.

Do not use real production secrets for exploratory local DEV. Use explicit local or staging secrets only when the task requires integration validation and the user has supplied/authorized them.

## CI_PARITY

For CI parity, prefer the repository's actual CI commands and build graph. A local macOS `run_local` pass is not enough.

Use the smallest relevant CI-equivalent command set, then report exactly what was and was not covered.

