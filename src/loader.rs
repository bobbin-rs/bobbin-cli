use clap::ArgMatches;
use std::io::Write;
use std::process::{Command, ExitStatus};
use config::Config;
use printer::Printer;
use device::Device;
use std::path::{Path, PathBuf};
use tempfile;

use Result;

pub trait Load {
    fn load(&self, cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches, out: &mut Printer, device: &Device, target: &Path) -> Result<()>;
}

pub fn loader(loader_type: &str) -> Option<Box<Load>> {
    match loader_type.to_lowercase().as_ref() {
        "openocd" => Some(Box::new(OpenOcdLoader {})),
        "jlink" => Some(Box::new(JLinkLoader {})),
        "bossa" => Some(Box::new(BossaLoader {})),
        "teensy" => Some(Box::new(TeensyLoader {})),
        _ => None
    }
}

pub struct OpenOcdLoader {}

impl Load for OpenOcdLoader {
    fn load(&self, cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches, out: &mut Printer, device: &Device, target: &Path) -> Result<()> {
        unimplemented!()
    }
}

pub struct JLinkLoader {}

impl Load for JLinkLoader {
    fn load(&self, cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches, out: &mut Printer, device: &Device, target: &Path) -> Result<()> {
        let mut dst = PathBuf::from(target);
        dst.set_extension("hex");
        objcopy("ihex", target, &dst)?;

        // Generate Script File

        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        try!(writeln!(tmpfile, "r"));
        try!(writeln!(tmpfile, "h"));
        try!(writeln!(tmpfile, "loadfile {}", dst.display()));
        if cmd_args.is_present("run") {
            try!(writeln!(tmpfile, "g"));
        }
        try!(writeln!(tmpfile, "exit"));

        // Execute Command

        let mut cmd = Command::new("JLinkExe");        
        cmd.arg("-device").arg("S32K144"); // Allow setting these parameters from the command line and config 
        cmd.arg("-if").arg("SWD");
        cmd.arg("-autoconnect").arg("1");
        cmd.arg("-speed").arg("4000");
        cmd.arg("-SelectEmuBySN").arg(device.usb().serial_number.clone());
        cmd.arg("-CommanderScript").arg(tmpfile.path());
        cmd.arg("-ExitOnError").arg("1");

        out.verbose("jlink",&format!("{:?}", cmd))?;

        out.info("Loading",&format!("{}", dst.display()))?;
        if out.is_verbose() {
            cmd.spawn()?.wait()?;
        } else {
            cmd.output()?;
        }
        out.info("Complete",&format!("Successfully flashed device"))?;
        Ok(())
    }
}

pub struct BossaLoader {}

impl Load for BossaLoader {
    fn load(&self, cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches, out: &mut Printer, device: &Device, target: &Path) -> Result<()> {
        unimplemented!()
    }
}

pub struct TeensyLoader {}

impl Load for TeensyLoader {
    fn load(&self, cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches, out: &mut Printer, device: &Device, target: &Path) -> Result<()> {
        unimplemented!()
    }
}

pub fn objcopy(output: &str, src: &Path, dst: &Path) -> Result<ExitStatus> {
    let mut cmd = Command::new("arm-none-eabi-objcopy");
    cmd.arg("-O").arg(output).arg(src).arg(dst);    
    Ok(cmd.spawn()?.wait()?)
}