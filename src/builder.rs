use std::path::PathBuf;
use std::process::Command;
use config::Config;
use clap::ArgMatches;
use printer::Printer;
use Result;

pub fn build_path(cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches) -> Result<PathBuf> {
    if let Some(dst) = cmd_args.value_of("binary") {
        return Ok(PathBuf::from(dst));
    }

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
        let mut tmp = dst.clone();
        tmp.push(name);
        if tmp.exists() {            
            dst = tmp;
        } else {
            dst.push("main");
        }
    } else {
        bail!("No binary specified");
    };
    Ok(dst)
}

pub fn build(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<Option<PathBuf>> {
    if cmd_args.is_present("no-build") {
        Ok(Some(build_path(cfg, args, cmd_args)?))
    } else if cmd_args.is_present("make") {
        build_make(cfg, args, cmd_args, out)
    } else {
        build_xargo(cfg, args, cmd_args, out)
    }
}
pub fn build_make(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<Option<PathBuf>> {
    let mut cmd = Command::new("make");
    out.verbose("make", &format!("{:?}", cmd))?;
    for arg in cmd_args.values_of_os("make").unwrap().into_iter() {
        cmd.arg(arg);
    }

    if !cmd.status()?.success() {
        bail!("make failed");
    }
    let dst = build_path(cfg, args, cmd_args)?;
    Ok(Some(dst))
}
pub fn build_xargo(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<Option<PathBuf>> {
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
    out.verbose("xargo", &format!("{:?}", cmd))?;
    if !cmd.status()?.success() {
        bail!("xargo build failed");
    }
    let dst = build_path(cfg, args, cmd_args)?;
    if dst.is_file() {
        let mut cmd = Command::new("arm-none-eabi-size");
        out.verbose("size", &format!("{:?}", cmd))?;
        cmd.arg(&dst);
        if !cmd.status()?.success() {
            bail!("arm-none-eabi-size failed");
        }
        Ok(Some(dst))
    } else {
        Ok(None)
    }
}
