use Result;
use clap::ArgMatches;
use config::Config;
use printer::Printer;
use std::io::Write;
use device::{self};

pub fn list(cfg: &Config, args: &ArgMatches, out: &mut Printer) -> Result<()> {
    let filter = device::DeviceFilter::from(args);
    let devices = device::search(&filter);

    writeln!(out, "{:08} {:04}:{:04} {:12} {:24} {:32} {:24}", 
        "ID",
        "VID", 
        "PID", 
        "Type",
        "Vendor", 
        "Product",
        "Serial Number",
        )?;        
    for d in devices?.iter() {
        let u = d.usb();
        write!(out, "{:08} {:04x}:{:04x} {:12} {:24} {:32} {:24}",
            &d.hash()[..8],
            u.vendor_id, 
            u.product_id, 
            d.device_type().unwrap_or(""),
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
    let devices = device::search(&filter);

    for d in devices?.iter() {
        let u = d.usb();        
        writeln!(out, "{:16} {}", "ID", d.hash())?;
        writeln!(out, "{:16} {}", "Type", d.device_type().unwrap_or("Unknown"))?;
        writeln!(out, "{:16} {:04x}", "Vendor ID", u.vendor_id)?;
        writeln!(out, "{:16} {:04x}", "Product ID", u.product_id)?;
        writeln!(out, "{:16} {}", "Vendor", u.vendor_string)?;
        writeln!(out, "{:16} {}", "Product", u.product_string)?;
        writeln!(out, "{:16} {}", "Serial Number", u.serial_number)?;
        if let Some(serial_path) = d.serial_path() {
            writeln!(out, "{:16} {}", "Serial Device", serial_path)?;
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