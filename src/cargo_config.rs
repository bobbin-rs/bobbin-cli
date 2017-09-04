#[derive(Debug, Deserialize)]
pub struct CargoConfig {
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    pub target: Option<String>,
}
