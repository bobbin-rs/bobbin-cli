use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct CargoConfig {
    pub build: Option<BuildConfig>,
    pub target: Option<HashMap<String, Target>>,
}

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    pub target: Option<String>,    
}

#[derive(Debug, Deserialize)]
pub struct Target {

}
