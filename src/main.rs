use determinator::{rules::DeterminatorRules, Determinator};
use guppy::{
    graph::{BuildTarget, BuildTargetKind, DependencyDirection},
    CargoMetadata,
};

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

    for package in determinator_set
        .path_changed_set
        .packages(DependencyDirection::Forward)
    {
        println!("Path changed: {}", package.name());
    }

    for package in determinator_set
        .summary_changed_set
        .packages(DependencyDirection::Forward)
    {
        println!("Summary changed: {}", package.name());
    }

    // determinator_set.affected_set contains the workspace packages directly or indirectly affected
    // by the change.
    for package in determinator_set
        .affected_set
        .packages(DependencyDirection::Forward)
    {
        if package.has_test_targets() {
            println!("should test: {}", package.name());
        }
    }

    for package in determinator_set
        .affected_set
        .root_packages(DependencyDirection::Forward)
    {
        let targets = package.binary_targets();
        if !targets.is_empty() {
            println!("package: {}", package.name());
            for target in targets {
                println!("target: {}", target.name());
            }
        }
    }
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
