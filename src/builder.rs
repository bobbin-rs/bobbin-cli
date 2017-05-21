use std::path::{PathBuf};
use std::process::{Command};
use config::Config;
use clap::ArgMatches;
use printer::Printer;
use Result;

pub fn build_path(cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches) -> Result<PathBuf> {
    let mut dst = PathBuf::from("target");

    if let Some(t) = cmd_args.value_of("target") {
        dst.push(t)
    } else if let Some(t) = cfg.default_target() {
        dst.push(t)
    } else {
        bail!("No target specified");
    }

    if cmd_args.is_present("release") {
        dst.push("release")
    } else {
        dst.push("debug")
    }

    if let Some(name) = cmd_args.value_of("example") {
        dst.push("examples");
        dst.push(name);
    } else if let Some(name) = cmd_args.value_of("bin") {
        dst.push(name);
    } else if let Some(name) = cfg.default_binary() {
        dst.push(name);
    } else {
        bail!("No binary specified");
    };    
    Ok(dst)
}

pub fn build(cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches, out: &mut Printer) -> Result<Option<PathBuf>> {
    let dst = build_path(cfg, args, cmd_args)?;
    let mut cmd = Command::new("xargo");
    cmd.arg("build");

    if let Some(name) = cmd_args.value_of("example") {
        cmd.arg("--example").arg(name);
    }
    if let Some(name) = cmd_args.value_of("bin") {
        cmd.arg("--bin").arg(name);
    }
    if cmd_args.is_present("release") {
        cmd.arg("--release");
    }
    if let Some(value) = cmd_args.value_of("features") {
        cmd.arg("--features").arg(value);
    }        
    if let Some(value) = cmd_args.value_of("target") {
        cmd.arg("--target").arg(value);
    }    
    out.verbose("xargo",&format!("{:?}", cmd))?;
    if !cmd.status()?.success() {
        bail!("Build failed");
    }
    if dst.is_file() {
        let mut cmd = Command::new("arm-none-eabi-size");
        out.verbose("size",&format!("{:?}", cmd))?;
        cmd.arg(&dst);
        cmd.spawn()?.wait()?;

        Ok(Some(dst))
    } else {
        Ok(None)
    }    
}
