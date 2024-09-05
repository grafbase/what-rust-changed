use guppy::graph::{BuildTarget, BuildTargetKind};

pub trait PackageMetadataExt {
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
