use bobbin_config::BobbinConfig;
use cargo_config::CargoConfig;
use clap::ArgMatches;
use Result;
use toml;
use toml::value::{Value, Table};
use std::io::Read;
use std::fs::File;
use std::path::Path;


pub fn config(args: &ArgMatches) -> Result<Config> {    
    Ok(Config {
        bobbin: read_bobbin()?,
        cargo: read_cargo()?,
        bobbin_cfg: read_bobbin_config()?,
        cargo_cfg: read_cargo_config()?,
    })
}

#[derive(Debug)]
pub struct Config {
    bobbin: Option<BobbinConfig>,
    cargo: Option<CargoConfig>,
    bobbin_cfg: Value,
    cargo_cfg: Value,
}

impl Config {
    pub fn target(&self) -> Option<&str> {
        if let Some(ref bobbin) = self.bobbin {
            if let Some(ref builder) = bobbin.builder {
                if let Some(ref target) = builder.target {
                    return Some(target)
                }
            }
        }

        if let Some(ref cargo) = self.cargo {
            if let Some(ref build) = cargo.build {
                if let Some(ref target) = build.target {
                    return Some(target)
                }

            }
        }

        None
    }

    pub fn default_filter(&self) -> Option<&Value> {
        self.bobbin_cfg.get("filter")
    }

    pub fn default_loader(&self) -> Option<&Value> {
        self.bobbin_cfg.get("loader")
    }

    pub fn default_itm(&self) -> Option<&Value> {
        self.bobbin_cfg.get("itm")
    }

    pub fn itm_target_clock(&self) -> Option<u32> {
        if let Some(default_itm) = self.default_itm() {
            if let Some(default_itm) = default_itm.as_table() {
                if let Some(target_clock) = default_itm.get("target-clock") {
                    if let Some(target_clock) = target_clock.as_integer() {
                        Some(target_clock as u32)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn jlink_device(&self) -> Option<&str> {
        if let Some(default_loader) = self.default_loader() {
            if let Some(ldr_cfg) = default_loader.as_table() {
                if let Some(jlink_device) = ldr_cfg.get("jlink-device") {
                    jlink_device.as_str()
                } else {
                    None
                }   
            } else {
                None
            }
        } else {
            None
        }
    }    

    pub fn teensy_mcu(&self) -> Option<&str> {
        if let Some(default_loader) = self.default_loader() {
            if let Some(ldr_cfg) = default_loader.as_table() {
                if let Some(teensy_mcu) = ldr_cfg.get("teensy-mcu") {
                    teensy_mcu.as_str()
                } else {
                    None
                }   
            } else {
                None
            }
        } else {
            None
        }
    }


    pub fn blackmagic_mode(&self) -> Option<&str> {
        if let Some(default_loader) = self.default_loader() {
            if let Some(ldr_cfg) = default_loader.as_table() {
                if let Some(blackmagic_mode) = ldr_cfg.get("blackmagic-mode") {
                    blackmagic_mode.as_str()
                } else {
                    None
                }   
            } else {
                None
            }
        } else {
            None
        }
    }    
}

pub fn read_cargo_config() -> Result<Value> {
    read_toml(Path::new("./.cargo/config"))
}

pub fn read_bobbin_config() -> Result<Value> {
    read_toml(Path::new("./.bobbin/config"))
}

pub fn read_toml<P: AsRef<Path>>(path: P) -> Result<Value> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Value::Table(Table::new()));
    }
    let mut f = File::open(path)?;
    let mut data = String::new();
    f.read_to_string(&mut data)?;
    let value = data.parse::<Value>()?;
    Ok(value)
}

pub fn read_bobbin() -> Result<Option<BobbinConfig>> {
    let path = Path::new("./.bobbin/config");
    if path.exists() {
        let mut f = File::open(path)?;
        let mut data = String::new();
        f.read_to_string(&mut data)?;
        let config: BobbinConfig = toml::from_str(&data)?;
        Ok(Some(config))
       
    } else {
        Ok(None)
    }
}


pub fn read_cargo() -> Result<Option<CargoConfig>> {
    let path = Path::new("./.cargo/config");
    if path.exists() {
        let mut f = File::open(path)?;
        let mut data = String::new();
        f.read_to_string(&mut data)?;
        let config: CargoConfig = toml::from_str(&data)?;
        Ok(Some(config))
       
    } else {
        Ok(None)
    }
}
