use clap::ArgMatches;
use std::io::Write;
use std::process::{Command, ExitStatus};
use std::env;
use config::Config;
use printer::Printer;
use device::Device;
use std::path::{Path, PathBuf};

use tempfile;

use Result;

use blackmagic::blackmagic_scan;

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
        "dfu-util" => Some(Box::new(DfuUtilLoader {})),
        "blackmagic" => Some(Box::new(BlackMagicLoader {})),
        _ => None,
    }
}

pub struct OpenOcdLoader {}

impl OpenOcdLoader {
    fn find_config(&self, device: &Device) -> Option<PathBuf> {
        let device_id = &device.hash()[..8];            

        // Look in current path
        let bobbin_openocd = Path::new("openocd.cfg");
        if bobbin_openocd.exists() {
            return Some(bobbin_openocd.into())
        }

        // Look in .bobbin/<device-id>/
        let mut bobbin_openocd = PathBuf::from(".bobbin");
        bobbin_openocd.push(device_id);
        bobbin_openocd.push("openocd.cfg");
        if bobbin_openocd.exists() {
            return Some(bobbin_openocd.into())
        }

        // Look in ~/.bobbin/<device-id>/
        if let Some(home) = env::home_dir() {
            let mut bobbin_openocd = home;
            bobbin_openocd.push(".bobbin");
            bobbin_openocd.push(device_id);
            bobbin_openocd.push("openocd.cfg");
            if bobbin_openocd.exists() {
                return Some(bobbin_openocd)
            }        
        }

        None
    }
}

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
        if let Some(openocd_cfg) = self.find_config(device) {
            cmd.arg("--file").arg(openocd_cfg);
        } else {
            bail!("No openocd.cfg file was found.");
        }
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

        let jlink_dev = if let Some(jlink_dev) = cfg.jlink_device(cmd_args) {
            jlink_dev
        } else {
            bail!("JLink Loader requires that --jlink-device is specified");
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
            .arg(device.bossa_path().unwrap());

        if let Some(offset) = cfg.offset(cmd_args) {
            cmd.arg("-o").arg(offset);
        }

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

        let teensy_mcu = if let Some(teensy_mcu) = cfg.teensy_mcu(cmd_args) {
            teensy_mcu
        } else {
            bail!("Teensy Loader requires that --teensy-mcu is specified. Try 'teensy_loader_cli --list-mcus'.");
        };

        out.info("Loading", &format!("{}", dst.display()))?;

        let mut cmd = Command::new("teensy_loader_cli");
        cmd.arg(&format!("-mmcu={}", teensy_mcu));
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

pub struct DfuUtilLoader {}

impl Load for DfuUtilLoader {
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

        let mut cmd = Command::new("dfu-util");
        cmd.arg("-d").arg(format!("{:04x}:{:04x}", device.usb().vendor_id, device.usb().product_id));
        cmd.arg("-a").arg("0");
        cmd.arg("-s").arg("0x08000000");
        cmd.arg("-D").arg(dst);

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

pub struct BlackMagicLoader {}
impl Load for BlackMagicLoader {
    fn load(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
        target: &Path,
    ) -> Result<()> {

        let blackmagic_scan = blackmagic_scan(cfg, args, cmd_args)?;

        out.info("Loading", &format!("{}", target.display()))?;

        let mut cmd = Command::new("arm-none-eabi-gdb");
        if let Some(gdb_path) = device.gdb_path() {
            cmd.arg("-ex").arg("set confirm off");
            cmd.arg("-ex").arg(format!("target extended-remote {}", gdb_path));
            // These commands are BlackMagic Probe Specific
            cmd.arg("-ex").arg(blackmagic_scan);
            cmd.arg("-ex").arg("attach 1");
            cmd.arg("-ex").arg("load");
            cmd.arg("-ex").arg("kill");
            cmd.arg("-ex").arg("quit 0");
        }        
        cmd.arg(target);
        if out.is_verbose() {
            println!("{:?}", cmd);
        }
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

// pub struct RemoteLoader {}
// impl RemoteLoader {
//     pub fn load_remote(
//         &self,
//         cfg: &Config,
//         args: &ArgMatches,
//         cmd_args: &ArgMatches,
//         out: &mut Printer,
//         target: &Path,
//         host: &str,
//     ) -> Result<()> {
//         use std::process::*;
//         use std::os::unix::io::*;
//         use std::os::unix::process::CommandExt;

//         let bin = File::open(target)?;
//         let bin_fd = bin.into_raw_fd();

//         let mut cmd = Command::new("ssh");
//         cmd.arg(remote_host);
//         cmd.arg(".cargo/bin/bobbin");
//         if args.is_present("verbose") {
//             cmd.arg("-v");
//         }
//         if let Some(remote_device) = cmd_args.value_of("remote-device") {
//             cmd.arg("--device").arg(remote_device);
//         }
//         let subcmd = if args.is_present("load") {
//             "load"
//         } else if args.is_present("run") {
//             "run"
//         } else if args.is_present("test") {
//             "test"
//         } else {
//             bail!("Only load, run and test are supported for remote hosts")
//         };
//         cmd.arg(subcmd);
//         cmd.arg("--stdin");
//         if out.is_verbose() {
//             println!("{:?}", cmd);
//         }

//         let new_stdio = unsafe { Stdio::from_raw_fd(bin_fd) };

//         cmd
//             .stdin(new_stdio)
//             .exec();
//         unreachable!()
//     }
// }