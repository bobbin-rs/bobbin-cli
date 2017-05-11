use clap::{ArgMatches};
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
        self.cargo_cfg["build"]
            .as_table().unwrap()["target"]
            .as_str().map(PathBuf::from)
    }

    pub fn default_binary(&self) -> Option<PathBuf> {
        self.cargo["package"]
            .as_table().unwrap()["name"]
            .as_str().map(PathBuf::from)
    }

    pub fn default_filter(&self) -> &Value {
        &self.bobbin_cfg["filter"]
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
        return Ok(Value::Table(Table::new()))
    }
    let mut f = File::open(path)?;
    let mut data = String::new();
    f.read_to_string(&mut data)?;
    let value = data.parse::<Value>()?;
    Ok(value)
}