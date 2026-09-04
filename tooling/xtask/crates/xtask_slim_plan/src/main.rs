//! `cargo x slim-plan`
//!
//! Analyzes real marginal removable closures and top-heavy dependencies for cargo decoupling.

use anyhow::{Context, Result, bail};
use clap::Parser;
use guppy::PackageId;
use guppy::graph::feature::FeatureId;
use guppy::graph::{DependencyDirection, PackageGraph, PackageMetadata};
use std::collections::BTreeSet;
use xtask_graph::build_graph;

#[derive(Parser, Debug)]
#[command(
    name = "xtask_slim_plan",
    about = "Cargo decoupling change planner & marginal closure analyzer"
)]
struct Cli {
    /// The root workspace package to analyze (e.g. email_service)
    #[arg(short = 'p', long = "package")]
    package: String,

    /// Target dependency to analyze and simulate cutting
    #[arg(value_name = "TARGET")]
    target: Option<String>,

    /// Rank top-heavy dependencies by real marginal removable closure
    #[arg(long = "top-heavy")]
    top_heavy: bool,

    /// Number of top dependencies to list (defaults to 10)
    #[arg(long = "limit", default_value = "10")]
    limit: usize,
}

fn compute_workspace_closure(
    graph: &PackageGraph,
    root_id: &PackageId,
    excluded_links: &[(PackageId, PackageId)],
) -> Result<BTreeSet<String>> {
    let query = graph
        .query_forward([root_id])
        .context("querying root package")?;
    let set = query.resolve_with_fn(|_, link| {
        if !link.to().in_workspace() || link.dev_only() {
            return false;
        }
        let from_id = link.from().id().clone();
        let to_id = link.to().id().clone();
        !excluded_links
            .iter()
            .any(|(from, to)| from == &from_id && to == &to_id)
    });

    let packages = set
        .packages(DependencyDirection::Forward)
        .filter(|p| p.in_workspace())
        .map(|p| p.name().to_owned())
        .collect();

    Ok(packages)
}

fn compute_raw_closure(graph: &PackageGraph, pkg_id: &PackageId) -> Result<BTreeSet<String>> {
    let query = graph.query_forward([pkg_id]).context("querying package")?;
    let set = query.resolve_with_fn(|_, link| link.to().in_workspace() && !link.dev_only());
    let packages = set
        .packages(DependencyDirection::Forward)
        .filter(|p| p.in_workspace())
        .map(|p| p.name().to_owned())
        .collect();
    Ok(packages)
}

fn find_workspace_package<'a>(graph: &'a PackageGraph, name: &str) -> Option<PackageMetadata<'a>> {
    graph.workspace().iter().find(|p| p.name() == name)
}

fn collect_dependency_paths(
    graph: &PackageGraph,
    current_id: &PackageId,
    target_name: &str,
    current_path: &mut Vec<String>,
    all_paths: &mut Vec<Vec<String>>,
    visited: &mut BTreeSet<PackageId>,
) {
    let pkg = match graph.metadata(current_id) {
        Ok(p) => p,
        Err(_) => return,
    };

    if pkg.name() == target_name {
        all_paths.push(current_path.clone());
        return;
    }

    visited.insert(current_id.clone());

    for link in pkg.direct_links() {
        if !link.to().in_workspace() || link.dev_only() {
            continue;
        }
        let next_id = link.to().id();
        if visited.contains(next_id) {
            continue;
        }
        current_path.push(link.to().name().to_owned());
        collect_dependency_paths(
            graph,
            next_id,
            target_name,
            current_path,
            all_paths,
            visited,
        );
        current_path.pop();
    }

    visited.remove(current_id);
}

fn check_is_optional(link: &guppy::graph::PackageLink) -> bool {
    let req = link.normal();
    req.status().is_never() || req.status().optional_status().is_present()
}

fn find_activating_features(
    graph: &PackageGraph,
    root_pkg: &PackageMetadata,
    target_id: &PackageId,
    is_optional: bool,
) -> Vec<String> {
    if !is_optional {
        return vec!["<mandatory: unconditionally compiled>".to_string()];
    }

    let feature_graph = graph.feature_graph();
    let target_base_feature = FeatureId::base(target_id);
    let mut activating = Vec::new();

    for feature_name in root_pkg.named_features() {
        let root_feature = FeatureId::named(root_pkg.id(), feature_name);
        if let Ok(true) = feature_graph.depends_on(root_feature, target_base_feature) {
            activating.push(feature_name.to_owned());
        }
    }

    let default_feature = root_pkg.default_feature_id();
    if let Ok(true) = feature_graph.depends_on(default_feature, target_base_feature) {
        if !activating.contains(&"default".to_string()) {
            activating.push("default".to_string());
        }
    }

    activating
}

fn run_single_target(
    graph: &PackageGraph,
    root_pkg: &PackageMetadata,
    target_name: &str,
) -> Result<()> {
    let target_pkg = find_workspace_package(graph, target_name)
        .with_context(|| format!("target package '{target_name}' not found in workspace"))?;

    let root_id = root_pkg.id();
    let target_id = target_pkg.id();

    // 1. Check direct link
    let direct_link = root_pkg
        .direct_links()
        .find(|link| link.to().id() == target_id);
    let is_direct = direct_link.is_some();

    // 2. Find all paths from root to target
    let mut all_paths = Vec::new();
    let mut current_path = vec![root_pkg.name().to_owned()];
    let mut visited = BTreeSet::new();
    collect_dependency_paths(
        graph,
        root_id,
        target_name,
        &mut current_path,
        &mut all_paths,
        &mut visited,
    );

    if all_paths.is_empty() {
        println!("TARGET: {}", target_name);
        println!("ROOT:   {}", root_pkg.name());
        println!(
            "Status: '{}' is NOT in the workspace dependency tree of '{}'.",
            target_name,
            root_pkg.name()
        );
        return Ok(());
    }

    let mut dep_kind = "transitive";
    let mut default_features = true;
    let mut req_features: Vec<String> = Vec::new();
    let mut activating_features: Vec<String> = Vec::new();

    if let Some(link) = direct_link {
        let opt = check_is_optional(&link);
        dep_kind = if opt {
            "direct optional"
        } else {
            "direct mandatory"
        };
        let req = link.normal();
        default_features = !req.default_features().is_never();
        req_features = req.features().map(|s| s.to_string()).collect();
        activating_features = find_activating_features(graph, root_pkg, target_id, opt);
    }

    // 4. Closures: Before vs After cut
    let base_closure = compute_workspace_closure(graph, root_id, &[])?;
    let raw_target_closure = compute_raw_closure(graph, target_id)?;

    // Cut edges from root to target (or all incoming edges if transitive)
    let cut_links: Vec<(PackageId, PackageId)> = if is_direct {
        vec![(root_id.clone(), target_id.clone())]
    } else {
        let mut immediate_leading_to_target = Vec::new();
        for path in &all_paths {
            if path.len() >= 2 {
                if let Some(pkg) = find_workspace_package(graph, &path[1]) {
                    immediate_leading_to_target.push((root_id.clone(), pkg.id().clone()));
                }
            }
        }
        immediate_leading_to_target
    };

    let new_closure = compute_workspace_closure(graph, root_id, &cut_links)?;
    let mut marginal_removable: BTreeSet<String> =
        base_closure.difference(&new_closure).cloned().collect();
    if !new_closure.contains(target_name) {
        marginal_removable.insert(target_name.to_owned());
    }

    let shared_dependencies: BTreeSet<String> = raw_target_closure
        .difference(&marginal_removable)
        .cloned()
        .collect();

    // 5. Formatted Output
    println!("================================================================================");
    println!(
        "🎯 Macro Slimming Plan: {} ➔ {}",
        root_pkg.name(),
        target_name
    );
    println!("================================================================================");
    println!("1. BASIC ATTRIBUTES");
    println!("   - Root Package:        {}", root_pkg.name());
    println!("   - Target Package:      {}", target_name);
    println!(
        "   - Relation:            {}",
        if is_direct {
            "Direct Dependency"
        } else {
            "Transitive Dependency Only"
        }
    );
    println!("   - Cargo Dependency:    {}", dep_kind);
    if is_direct {
        println!("   - Default Features:    {}", default_features);
        println!("   - Requested Features:  [{}]", req_features.join(", "));
        if !activating_features.is_empty() {
            println!(
                "   - Activating Features: [{}]",
                activating_features.join(", ")
            );
        }
    }

    println!(
        "\n2. WORKSPACE DEPENDENCY PATHS ({} found)",
        all_paths.len()
    );
    for (idx, path) in all_paths.iter().take(5).enumerate() {
        println!("   [{}] {}", idx + 1, path.join(" ➔ "));
    }
    if all_paths.len() > 5 {
        println!("   ... and {} more path(s)", all_paths.len() - 5);
    }

    println!("\n3. CLOSURE REDUCTION IMPACT");
    println!(
        "   - Current Workspace Closure:       {} crates",
        base_closure.len()
    );
    println!(
        "   - Simulated New Closure:           {} crates",
        new_closure.len()
    );
    println!(
        "   - Raw Target Closure:              {} crates",
        raw_target_closure.len()
    );
    println!(
        "   - ⚡ REAL MARGINAL REMOVABLE:       {} crates",
        marginal_removable.len()
    );
    println!(
        "   - 🔒 Shared Blockers (Retained):    {} crates",
        shared_dependencies.len()
    );

    if !marginal_removable.is_empty() {
        println!(
            "\n   Crates completely eliminated ({}):",
            marginal_removable.len()
        );
        for crate_name in &marginal_removable {
            println!("     ✓ {}", crate_name);
        }
    }

    if !shared_dependencies.is_empty() {
        println!(
            "\n   Shared crates retained by other paths ({}):",
            shared_dependencies.len()
        );
        for crate_name in shared_dependencies.iter().take(8) {
            println!("     • {}", crate_name);
        }
        if shared_dependencies.len() > 8 {
            println!("     • ... and {} more", shared_dependencies.len() - 8);
        }
    }

    println!("\n4. CUT ACTION & RECOMMENDATIONS");
    if is_direct {
        println!(
            "   - Action: Convert '{}' to an optional dependency in services/{}/Cargo.toml",
            target_name,
            root_pkg.name()
        );
        println!("   - Suggested Cargo.toml configuration:");
        println!("       [features]");
        println!("       {} = [\"dep:{}\"]", target_name, target_name);
        println!("       [dependencies]");
        println!(
            "       {} = {{ path = \"...\", optional = true }}",
            target_name
        );
        println!(
            "   - Next Step: Run CodeGraph to find all call sites in {}:",
            root_pkg.name()
        );
        println!("       codegraph explore \"{}\"", target_name);
        println!("       codegraph callers \"{}\"", target_name);
        println!("   - Gate with: #[cfg(feature = \"{}\")]", target_name);
    } else {
        println!(
            "   - Action: Target is indirect. Decouple intermediate crates listed in PATHS above."
        );
    }
    println!("================================================================================\n");

    Ok(())
}

fn run_top_heavy(graph: &PackageGraph, root_pkg: &PackageMetadata, limit: usize) -> Result<()> {
    let root_id = root_pkg.id();
    let base_closure = compute_workspace_closure(graph, root_id, &[])?;

    println!("================================================================================");
    println!("📊 Top-Heavy Dependency Analysis for: {}", root_pkg.name());
    println!("Current Workspace Closure: {} crates", base_closure.len());
    println!("Ranking criteria: Real Marginal Removable Closure (Counterfactual Elimination)");
    println!("================================================================================\n");

    let mut candidates = Vec::new();

    for link in root_pkg.direct_links() {
        if !link.to().in_workspace() || link.dev_only() {
            continue;
        }
        let target_pkg = link.to();
        let target_id = target_pkg.id();
        let target_name = target_pkg.name();

        let cut_links = vec![(root_id.clone(), target_id.clone())];
        let new_closure = compute_workspace_closure(graph, root_id, &cut_links)?;
        let marginal: BTreeSet<String> = base_closure.difference(&new_closure).cloned().collect();
        let raw_closure = compute_raw_closure(graph, target_id)?;
        let shared: BTreeSet<String> = raw_closure.difference(&marginal).cloned().collect();

        let is_optional = check_is_optional(&link);

        candidates.push((
            target_name.to_owned(),
            is_optional,
            raw_closure.len(),
            shared.len(),
            marginal.len(),
        ));
    }

    // Sort by marginal removable descending
    candidates.sort_by(|a, b| b.4.cmp(&a.4));

    println!(
        "{:<4} {:<32} {:<10} {:<12} {:<10} {:<10}",
        "Rank", "Dependency", "Type", "Raw Closure", "Shared", "Marginal"
    );
    println!("{:-<84}", "");

    for (idx, (name, is_opt, raw_len, shared_len, marginal_len)) in
        candidates.iter().take(limit).enumerate()
    {
        let type_str = if *is_opt { "Optional" } else { "Mandatory" };
        println!(
            "{:<4} {:<32} {:<10} {:<12} {:<10} ⚡ {:<8}",
            idx + 1,
            name,
            type_str,
            format!("{} crates", raw_len),
            format!("{} crates", shared_len),
            format!("-{} crates", marginal_len),
        );
    }

    println!(
        "\n💡 Recommendation: Focus first on Mandatory dependencies with highest Marginal Reduction."
    );
    println!(
        "Run `cargo x slim-plan -p {} <dependency>` for in-depth path and cut analysis.",
        root_pkg.name()
    );
    println!("================================================================================\n");

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let graph = build_graph(false).context("building workspace package graph")?;

    let root_pkg = find_workspace_package(&graph, &cli.package)
        .with_context(|| format!("workspace package '{}' not found", cli.package))?;

    if cli.top_heavy {
        run_top_heavy(&graph, &root_pkg, cli.limit)?;
    } else if let Some(target) = cli.target {
        run_single_target(&graph, &root_pkg, &target)?;
    } else {
        bail!("either a TARGET dependency or --top-heavy must be specified. See --help.");
    }

    Ok(())
}
