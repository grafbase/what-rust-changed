use std::fs::read_to_string;

pub fn load() -> Config {
    do_load().unwrap_or_default()
}

fn do_load() -> Option<Config> {
    let path = std::env::var("WHAT_RUST_CHANGED_CONFIG").ok()?;

    eprintln!("Loading config from {path}");

    let data = read_to_string(path).expect("to be able to read named config file");

    dbg!(toml::from_str(&data).expect("invalid config format"))
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Packages that shouldn't automatically be included in test runs
    pub ignore_test_packages: Vec<String>,

    /// Packages that require docker for their tests and should be put
    /// into a separate section of the output
    pub docker_test_packages: Vec<String>,

    #[serde(flatten)]
    pub determinator_rules: determinator::rules::DeterminatorRules,
}

impl Config {
    pub fn mark_all_changed(&mut self) {
        // Bit of a hack, but this seems like it'll work.
        // It won't mark everything as changed if there are literally no changed files
        // but not sure if that's an edge case we care about?
        self.determinator_rules.path_rules.insert(
            0,
            determinator::rules::PathRule {
                globs: vec!["**".into()],
                mark_changed: determinator::rules::DeterminatorMarkChanged::All,
                post_rule: Default::default(),
            },
        )
    }
}
