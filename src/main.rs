#![allow(unstable_name_collisions)]

use determinator::{rules::DeterminatorRules, Determinator};
use guppy::{
    graph::{BuildTarget, BuildTargetKind, DependencyDirection},
    CargoMetadata,
};

// TODO: Make this configurable via a file once I've finished iterating
const TESTS_THAT_NEED_DOCKER: &[&str] = &["integration-tests", "grafbase-gateway"];

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    // guppy accepts `cargo metadata` JSON output. Use a pre-existing fixture for these examples.
    let old_metadata =
        CargoMetadata::parse_json(std::fs::read_to_string(dbg!(&args[1])).unwrap()).unwrap();
    let old = old_metadata.build_graph().unwrap();
    let new_metadata =
        CargoMetadata::parse_json(std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let new = new_metadata.build_graph().unwrap();

    let mut determinator = Determinator::new(&old, &new);

    // The determinator supports custom rules read from a TOML file.
    // let rules =
    //     DeterminatorRules::parse(include_str!("../../../fixtures/guppy/path-rules.toml")).unwrap();

    determinator
        .set_rules(DeterminatorRules::default_rules())
        .unwrap();

    // The determinator expects a list of changed files to be passed in.
    determinator.add_changed_paths(dbg!(&args[3..]));

    let determinator_set = determinator.compute();

    let cargo_build_specs = determinator_set
        .affected_set
        .packages(DependencyDirection::Forward)
        .filter(|package| {
            // TODO: Make this configurable somehow...
            package.name() != "grafbase-docker-tests"
        })
        .flat_map(|package| ["-p", package.name()])
        .collect::<Vec<_>>()
        .join(" ");

    let cargo_test_specs = determinator_set
        .affected_set
        .packages(DependencyDirection::Forward)
        .filter(|package| package.has_test_targets())
        .filter(|package| {
            // TODO: Make this configurable somehow...
            package.name() != "grafbase-docker-tests"
        })
        .filter(|package| !TESTS_THAT_NEED_DOCKER.contains(&package.name()))
        .flat_map(|package| ["-p", package.name()])
        .collect::<Vec<_>>()
        .join(" ");

    let cargo_docker_test_specs = determinator_set
        .affected_set
        .packages(DependencyDirection::Forward)
        .filter(|package| package.has_test_targets())
        .filter(|package| {
            // TODO: Make this configurable somehow...
            package.name() != "grafbase-docker-tests"
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

trait PackageMetadataExt {
    fn has_test_targets(&self) -> bool;
    fn binary_targets(&self) -> Vec<BuildTarget<'_>>;
}

impl PackageMetadataExt for guppy::graph::PackageMetadata<'_> {
    fn has_test_targets(&self) -> bool {
        let package_root_path = self
            .manifest_path()
            .parent()
            .expect("all packages to have manifests with one parent");

        self.build_targets()
            .filter(|target| {
                matches!(
                    target.kind(),
                    BuildTargetKind::Binary | BuildTargetKind::LibraryOrExample(_)
                )
            })
            .any(|target| {
                let relative_path = target
                    .path()
                    .strip_prefix(package_root_path)
                    .expect("targets to live inside package");

                let Some(root_folder) = relative_path.components().next() else {
                    return false;
                };

                let root_folder = root_folder.as_str();

                // Unfortunately doesn't seem to be any way to tell whether something rooted in
                // src actually has tests or not, so best to just assume they do
                root_folder == "tests" || root_folder == "src"
            })
    }

    fn binary_targets(&self) -> Vec<BuildTarget<'_>> {
        let package_root_path = self
            .manifest_path()
            .parent()
            .expect("all packages to have manifests with one parent");

        self.build_targets()
            .filter(|target| matches!(target.kind(), BuildTargetKind::Binary))
            .filter(|target| {
                let relative_path = target
                    .path()
                    .strip_prefix(package_root_path)
                    .expect("targets to live inside package");

                let Some(root_folder) = relative_path.components().next() else {
                    return false;
                };

                root_folder.as_str() == "src" && relative_path.file_name() == Some("main.rs")
            })
            .collect()
    }
}
