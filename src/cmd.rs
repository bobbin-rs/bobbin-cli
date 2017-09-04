use Result;
use clap::ArgMatches;
use config::Config;
use printer::Printer;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::*;
use std::os::unix::io::*;
use std::os::unix::process::CommandExt;
use std::fs::File;

use device;
use builder;
use loader;
use debugger;
use console;
use check;
use tempfile;

pub fn check(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    writeln!(out, "    Bobbin {}", crate_version!())?;
    writeln!(out, "      Rust {}", check::rust_version().unwrap_or(String::from("Not Found")))?;
    writeln!(out, "     Cargo {}", check::cargo_version().unwrap_or(String::from("Not Found")))?;
    writeln!(out, "     Xargo {}", check::xargo_version().unwrap_or(String::from("Not Found")))?;
    writeln!(out, "       GCC {}", check::gcc_version().unwrap_or(String::from("Not Found")))?;
    writeln!(out, "   OpenOCD {}", check::openocd_version().unwrap_or(String::from("Not Found")))?;
    writeln!(out, "     JLink {}", check::jlink_version().unwrap_or(String::from("Not Found")))?;
    writeln!(out, "     Bossa {}", check::bossac_version().unwrap_or(String::from("Not Found")))?;    
    writeln!(out, "    Teensy {}", check::teensy_version().unwrap_or(String::from("Not Found")))?;
    writeln!(out, "  dfu-util {}", check::dfu_util_version().unwrap_or(String::from("Not Found")))?;
    Ok(())
}

pub fn list(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {   
    if let Some(host) = args.value_of("host").or_else(|| cfg.filter_host()) {
        let mut cmd = Command::new("ssh");
        cmd.arg(host);
        cmd.arg(".cargo/bin/bobbin");
        if let Some(device) = args.value_of("device") {
            cmd.arg("--device").arg(device);
        }        
        if args.is_present("verbose") {
            cmd.arg("--verbose");
        }                
        cmd.arg("list");
        cmd.exec();
        unreachable!()
    }

    let filter = device::filter(cfg, args, cmd_args);
    let devices = device::search(&filter);

    writeln!(out, "{:08} {:08}  {:40} {:24}",
        "ID",
        " VID:PID",
        "Vendor / Product",
        "Serial Number",
        )?;
    for d in devices?.iter() {
        let u = d.usb();
        write!(out, "{:08} {:04x}:{:04x} {:40} {:24}",
            &d.hash()[..8],
            u.vendor_id,
            u.product_id,
            format!("{} / {}", u.vendor_string, u.product_string),
            u.serial_number,
            )?;
        writeln!(out, "")?;
    }
    Ok(())
}


pub fn info(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    if let Some(host) = args.value_of("host").or_else(|| cfg.filter_host()) {
        let mut cmd = Command::new("ssh");
        cmd.arg(host);
        cmd.arg(".cargo/bin/bobbin");
        if let Some(device) = args.value_of("device") {
            cmd.arg("--device").arg(device);
        }        
        if args.is_present("verbose") {
            cmd.arg("--verbose");
        }                
        cmd.arg("info");
        cmd.exec();
        unreachable!()
    }

    let filter = device::filter(cfg, args, cmd_args);
    let devices = device::search(&filter)?;

    for d in devices.iter() {
        let u = d.usb();
        writeln!(out, "{:16} {}", "ID", d.hash())?;
        writeln!(out, "{:16} {:04x}", "Vendor ID", u.vendor_id)?;
        writeln!(out, "{:16} {:04x}", "Product ID", u.product_id)?;
        writeln!(out, "{:16} {}", "Vendor", u.vendor_string)?;
        writeln!(out, "{:16} {}", "Product", u.product_string)?;
        writeln!(out, "{:16} {}", "Serial Number", u.serial_number)?;
        writeln!(
            out,
            "{:16} {}",
            "Type",
            d.device_type().unwrap_or("Unknown")
        )?;
        if let Some(loader_type) = d.loader_type() {
            writeln!(out, "{:16} {}", "Loader Type", loader_type)?;
        }
        if let Some(debugger_type) = d.debugger_type() {
            writeln!(out, "{:16} {}", "Debugger Type", debugger_type)?;
        }

        if let Some(bossa_path) = d.bossa_path() {
            writeln!(out, "{:16} {}", "Bossac Device", bossa_path)?;
        }
        if let Some(cdc_path) = d.cdc_path() {
            writeln!(out, "{:16} {}", "CDC Device", cdc_path)?;
        }
        if let Some(msd_path) = d.msd_path() {
            writeln!(out, "{:16} {}", "MSD Device", msd_path.display())?;
        }
        if let Some(gdb_path) = d.gdb_path() {
            writeln!(out, "{:16} {}", "GDB Device", gdb_path)?;
        }        
        if let Some(openocd_serial) = d.openocd_serial() {
            writeln!(out, "{:16} {}", "OpenOCD Serial", openocd_serial)?;
        }
        writeln!(out, "")?;
    }
    Ok(())
}

pub fn build(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    let dst = builder::build(cfg, args, args, out)?;
    Ok(())
}

pub fn load(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {

    let dst = if let Some(dst) = builder::build(cfg, args, cmd_args, out)? {
        dst
    } else {
        bail!("No build output available to load");
    };

    if let Some(host) = args.value_of("host").or_else(|| cfg.filter_host()) {
        let bin = File::open(dst)?;
        let mut cmd = Command::new("ssh");
        unsafe {
            cmd.stdin(Stdio::from_raw_fd(bin.into_raw_fd()));
        }
        cmd.arg(host);
        cmd.arg(".cargo/bin/bobbin");
        if let Some(device) = args.value_of("device") {
            cmd.arg("--device").arg(device);
        }        
        if args.is_present("verbose") {
            cmd.arg("--verbose");
        }          
        let subcmd = if args.is_present("load") {
            "load"
        } else if args.is_present("run") {
            "run"
        } else if args.is_present("test") {
            "test"
        } else {
            bail!("Only load, run and test are supported for remote hosts")
        };
        cmd.arg(subcmd);
        cmd.arg("--stdin");
        out.verbose("Remote", &format!("{:?}", cmd))?;

        cmd.exec();
        unreachable!()
    }

    let filter = device::filter(cfg, args, cmd_args);
    let mut devices = device::search(&filter)?;

    let device = if devices.len() == 0 {
        bail!("No matching devices found.");
    } else if devices.len() > 1 {
        bail!("More than one device found ({})", devices.len());
    } else {
        devices.remove(0)
    };

    let ldr = if let Some(ldr) = device.loader_type() {
        out.verbose("loader", ldr)?;
        if let Some(ldr) = loader::loader(ldr) {
            ldr
        } else {
            bail!("Unknown loader type: {}", ldr);
        }
    } else {
        bail!("Selected device has no associated loader");
    };

    let con = if !cmd_args.is_present("noconsole") && !cmd_args.is_present("itm") {
        if args.is_present("run") || args.is_present("test") {
            if let Some(cdc_path) = device.cdc_path() {
                let mut con = console::open(&cdc_path)?;
                con.clear()?;
                Some(con)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if dst == PathBuf::from("--") {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        out.verbose("stdin", &format!("Read {} bytes from stdin", buffer.len()))?;
        let mut tmpfile = tempfile::NamedTempFile::new()?;
        tmpfile.write(buffer.as_ref())?;
        tmpfile.flush()?;
        ldr.load(
            cfg,
            args,
            cmd_args,
            out,
            device.as_ref(),
            tmpfile.path()
        )?;
        out.verbose("stdin", "Removing temporary file.")?;
    } else {
        ldr.load(
            cfg,
            args,
            cmd_args,
            out,
            device.as_ref(),
            dst.as_path(),
        )?;
    }
    out.info("Loader", "Load Complete")?;

    if cmd_args.is_present("itm") {
        if device.can_trace_itm() {
            out.info("ITM", "Starting ITM Trace")?;
            let target_clk = if let Some(v) = cmd_args.value_of("itm-target-clock") {
                v.parse::<u32>()?
            } else {
                if let Some(v) = cfg.itm_target_clock() {
                    v
                } else {
                    bail!("itm-target-clock is required for ITM trace.")
                }
            };
            let trace_clk = 2_000_000;
            device.trace_itm(target_clk, trace_clk)?;

        } else {
            bail!("Currently selected device does not support ITM trace");
        }
    } else if let Some(mut con) = con {
        out.info("Console", "Opening Console")?;
        if args.is_present("test") {
            con.test(&args, &cmd_args)?;
        } else {
            con.view()?;
        }
    }

    Ok(())
}

pub fn control(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    if let Some(host) = args.value_of("host").or_else(|| cfg.filter_host()) {
        let mut cmd = Command::new("ssh");
        cmd.arg(host);
        cmd.arg(".cargo/bin/bobbin");
        if let Some(device) = args.value_of("device") {
            cmd.arg("--device").arg(device);
        }        
        if args.is_present("verbose") {
            cmd.arg("--verbose");
        }          
        let subcmd = if args.is_present("halt") {
            cmd.arg("halt");
        } else if args.is_present("resume") {
            cmd.arg("resume");
        } else if args.is_present("reset") {
            cmd.arg("reset");
            if cmd_args.is_present("run") {
                cmd.arg("--run");
            } else if cmd_args.is_present("halt") {
                cmd.arg("--halt");
            } else if cmd_args.is_present("init") {
                cmd.arg("--init");
            }        
        };
        out.verbose("Remote", &format!("{:?}", cmd))?;

        cmd.exec();
        unreachable!()
    }

    let filter = device::filter(cfg, args, cmd_args);
    let mut devices = device::search(&filter)?;

    let device = if devices.len() == 0 {
        bail!("No matching devices found.");
    } else if devices.len() > 1 {
        bail!("More than one device found ({})", devices.len());
    } else {
        devices.remove(0)
    };

    let dbg = if let Some(dbg) = device.debugger_type() {
        out.verbose("debugger", dbg)?;
        if let Some(dbg) = debugger::debugger(dbg) {
            dbg
        } else {
            bail!("Unknown debugger type: {}", dbg);
        }
    } else {
        bail!("Selected device has no associated loader");
    };

    if let Some(_) = args.subcommand_matches("halt") {
        dbg.halt(cfg, args, cmd_args, out, device.as_ref())?;
    } else if let Some(_) = args.subcommand_matches("resume") {
        dbg.resume(cfg, args, cmd_args, out, device.as_ref())?;
    } else if let Some(_) = args.subcommand_matches("reset") {
        if cmd_args.is_present("run") {
            dbg.reset_run(cfg, args, cmd_args, out, device.as_ref())?;
        } else if cmd_args.is_present("halt") {
            dbg.reset_halt(cfg, args, cmd_args, out, device.as_ref())?;
        } else if cmd_args.is_present("init") {
            dbg.reset_init(cfg, args, cmd_args, out, device.as_ref())?;
        } else {
            dbg.reset(cfg, args, cmd_args, out, device.as_ref())?;
        }
    }

    Ok(())
}

pub fn openocd(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    if let Some(host) = args.value_of("host").or_else(|| cfg.filter_host()) {
        let mut cmd = Command::new("ssh");
        cmd.arg("-L").arg("3333:localhost:3333");        
        cmd.arg(host);
        cmd.arg(".cargo/bin/bobbin");
        if let Some(device) = args.value_of("device") {
            cmd.arg("--device").arg(device);
        }        
        if args.is_present("verbose") {
            cmd.arg("--verbose");
        }                
        cmd.arg("openocd");
        println!("{:?}", cmd);
        cmd.exec();
        unreachable!()
    }

    let filter = device::filter(cfg, args, cmd_args);
    let mut devices = device::search(&filter)?;

    let device = if devices.len() == 0 {
        bail!("No matching devices found.");
    } else if devices.len() > 1 {
        bail!("More than one device found ({})", devices.len());
    } else {
        devices.remove(0)
    };

    let dbg = debugger::OpenOcdDebugger {};
    dbg.run(cfg, args, cmd_args, out, device.as_ref())?;
    unreachable!()
}


pub fn jlink(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    if let Some(host) = args.value_of("host").or_else(|| cfg.filter_host()) {
        let mut cmd = Command::new("ssh");
        cmd.arg("-L").arg("3333:localhost:3333");
        cmd.arg(host);
        cmd.arg(".cargo/bin/bobbin");
        if let Some(device) = args.value_of("device") {
            cmd.arg("--device").arg(device);
        }        
        if args.is_present("verbose") {
            cmd.arg("--verbose");
        }                
        cmd.arg("openocd");
        cmd.exec();
        unreachable!()
    }

    let filter = device::filter(cfg, args, cmd_args);
    let mut devices = device::search(&filter)?;

    let device = if devices.len() == 0 {
        bail!("No matching devices found.");
    } else if devices.len() > 1 {
        bail!("More than one device found ({})", devices.len());
    } else {
        devices.remove(0)
    };

    let jlink_dev = if let Some(jlink_dev) = cmd_args.value_of("jlink-device") {
        jlink_dev
    } else if let Some(jlink_dev) = cfg.jlink_device() {
        jlink_dev
    } else {
        bail!("JLink Loader requires that --jlink-device is specified");
    }; 

    let mut cmd = Command::new("JLinkGDBServer");
    cmd.arg("-device").arg(jlink_dev);
    cmd.arg("-if").arg("SWD");
    cmd.arg("-speed").arg("4000");
    cmd.arg("-port").arg("3333");
    cmd.arg("-select").arg(
        format!("usb={}",device.usb().serial_number),
    );

    cmd.exec();

    let status = cmd.status()?;
    if !status.success() {
        bail!("openocd failed")
    }
    Ok(())
}


pub fn gdb(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    let dst = if let Some(dst) = builder::build(cfg, args, cmd_args, out)? {
        dst
    } else {
        bail!("No build output available for gdb");
    };

    if let Some(host) = args.value_of("host").or_else(|| cfg.filter_host()) {
        let mut cmd = Command::new("arm-none-eabi-gdb");
        cmd.arg("-ex").arg(format!("target extended-remote :3333"));
        cmd.arg(dst);
        cmd.exec();
        unreachable!()
    }

    let filter = device::filter(cfg, args, cmd_args);
    let mut devices = device::search(&filter)?;

    let device = if devices.len() == 0 {
        bail!("No matching devices found.");
    } else if devices.len() > 1 {
        bail!("More than one device found ({})", devices.len());
    } else {
        devices.remove(0)
    };

    let mut cmd = Command::new("arm-none-eabi-gdb");
    if let Some(gdb_path) = device.gdb_path() {
        cmd.arg("-ex").arg(format!("target extended-remote {}", gdb_path));
        // These commands are BlackMagic Probe Specific
        cmd.arg("-ex").arg("monitor swdp_scan");
        cmd.arg("-ex").arg("attach 1");
    }
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit()).arg(
        dst,
    );
    out.verbose("gdb", &format!("{:?}", cmd))?;

    cmd.exec();

    let status = cmd.status()?;
    if !status.success() {
        bail!("gdb failed")
    }
    Ok(())
}

pub fn console(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    if let Some(host) = args.value_of("host").or_else(|| cfg.filter_host()) {
        let mut cmd = Command::new("ssh");
        cmd.arg(host);
        cmd.arg(".cargo/bin/bobbin");
        if let Some(device) = args.value_of("device") {
            cmd.arg("--device").arg(device);
        }        
        if args.is_present("verbose") {
            cmd.arg("--verbose");
        }                
        cmd.arg("console");
        cmd.exec();
        unreachable!()
    }

    let filter = device::filter(cfg, args, cmd_args);
    let mut devices = device::search(&filter)?;

    let device = if devices.len() == 0 {
        bail!("No matching devices found.");
    } else if devices.len() > 1 {
        bail!("More than one device found ({})", devices.len());
    } else {
        devices.remove(0)
    };

    if let Some(cdc_path) = device.cdc_path() {
        let mut con = console::open(&cdc_path)?;
        con.view()?
    } else {
        bail!("No console found for device");
    }

    Ok(())
}

pub fn screen(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    if let Some(host) = args.value_of("host").or_else(|| cfg.filter_host()) {
        let mut cmd = Command::new("ssh");
        cmd.arg(host);
        cmd.arg(".cargo/bin/bobbin");
        if let Some(device) = args.value_of("device") {
            cmd.arg("--device").arg(device);
        }        
        if args.is_present("verbose") {
            cmd.arg("--verbose");
        }                
        cmd.arg("screen");
        cmd.exec();
        unreachable!()
    }

    let filter = device::filter(cfg, args, cmd_args);
    let mut devices = device::search(&filter)?;

    let device = if devices.len() == 0 {
        bail!("No matching devices found.");
    } else if devices.len() > 1 {
        bail!("More than one device found ({})", devices.len());
    } else {
        devices.remove(0)
    };

    let mut cmd = Command::new("screen");
    if let Some(cdc_path) = device.cdc_path() {
        cmd.arg(cdc_path);
    } else {
        bail!("No serial device path found");
    }
    cmd.arg("115200");
    cmd.exec();

    let status = cmd.status()?;
    if !status.success() {
        bail!("screen failed")
    }
    Ok(())
}

pub fn objdump(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    Ok(())
}

pub fn itm(
    cfg: &Config,
    args: &ArgMatches,
    cmd_args: &ArgMatches,
    out: &mut Printer,
) -> Result<()> {
    if let Some(host) = args.value_of("host").or_else(|| cfg.filter_host()) {
        let mut cmd = Command::new("ssh");
        cmd.arg(host);
        cmd.arg(".cargo/bin/bobbin");
        if let Some(device) = args.value_of("device") {
            cmd.arg("--device").arg(device);
        }        
        if args.is_present("verbose") {
            cmd.arg("--verbose");
        }                
        cmd.arg("itm");
        cmd.exec();
        unreachable!()
    }

    let filter = device::filter(cfg, args, cmd_args);
    let mut devices = device::search(&filter)?;

    let device = if devices.len() == 0 {
        bail!("No matching devices found.");
    } else if devices.len() > 1 {
        bail!("More than one device found ({})", devices.len());
    } else {
        devices.remove(0)
    };

    if device.can_trace_itm() {
        out.info("ITM", "Starting ITM Trace")?;
        let target_clk = if let Some(v) = cmd_args.value_of("itm-target-clock") {
            v.parse::<u32>()?
        } else {
            if let Some(v) = cfg.itm_target_clock() {
                v
            } else {
                bail!("itm-target-clock is required for ITM trace.")
            }
        };
        let trace_clk = 2_000_000;
        device.trace_itm(target_clk, trace_clk)?;
    } else {
        bail!("Currently selected device does not support ITM trace");
    }
    Ok(())
}
