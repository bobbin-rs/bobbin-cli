use clap::ArgMatches;
use config::Config;
use printer::Printer;
use device::Device;

use std::process::Command;
use std::io::Write;
use std::path::Path;

use tempfile;
use Result;

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
    fn command(
        &self,
        cfg: &Config,
        args: &ArgMatches,
        cmd_args: &ArgMatches,
        out: &mut Printer,
        device: &Device,
        action: &str,
    ) -> Result<()> {
        if !Path::new("openocd.cfg").exists() {
            bail!("No openocd.cfg found in current directory");
        }

        let mut cmd = Command::new("openocd");
        cmd.arg("--file").arg("openocd.cfg");
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
        // Generate Script File
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        try!(writeln!(tmpfile, "{}", action));
        try!(writeln!(tmpfile, "exit"));

        // Execute Command

        let mut cmd = Command::new("JLinkExe");
        // Allow setting these parameters from the command line and config
        cmd.arg("-device").arg("S32K144");
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
        let mut cmd = Command::new("arm-none-eabi-gdb");
        if let Some(gdb_path) = device.gdb_path() {
            cmd.arg("-ex").arg("set confirm off");
            cmd.arg("-ex").arg(format!("target extended-remote {}", gdb_path));
            // These commands are BlackMagic Probe Specific
            cmd.arg("-ex").arg("monitor jtag_scan");
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
