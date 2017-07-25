use clap::ArgMatches;
use Result;
use toml::value::{Value, Table};
use std::io::Read;
use std::fs::File;
use std::path::{Path, PathBuf};


pub fn config(args: &ArgMatches) -> Result<Config> {
    Ok(Config {
        bobbin_cfg: read_bobbin_config()?,
        cargo_cfg: read_cargo_config()?,
        cargo: read_cargo()?,
    })
}

#[derive(Debug)]
pub struct Config {
    bobbin_cfg: Value,
    cargo_cfg: Value,
    cargo: Value,
}

impl Config {
    pub fn default_target(&self) -> Option<PathBuf> {
        if let Some(build) = self.cargo_cfg.get("build") {
            build.as_table().unwrap()["target"]
                .as_str()
                .map(PathBuf::from)
        } else {
            None
        }
    }

    pub fn default_binary(&self) -> Option<PathBuf> {
        if let Some(package) = self.cargo.get("package") {
            package.as_table().unwrap()["name"]
                .as_str()
                .map(PathBuf::from)
        } else {
            None
        }
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

pub fn read_cargo() -> Result<Value> {
    read_toml(Path::new("./Cargo.toml"))
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
