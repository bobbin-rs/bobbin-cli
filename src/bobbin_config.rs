
#[derive(Debug, Deserialize)]
pub struct BobbinConfig {
    pub filter: Option<FilterConfig>,
    pub console: Option<ConsoleConfig>,
    pub builder: Option<BuilderConfig>,
    pub loader: Option<LoaderConfig>,
    pub itm: Option<ItmConfig>,
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
pub struct ConsoleConfig {
    pub device: Option<String>,
    pub path: Option<String>,
    pub speed: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ItmConfig {
    #[serde(rename = "target-clock")]
    pub target_clock: Option<u32>,
}


#[derive(Debug, Deserialize)]
pub struct LoaderConfig {
    #[serde(rename = "jlink-device")]
    pub jlink_device: Option<String>,
    #[serde(rename = "teensy-mcu")]
    pub teensy_mcu: Option<String>,
    #[serde(rename = "blackmagic-mode")]
    pub blackmagic_mode: Option<String>,
    pub offset: Option<String>,
}