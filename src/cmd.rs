use Result;
use clap::ArgMatches;
use config::Config;
use printer::Printer;
use std::io::Write;
use device;
use builder;
use loader;
use debugger;

pub fn list(cfg: &Config, args: &ArgMatches, out: &mut Printer) -> Result<()> {
    let filter = device::DeviceFilter::from(args);
    let devices = device::search(&filter);

    writeln!(out, "{:08} {:04}:{:04} {:24} {:32} {:24}", 
        "ID",
        "VID", 
        "PID", 
        "Vendor", 
        "Product",
        "Serial Number",
        )?;        
    for d in devices?.iter() {
        let u = d.usb();
        write!(out, "{:08} {:04x}:{:04x} {:24} {:32} {:24}",
            &d.hash()[..8],
            u.vendor_id, 
            u.product_id, 
            u.vendor_string, 
            u.product_string,
            u.serial_number,
            )?;
        writeln!(out, "")?;
    }
    Ok(())
}


pub fn info(cfg: &Config, args: &ArgMatches, out: &mut Printer) -> Result<()> {
    let filter = device::DeviceFilter::from(args);
    let devices = device::search(&filter)?;

    for d in devices.iter() {
        let u = d.usb();        
        writeln!(out, "{:16} {}", "ID", d.hash())?;
        writeln!(out, "{:16} {:04x}", "Vendor ID", u.vendor_id)?;
        writeln!(out, "{:16} {:04x}", "Product ID", u.product_id)?;
        writeln!(out, "{:16} {}", "Vendor", u.vendor_string)?;
        writeln!(out, "{:16} {}", "Product", u.product_string)?;
        writeln!(out, "{:16} {}", "Serial Number", u.serial_number)?;
        writeln!(out, "{:16} {}", "Type", d.device_type().unwrap_or("Unknown"))?;
        if let Some(loader_type) = d.loader_type() {
            writeln!(out, "{:16} {}", "Loader Type", loader_type)?;
        }
        if let Some(debugger_type) = d.debugger_type() {
            writeln!(out, "{:16} {}", "Debugger Type", debugger_type)?;
        }        
        if let Some(cdc_path) = d.cdc_path() {
            writeln!(out, "{:16} {}", "CDC Device", cdc_path)?;
        }
        if let Some(msd_path) = d.msd_path() {
            writeln!(out, "{:16} {}", "MSD Device", msd_path.display())?;
        }
        if let Some(openocd_serial) = d.openocd_serial() {
            writeln!(out, "{:16} {}", "OpenOCD Serial", openocd_serial)?;
        }
        writeln!(out, "")?;
    }
    Ok(())
}

pub fn build(cfg: &Config, args: &ArgMatches, out: &mut Printer) -> Result<()> {
    let dst = builder::build(cfg, args, args.subcommand_matches("build").unwrap(), out)?;
    Ok(())
}

pub fn load(cfg: &Config, args: &ArgMatches, out: &mut Printer) -> Result<()> {
    let cmd_args = args.subcommand_matches("load").unwrap();
    let filter = device::DeviceFilter::from(args);
    let mut devices = device::search(&filter)?;

    let device = if devices.len() == 0 {
        bail!("No matching devices found.");
    } else if devices.len() > 1 {
        bail!("More than one device found ({})", devices.len());
    } else {
        devices.remove(0)
    };
    
    let ldr = if let Some(ldr) = device.loader_type() {
        out.verbose("loader",ldr)?;
        if let Some(ldr) = loader::loader(ldr) {
            ldr
        } else {
            bail!("Unknown loader type: {}", ldr);
        }
    } else {
        bail!("Selected device has no associated loader");
    };

    let dst = if let Some(dst) = builder::build(cfg, args, cmd_args, out)? {
        dst
    } else {
        bail!("No build output available to load");
    };
    out.verbose("target", &format!("{}", dst.display()))?;
    
    ldr.load(cfg, args, cmd_args, out, device.as_ref(), dst.as_path())?;

    Ok(())
}

pub fn control(cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches, out: &mut Printer) -> Result<()> {    
    let filter = device::DeviceFilter::from(args);
    let mut devices = device::search(&filter)?;

    let device = if devices.len() == 0 {
        bail!("No matching devices found.");
    } else if devices.len() > 1 {
        bail!("More than one device found ({})", devices.len());
    } else {
        devices.remove(0)
    };

    let dbg = if let Some(dbg) = device.debugger_type() {
        out.verbose("debugger",dbg)?;
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