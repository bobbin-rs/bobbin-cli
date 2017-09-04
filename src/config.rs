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
        if let Some(ref bobbin) = self.bobbin {
            if let Some(ref itm) = bobbin.itm {
                return itm.target_clock
            }
        }
        None        
    }

    pub fn jlink_device(&self) -> Option<&str> {
        if let Some(ref bobbin) = self.bobbin {
            if let Some(ref loader) = bobbin.loader {
                if let Some(ref jlink_device) = loader.jlink_device {
                    return Some(jlink_device)
                }
            }
        }
        None
    }    

    pub fn teensy_mcu(&self) -> Option<&str> {
        if let Some(ref bobbin) = self.bobbin {
            if let Some(ref loader) = bobbin.loader {
                if let Some(ref teensy_mcu) = loader.teensy_mcu {
                    return Some(teensy_mcu)
                }
            }
        }
        None        
    }


    pub fn blackmagic_mode(&self) -> Option<&str> {
        if let Some(ref bobbin) = self.bobbin {
            if let Some(ref loader) = bobbin.loader {
                if let Some(ref blackmagic_mode) = loader.blackmagic_mode {
                    return Some(blackmagic_mode)
                }
            }
        }
        None
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

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Option<String>> {
    let path = path.as_ref();
    if path.exists() {
        let mut data = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut data)?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

pub fn read_bobbin() -> Result<Option<BobbinConfig>> {    
    if let Some(s) = read_file("./bobbin/config")? {
        Ok(Some(toml::from_str(&s)?))
    } else {
        Ok(None)
    }
}

pub fn read_cargo() -> Result<Option<CargoConfig>> {
    if let Some(s) = read_file("./cargo/config")? {
        Ok(Some(toml::from_str(&s)?))
    } else {
        Ok(None)
    }
}
