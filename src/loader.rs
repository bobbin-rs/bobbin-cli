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
    fn load(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
        target: &Path,
    ) -> Result<()>;
}

pub fn loader(loader_type: &str) -> Option<Box<Load>> {
    match loader_type.to_lowercase().as_ref() {
        "openocd" => Some(Box::new(OpenOcdLoader {})),
        "jlink" => Some(Box::new(JLinkLoader {})),
        "bossa" => Some(Box::new(BossaLoader {})),
        "teensy" => Some(Box::new(TeensyLoader {})),
        _ => None,
    }
}

pub struct OpenOcdLoader {}

impl Load for OpenOcdLoader {
    fn load(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
        target: &Path,
    ) -> Result<()> {
        let mut cmd = Command::new("openocd");
        cmd.arg("--file").arg("openocd.cfg");
        cmd.arg("--command").arg(&device.openocd_serial().unwrap());
        cmd.arg("--command").arg("gdb_port disabled");
        cmd.arg("--command").arg("tcl_port disabled");
        cmd.arg("--command").arg("telnet_port disabled");

        if args.is_present("run") || args.is_present("test") {
            cmd.arg("--command").arg(&format!(
                "program {} reset exit",
                target.display()
            ));
        } else {
            cmd.arg("--command").arg(&format!(
                "program {} exit",
                target.display()
            ));
        }

        out.verbose("openocd", &format!("{:?}", cmd))?;

        out.info("Loading", &format!("{}", target.display()))?;
        let status = if out.is_verbose() {
            cmd.status()?
        } else {
            cmd.output()?.status
        };

        if status.success() {
            out.info(
                "Complete",
                &format!("Successfully flashed device"),
            )?;
        } else {
            bail!("Error flashing device");
        }
        Ok(())
    }
}

pub struct JLinkLoader {}

impl Load for JLinkLoader {
    fn load(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
        target: &Path,
    ) -> Result<()> {
        let mut dst = PathBuf::from(target);
        dst.set_extension("hex");
        objcopy("ihex", target, &dst)?;

        let jlink_dev = if let Some(default_loader) = cfg.default_loader() {
            if let Some(ldr_cfg) = default_loader.as_table() {
                if let Some(mcu) = ldr_cfg["jlink_device"].as_str() {
                    mcu
                } else {
                    bail!("JLink Loader requires that jlink_device is specified");
                }
            } else {
                bail!("JLink Loader requires that jlink_device is specified");
            }
        } else {
            bail!("JLink Loader requires that jlink_device is specified");
        };

        // Generate Script File

        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        try!(writeln!(tmpfile, "r"));
        try!(writeln!(tmpfile, "h"));
        try!(writeln!(tmpfile, "loadfile {}", dst.display()));
        if args.is_present("run") || args.is_present("test") {
            try!(writeln!(tmpfile, "g"));
        }
        try!(writeln!(tmpfile, "exit"));

        // Execute Command

        let mut cmd = Command::new("JLinkExe");
        cmd.arg("-device").arg(jlink_dev);
        cmd.arg("-if").arg("SWD");
        cmd.arg("-autoconnect").arg("1");
        cmd.arg("-speed").arg("4000");
        cmd.arg("-SelectEmuBySN").arg(
            device.usb().serial_number.clone(),
        );
        cmd.arg("-CommanderScript").arg(tmpfile.path());
        cmd.arg("-ExitOnError").arg("1");

        out.verbose("jlink", &format!("{:?}", cmd))?;

        out.info("Loading", &format!("{}", dst.display()))?;
        let status = if out.is_verbose() {
            cmd.status()?
        } else {
            cmd.output()?.status
        };

        if status.success() {
            out.info(
                "Complete",
                &format!("Successfully flashed device"),
            )?;
        } else {
            bail!("Error flashing device");
        }
        Ok(())
    }
}

pub struct BossaLoader {}

impl Load for BossaLoader {
    fn load(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
        target: &Path,
    ) -> Result<()> {
        let mut dst = PathBuf::from(target);
        dst.set_extension("bin");
        objcopy("binary", target, &dst)?;

        // Execute Command

        out.info("Loading", &format!("{}", dst.display()))?;

        let mut cmd = Command::new("bossac");
        cmd.arg("-eivRw")
            .arg("-p")
            .arg(device.bossa_path().unwrap())
            .arg(dst);

        let status = if out.is_verbose() {
            cmd.status()?
        } else {
            cmd.output()?.status
        };

        if status.success() {
            out.info(
                "Complete",
                &format!("Successfully flashed device"),
            )?;
        } else {
            bail!("Error flashing device");
        }
        Ok(())
    }
}

pub struct TeensyLoader {}

impl Load for TeensyLoader {
    fn load(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
        target: &Path,
    ) -> Result<()> {
        let mut dst = PathBuf::from(target);
        dst.set_extension("hex");
        objcopy("ihex", target, &dst)?;

        // Execute Command

        let mcu = if let Some(default_loader) = cfg.default_loader() {
            if let Some(ldr_cfg) = default_loader.as_table() {
                if let Some(mcu) = ldr_cfg["mcu"].as_str() {
                    mcu
                } else {
                    bail!(
                        "Teensy Loader requires that a MCU is specified in the [loader] section of .bobbin/config"
                    );
                }
            } else {
                bail!(
                    "Teensy Loader requires that a MCU is specified in the [loader] section of .bobbin/config"
                );
            }
        } else {
            bail!(
                "Teensy Loader requires that a MCU is specified in the [loader] section of .bobbin/config"
            );
        };


        //let mcu = "mk20dx256";

        out.info("Loading", &format!("{}", dst.display()))?;

        let mut cmd = Command::new("teensy_loader_cli");
        cmd.arg(&format!("--mcu={}", mcu));
        cmd.arg("-v");
        cmd.arg(dst);

        let status = if out.is_verbose() {
            cmd.status()?
        } else {
            cmd.output()?.status
        };

        if status.success() {
            out.info(
                "Complete",
                &format!("Successfully flashed device"),
            )?;
        } else {
            bail!("Error flashing device");
        }
        Ok(())
    }
}

pub fn objcopy(output: &str, src: &Path, dst: &Path) -> Result<ExitStatus> {
    let mut cmd = Command::new("arm-none-eabi-objcopy");
    cmd.arg("-O").arg(output).arg(src).arg(dst);
    Ok(cmd.status()?)
}
