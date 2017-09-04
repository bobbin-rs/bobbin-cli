
#[derive(Debug, Deserialize)]
pub struct BobbinConfig {
    pub filter: Option<FilterConfig>,
    pub builder: Option<BuilderConfig>,
    pub loader: Option<LoaderConfig>,
}

#[derive(Debug, Deserialize)]
pub struct FilterConfig {
    pub host: Option<String>,
    pub device: Option<String>,    
}

#[derive(Debug, Deserialize)]
pub struct BuilderConfig {
    pub target: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoaderConfig {
    #[serde(rename = "jlink-device")]
    pub jlink_device: Option<String>,
    #[serde(rename = "teensy-mcu")]
    pub teensy_mcu: Option<String>,
}