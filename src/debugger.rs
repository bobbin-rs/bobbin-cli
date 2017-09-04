use clap::ArgMatches;
use config::Config;
use printer::Printer;
use device::Device;

use std::process::Command;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::env;

use tempfile;
use Result;

use blackmagic::blackmagic_scan;

pub fn debugger(debugger_type: &str) -> Option<Box<Control>> {
    match debugger_type.to_lowercase().as_ref() {
        "openocd" => Some(Box::new(OpenOcdDebugger {})),
        "jlink" => Some(Box::new(JLinkDebugger {})),
        "blackmagic" => Some(Box::new(BlackMagicDebugger {})),
        _ => None,
    }
}

pub trait Control {    
    fn halt(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()>;
    fn resume(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()>;
    fn reset(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()>;
    fn reset_halt(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()>;
    fn reset_run(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()>;
    fn reset_init(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()>;
}

pub struct OpenOcdDebugger {}

impl OpenOcdDebugger {
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

    pub fn command(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
        action: &str,
    ) -> Result<()> {
        let mut cmd = Command::new("openocd");
        if let Some(openocd_cfg) = self.find_config(device) {
            cmd.arg("--file").arg(openocd_cfg);
        } else {
            bail!("No openocd.cfg file was found.");
        }                
        cmd.arg("--command").arg(&device.openocd_serial().unwrap());
        cmd.arg("--command").arg("init");
        cmd.arg("--command").arg(action);
        cmd.arg("--command").arg("exit");

        out.verbose("openocd", &format!("{:?}", cmd))?;

        if out.is_verbose() {
            cmd.status()?;
        } else {
            cmd.output()?;
        }
        Ok(())
    }

    pub fn run(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        use std::os::unix::process::CommandExt;

        let mut cmd = Command::new("openocd");
        if let Some(openocd_cfg) = self.find_config(device) {
            cmd.arg("--file").arg(openocd_cfg);
        } else {
            bail!("No openocd.cfg file was found.");
        }                
        cmd.arg("--command").arg(&device.openocd_serial().unwrap());
        cmd.exec();
        unreachable!();
    }    
}


impl Control for OpenOcdDebugger {
    fn halt(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        out.info("Halting", &format!("Halting Device"))?;
        self.command(cfg, args, cmd_args, out, device, "halt")
    }
    fn resume(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        out.info("Resuming", &format!("Resuming Device"))?;
        self.command(cfg, args, cmd_args, out, device, "resume")
    }
    fn reset(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        out.info("Resetting", &format!("Resetting Device"))?;
        self.command(cfg, args, cmd_args, out, device, "reset")
    }
    fn reset_halt(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        out.info(
            "Resetting",
            &format!("Resetting and Halting Device"),
        )?;
        self.command(cfg, args, cmd_args, out, device, "reset halt")
    }
    fn reset_run(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        out.info(
            "Resetting",
            &format!("Resetting and Running Device"),
        )?;
        self.command(cfg, args, cmd_args, out, device, "reset run")
    }
    fn reset_init(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        out.info(
            "Resetting",
            &format!("Resetting and Initializing Device"),
        )?;
        self.command(cfg, args, cmd_args, out, device, "reset init")
    }
}


pub struct JLinkDebugger {}
impl JLinkDebugger {
    fn command(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
        action: &str,
    ) -> Result<()> {

        let jlink_dev = if let Some(jlink_dev) = cmd_args.value_of("jlink-device") {
            jlink_dev
        } else if let Some(jlink_dev) = cfg.jlink_device() {
            jlink_dev
        } else {
            bail!("JLink Loader requires that --jlink-device is specified");
        };

        // Generate Script File
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        try!(writeln!(tmpfile, "{}", action));
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
        cmd.arg("-ExitOnError").arg("1");
        cmd.arg("-CommanderScript").arg(tmpfile.path());

        out.verbose("jlink", &format!("{:?}", cmd))?;

        if out.is_verbose() {
            cmd.status()?;
        } else {
            cmd.output()?;
        }
        Ok(())
    }
}

impl Control for JLinkDebugger {
    fn halt(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        //self.command(cfg, args, cmd_args, out, device, "halt")
        bail!("halt is not supported for this debugger")
    }
    fn resume(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        //self.command(cfg, args, cmd_args, out, device, "go")
        bail!("halt is not supported for this debugger")
    }
    fn reset(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        self.command(cfg, args, cmd_args, out, device, "r")
    }
    fn reset_halt(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        //self.command(cfg, args, cmd_args, out, device, "r")
        bail!("reset halt is not supported for this debugger")
    }
    fn reset_run(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        self.command(cfg, args, cmd_args, out, device, "r")
    }
    fn reset_init(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        bail!("reset init is not supported for this debugger")
        //self.command(cfg, args, cmd_args, out, device, "r")
    }
}

pub struct BlackMagicDebugger {}
impl BlackMagicDebugger {
    fn command(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
        action: &str,
    ) -> Result<()> {
        let blackmagic_scan = blackmagic_scan(cfg, args, cmd_args)?;

        let mut cmd = Command::new("arm-none-eabi-gdb");
        if let Some(gdb_path) = device.gdb_path() {
            cmd.arg("-ex").arg("set confirm off");
            cmd.arg("-ex").arg(format!("target extended-remote {}", gdb_path));
            // These commands are BlackMagic Probe Specific
            cmd.arg("-ex").arg(blackmagic_scan);
            cmd.arg("-ex").arg("attach 1");
        }
        cmd.arg("-ex").arg(action);
        cmd.arg("-ex").arg("quit");
        out.verbose("blackmagic", &format!("{:?}", cmd))?;

        if out.is_verbose() {
            cmd.status()?;
        } else {
            cmd.output()?;
        }
        Ok(())
    }
}

impl Control for BlackMagicDebugger {
    fn halt(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        // self.command(cfg, args, cmd_args, out, device, "interrupt")
        bail!("halt is not supported for this debugger")
    }
    fn resume(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        // self.command(cfg, args, cmd_args, out, device, "c&")
        bail!("resume is not supported for this debugger")
    }
    fn reset(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        self.command(cfg, args, cmd_args, out, device, "kill")
    }
    fn reset_halt(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        // self.command(cfg, args, cmd_args, out, device, "start")
        bail!("reset halt is not supported for this debugger")
    }
    fn reset_run(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        self.command(cfg, args, cmd_args, out, device, "kill")
    }
    fn reset_init(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
    ) -> Result<()> {
        // self.command(cfg, args, cmd_args, out, device, "")
        bail!("reset init is not supported for this debugger")
    }
}
