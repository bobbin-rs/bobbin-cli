use std::io;
use std::process::Command;
use std::io::Write;
use regex::bytes::Regex;
use tempfile;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Status,
    Regex,
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}


pub fn which(exec: &str) -> Result<String, Error> {
    let out = Command::new("which").arg(exec).output()?;
    if out.status.success() {
        let re = Regex::new(r"(.*)").unwrap();
        let caps = re.captures(&out.stdout).unwrap();
        Ok(String::from_utf8_lossy(&caps[1]).into_owned())
    } else {
        panic!("error");
    }
}

pub fn rust_version() -> Result<String, Error> {
    let out = Command::new("rustc").arg("--version").output()?;
    if out.status.success() {
        let re = Regex::new(r"rustc (.*)").unwrap();
        let caps = re.captures(&out.stdout).unwrap();
        Ok(String::from_utf8_lossy(&caps[1]).into_owned())
    } else {
        panic!("error");
    }
}
pub fn cargo_version() -> Result<String, Error> {
    let out = Command::new("cargo").arg("--version").output()?;
    if out.status.success() {
        let re = Regex::new(r"cargo (.*)").unwrap();
        let caps = re.captures(&out.stdout).unwrap();
        Ok(String::from_utf8_lossy(&caps[1]).into_owned())
    } else {
        panic!("error");
    }
}

pub fn xargo_version() -> Result<String, Error> {
    let out = Command::new("xargo").arg("--version").output()?;
    if out.status.success() {
        let re = Regex::new(r"xargo (.*)\n").unwrap();
        let caps = re.captures(&out.stderr).unwrap();
        Ok(String::from_utf8_lossy(&caps[1]).into_owned())
    } else {
        panic!("error");
    }
}


pub fn openocd_version() -> Result<String, Error> {
    let out = Command::new("openocd").arg("--version").output()?;
    if out.status.success() {
        let re = Regex::new(r"Open On-Chip Debugger (.*)").unwrap();
        let caps = re.captures(&out.stderr).unwrap();
        Ok(String::from_utf8_lossy(&caps[1]).into_owned())
    } else {
        Err(Error::Status)
    }
}


pub fn gcc_version() -> Result<String, Error> {
    let out = Command::new("arm-none-eabi-gcc").arg("--version").output()?;
    if out.status.success() {
        let re = Regex::new(r"arm-none-eabi-gcc \(GNU Tools for ARM Embedded Processors\) (.*)").unwrap();
        if let Some(caps) = re.captures(&out.stdout) {
            Ok(String::from_utf8_lossy(&caps[1]).into_owned())
        } else {
            Err(Error::Regex)
        }
    } else {
        Err(Error::Status)
    }
}

pub fn bossac_version() -> Result<String, Error> {
    let out = Command::new("bossac").arg("-h").output()?;
    let re = Regex::new(r"(?m).*\nBasic Open Source SAM-BA Application \(BOSSA\) Version (.*)\n").unwrap();
    if let Some(caps) = re.captures(&out.stdout) {
        Ok(String::from_utf8_lossy(&caps[1]).into_owned())
    } else {
        Err(Error::Regex)
    }
}

pub fn jlink_version() -> Result<String, Error> {
    let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
    try!(writeln!(tmpfile, "exit"));

    let out = Command::new("JLinkExe")
        .arg("-ExitOnError")
        .arg("1")
        .arg("-CommanderScript")
        .arg(tmpfile.path())
        .output()?;
    let re = Regex::new(r"(?m)SEGGER J-Link Commander (.*)\n").unwrap();
    if let Some(caps) = re.captures(&out.stdout) {
        Ok(String::from_utf8_lossy(&caps[1]).into_owned())
    } else {
        Err(Error::Regex)
    }
}

pub fn teensy_version() -> Result<String, Error> {
    let out = Command::new("teensy_loader_cli")
        .arg("-v")
        .arg("--mcu")
        .arg("mkl26z64")
        .arg("dummy")
        .output()?;
    let re = Regex::new(r"(?m)Teensy Loader, Command Line, Version (.*)\n").unwrap();
    if let Some(caps) = re.captures(&out.stdout) {
        Ok(String::from_utf8_lossy(&caps[1]).into_owned())
    } else {
        Err(Error::Regex)
    }
}