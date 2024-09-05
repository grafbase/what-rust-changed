#![allow(unstable_name_collisions)]

mod config;
mod guppy_ext;

use determinator::Determinator;
use guppy::{graph::DependencyDirection, CargoMetadata};
use guppy_ext::PackageMetadataExt;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Report {
    /// A string that can be passed to cargo build to limit to changed packages
    cargo_build_specs: String,

    /// A string that can be passed to cargo test to limit to changed packages
    ///
    /// This one should be used for platforms that do not support docker
    cargo_test_specs: String,

    /// A string that can be passed to cargo test to limit to changed packages
    ///
    /// This one should be used for platforms that support docker
    cargo_docker_test_specs: String,

    /// A string that can be passed to cargo build to limit only to changed binaries
    cargo_bin_specs: String,

    /// The full list of packages that have changed, useful for other CI filtering purposes
    changed_packages: Vec<String>,

    /// The full list of binaries that have changed, useful for other CI filtering purposes
    changed_binaries: Vec<String>,
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    let config = config::load();

    eprintln!("Comparing metadata from {} to {}", &args[1], &args[2]);

    // guppy accepts `cargo metadata` JSON output. Use a pre-existing fixture for these examples.
    let old_metadata =
        CargoMetadata::parse_json(std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    let old = old_metadata.build_graph().unwrap();
    let new_metadata =
        CargoMetadata::parse_json(std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let new = new_metadata.build_graph().unwrap();

    let mut determinator = Determinator::new(&old, &new);

    determinator.set_rules(&config.determinator_rules).unwrap();

    eprintln!("Changed files:");
    for path in &args[3..] {
        eprintln!("- {path}");
    }
    eprintln!();

    // The determinator expects a list of changed files to be passed in.
    determinator.add_changed_paths(&args[3..]);

    let determinator_set = determinator.compute();

    let cargo_build_specs = determinator_set
        .affected_set
        .packages(DependencyDirection::Forward)
        .flat_map(|package| ["-p", package.name()])
        .collect::<Vec<_>>()
        .join(" ");

    let cargo_test_specs = determinator_set
        .affected_set
        .packages(DependencyDirection::Forward)
        .filter(|package| package.has_test_targets())
        .filter(|package| {
            !config
                .ignore_test_packages
                .iter()
                .any(|name| package.name() == name)
        })
        .filter(|package| {
            !config
                .docker_test_packages
                .iter()
                .any(|name| package.name() == name)
        })
        .flat_map(|package| ["-p", package.name()])
        .collect::<Vec<_>>()
        .join(" ");

    let cargo_docker_test_specs = determinator_set
        .affected_set
        .packages(DependencyDirection::Forward)
        .filter(|package| package.has_test_targets())
        .filter(|package| {
            !config
                .ignore_test_packages
                .iter()
                .any(|name| package.name() == name)
        })
        .flat_map(|package| ["-p", package.name()])
        .collect::<Vec<_>>()
        .join(" ");

    let changed_packages = determinator_set
        .affected_set
        .packages(DependencyDirection::Forward)
        .map(|package| package.name().to_string())
        .collect();

    let cargo_bin_specs = determinator_set
        .affected_set
        .root_packages(DependencyDirection::Forward)
        .flat_map(|package| {
            package
                .binary_targets()
                .into_iter()
                .flat_map(|target| ["--bin".into(), target.name().to_string()])
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join(" ");

    let changed_binaries = determinator_set
        .affected_set
        .packages(DependencyDirection::Forward)
        .flat_map(|package| {
            package
                .binary_targets()
                .into_iter()
                .map(|target| target.name().to_string())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let report = Report {
        cargo_build_specs,
        cargo_test_specs,
        cargo_docker_test_specs,
        cargo_bin_specs,
        changed_packages,
        changed_binaries,
    };

    println!("{}", serde_json::to_string(&report).unwrap())
}
