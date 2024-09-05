use std::fs::read_to_string;

pub fn load() -> Config {
    do_load().unwrap_or_default()
}

fn do_load() -> Option<Config> {
    let path = std::env::var("WHAT_RUST_CHANGED_CONFIG").ok()?;

    eprintln!("Loading config from {path}");

    let data = read_to_string(path).expect("to be able to read named config file");

    toml::from_str(&data).expect("invalid config format")
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Config {
    /// Packages that shouldn't automatically be included in test runs
    pub ignore_test_packages: Vec<String>,

    /// Packages that require docker for their tests and should be put
    /// into a separate section of the output
    pub docker_test_packages: Vec<String>,

    #[serde(flatten)]
    pub determinator_rules: determinator::rules::DeterminatorRules,
}
