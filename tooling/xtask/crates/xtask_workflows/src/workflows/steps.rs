//! Reusable workflow building blocks: a small fluent-builder trait plus typed
//! helpers that return `Step`s and `Job`s, composed by the workflow files.
//!
//! Third-party actions are pinned to a SHA with the human-readable version in a
//! trailing comment, matching the rest of the repo's workflows.

use gh_workflow::{Expression, Job, Run, Step, Use};
use xtask_paths::RepoDir;

use crate::workflows::vars;

#[cfg(test)]
mod test;

/// Namespace's sccache setup mints a short-lived workspace credential. GitHub
/// withholds repository secrets from fork PRs, but this runner-minted token is
/// not a `secrets.*` value, so enforce the equivalent trust boundary here.
const TRUSTED_NAMESPACE_SCCACHE_CONTEXT: &str = concat!(
    "(github.event_name != 'pull_request' && ",
    "github.event_name != 'pull_request_target') || ",
    "github.event.pull_request.head.repo.full_name == github.repository"
);

/// `.map` / `.when` combinators for fluent conditional composition
/// ("push ifs up"): centralize branching in the builder chain instead of
/// building values imperatively.
pub trait FluentBuilder: Sized {
    /// Apply `f` to `self`.
    fn map<U>(self, f: impl FnOnce(Self) -> U) -> U {
        f(self)
    }
    /// Apply `f` only when `cond` holds.
    fn when(self, cond: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if cond { f(self) } else { self }
    }
}

impl FluentBuilder for gh_workflow::Workflow {}
impl FluentBuilder for Job {}
impl<T> FluentBuilder for Step<T> {}

/// Reference a repo-local composite action (`uses: ./path`). The base
/// `gh-workflow` `uses()` only builds `owner/repo@version`, so we set the raw
/// `uses` field directly. Kept in one place so the workaround is contained.
pub(crate) fn uses_local(name: &str, path: RepoDir<'_>) -> Step<Use> {
    let mut step = Step::new(name).uses("local", "local", "0");
    step.value.uses = Some(format!("./{}", path.as_str()));
    step
}

/// `actions/checkout`, pinned. `full_history` fetches the full history, which
/// the path-filter diff in `path-check` needs. `persist_credentials` controls
/// whether checkout leaves the token in git config for later steps.
pub fn checkout(full_history: bool, persist_credentials: bool) -> Step<Use> {
    Step::new("Checkout")
        .uses(
            "actions",
            "checkout",
            "de0fac2e4500dabe0009e67214ff5f5447ce83dd",
        ) // v4
        .add_with(("clean", false))
        .when(full_history, |step| step.add_with(("fetch-depth", 0)))
        .when(!persist_credentials, |step| {
            step.add_with(("persist-credentials", false))
        })
}

/// `oven-sh/setup-bun` — installs Bun on the runner. Version-tagged (not
/// SHA-pinned) to match the JS deploy/publish workflows.
pub fn setup_bun() -> Step<Use> {
    Step::new("Setup Bun").uses("oven-sh", "setup-bun", "v2")
}

/// Install the Rust toolchain only (no sccache, no cache) — for the lightweight
/// `path-check` and workflow-drift jobs.
pub fn setup_rust_light() -> Step<Use> {
    uses_local(
        "Setup Rust",
        xtask_paths::repo_dir!(".github/actions/setup-rust"),
    )
    .add_with(("sccache", "false"))
    .add_with(("rust-cache", "false"))
}

/// [`setup_rust_light`] plus sccache, for jobs that actually compile something
/// but do not need the Nix dev shell. Pair with
/// [`configure_namespace_sccache`] to point the wrapper at the remote cache.
pub fn setup_rust_sccache() -> Step<Use> {
    uses_local(
        "Setup Rust",
        xtask_paths::repo_dir!(".github/actions/setup-rust"),
    )
    .add_with(("sccache", "true"))
    .add_with(("rust-cache", "false"))
}

/// Install + initialise Nix on the runner. Namespace profiles don't ship Nix,
/// so this must run before [`setup_dev_shell`] (which shells out to `nix`). The
/// `/nix` cache volume mounted by [`mount_cache_volume`] keeps the store warm,
/// so it re-inits the daemon rather than doing a full install.
pub fn setup_nix() -> Step<Use> {
    uses_local(
        "Setup Nix",
        xtask_paths::repo_dir!(".github/actions/setup-nix"),
    )
}

/// Enter the repo's Nix dev shell (toolchain, mold, just, the sccache binary,
/// and `RUSTC_WRAPPER=sccache`) without selecting an sccache provider or
/// configuring an external Nix binary cache. Jobs that compile Rust can follow
/// this with [`configure_namespace_sccache`] to use Namespace's official remote
/// cache. Requires [`setup_nix`] first. The composite action defaults to the
/// `default` flake shell when no `shell` input is passed.
pub fn setup_dev_shell() -> Step<Use> {
    uses_local(
        "Setup Nix dev shell",
        xtask_paths::repo_dir!(".github/actions/setup-nix-dev-shell"),
    )
}

/// [`setup_dev_shell`] for a named `devShells.<name>` flake output.
#[allow(dead_code)]
pub fn setup_dev_shell_named(name: &str) -> Step<Use> {
    setup_dev_shell().add_with(("shell", name))
}

/// Mount the Namespace profile's persisted cache volume: `cache: rust` persists
/// the cargo registry/git, and `path:` persists the Nix store. Compiled objects
/// deliberately use Namespace's official remote sccache instead of this volume.
/// `continue-on-error` because the volume is a pure optimization — a failure
/// just means cold Cargo/Nix state, never a wrong build.
pub fn mount_cache_volume() -> Step<Use> {
    nscloud_cache_action("Mount Namespace cache volume")
        .add_with(("cache", "rust"))
        .add_with(("path", xtask_paths::runtime_path!("/nix").as_str()))
}

/// The pinned `nscloud-cache-action`, shared by every mount helper below.
/// `continue-on-error` because a cache volume is always a pure optimization — a
/// failure just means cold state, never a wrong build.
fn nscloud_cache_action(name: &str) -> Step<Use> {
    Step::new(name)
        .uses(
            "namespacelabs",
            "nscloud-cache-action",
            "15799a6b54e5765f85b2aac25b3f0df43ed571c0", // v1.4.3
        )
        .continue_on_error(true)
}

/// Configure Namespace's official artifact-backed remote sccache. Call this
/// after [`setup_dev_shell`] or [`setup_reqs_web`], which install sccache and
/// export `RUSTC_WRAPPER=sccache`. The short-lived WebDAV credentials work
/// across runners and cache-volume misses. Fork PRs skip this step and retain
/// the setup action's local fallback so untrusted code never receives the
/// runner-minted Namespace workspace token.
pub fn configure_namespace_sccache(cache_name: &str) -> Step<Run> {
    namespace_sccache_step(cache_name)
        .if_condition(Expression::new(TRUSTED_NAMESPACE_SCCACHE_CONTEXT))
}

/// Configure Namespace's remote sccache in a trusted context when
/// `additional_condition` is also true.
pub fn configure_namespace_sccache_when(cache_name: &str, additional_condition: &str) -> Step<Run> {
    namespace_sccache_step(cache_name).if_condition(Expression::new(format!(
        "({TRUSTED_NAMESPACE_SCCACHE_CONTEXT}) && ({additional_condition})"
    )))
}

fn namespace_sccache_step(cache_name: &str) -> Step<Run> {
    Step::new("Configure Namespace remote sccache").run(format!(
        r#"set -euo pipefail
env_file="$(mktemp "$RUNNER_TEMP/namespace-sccache.XXXXXX")"
trap 'rm -f "$env_file"' EXIT
if nsc cache sccache setup --cache_name {cache_name} > "$env_file"; then
  # Register credential values as masked BEFORE exporting: GitHub only
  # masks `secrets.*`, so without this SCCACHE_WEBDAV_TOKEN — a broad,
  # ~24h Namespace workspace token (registry + cache write) — printed
  # verbatim in every subsequent step's env dump. Masking is selective by
  # key name: masking non-secrets like the endpoint URL or key prefix
  # would redact those strings everywhere in the logs.
  while IFS= read -r line; do
    k="${{line%%=*}}"
    v="${{line#*=}}"
    [ -n "$v" ] && [ "$k" != "$line" ] || continue
    case "$k" in
      *TOKEN*|*SECRET*|*PASSWORD*) echo "::add-mask::$v" ;;
    esac
  done < "$env_file"
  cat "$env_file" >> "$GITHUB_ENV"
  # Force the next compiler invocation to start a server with the new remote
  # backend even if a setup hook happened to launch one already.
  sccache --stop-server >/dev/null 2>&1 || true
else
  echo "::warning::Namespace remote sccache setup failed; using local cache fallback"
fi"#
    ))
}

/// Mount the web-app cache volume using Namespace's native Nix integration.
/// Bun's install cache is mounted as an explicit path because Bun comes from
/// the Nix dev shell and is not available when this step runs. `with_rust`
/// additionally persists cargo registry/git data for the `gen-api`
/// OpenAPI-binary build; compiled objects live in Namespace's remote sccache.
/// `continue-on-error` for the same reason as [`mount_cache_volume`].
pub fn mount_web_cache_volume(with_rust: bool) -> Step<Use> {
    nscloud_cache_action("Mount Namespace cache volume")
        .add_with(("cache", "nix"))
        .map(|step| {
            if with_rust {
                step.add_with((
                    "path",
                    format!(
                        "{}\n/home/runner/.cargo/registry\n/home/runner/.cargo/git",
                        vars::BUN_CACHE_VOLUME_DIR,
                    ),
                ))
            } else {
                step.add_with(("path", vars::BUN_CACHE_VOLUME_DIR))
            }
        })
}

/// The web-app composite: Nix dev shell (bun, biome, just) + `bun install`.
/// Jobs that run `gen-api` follow this with [`configure_namespace_sccache`].
/// Requires [`setup_nix`] first.
pub fn setup_reqs_web(name: &str, playwright: bool) -> Step<Use> {
    uses_local(
        name,
        xtask_paths::repo_dir!(".github/actions/setup-reqs-web"),
    )
    .when(playwright, |step| step.add_with(("playwright", "true")))
}

/// `sccache --show-stats` at the end of a job (never fails the job).
pub fn show_sccache_stats() -> Step<Run> {
    Step::new("show sccache stats")
        .run("sccache --show-stats || true")
        .if_condition(Expression::new("always()"))
}

/// Base for jobs gated behind `path-check`: depends on it and runs only on
/// non-draft PRs where the path filter matched. Shared by `check` and `test`.
pub fn gated_job() -> Job {
    Job::default()
        .needs(vec!["path-check".to_string()])
        .cond(Expression::new(
        "needs.path-check.outputs.should_run == 'true' && github.event.pull_request.draft == false",
    ))
}

/// Mount only the `/nix` store cache volume (no cargo/sccache). Used by the
/// jobs that delegate entirely to Nix.
pub fn mount_nix_cache_volume() -> Step<Use> {
    nscloud_cache_action("Mount /nix cache volume").add_with(("cache", "nix"))
}

/// Teardown Nix (always runs).
pub fn teardown_nix() -> Step<Use> {
    uses_local(
        "Teardown Nix",
        xtask_paths::repo_dir!(".github/actions/teardown-nix"),
    )
    .if_condition(Expression::new("always()"))
}
