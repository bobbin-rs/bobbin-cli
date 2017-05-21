use ::errors::*;

use std::io::{self, Read};
use std::path::Path;
use std::fs::{self, File};
use std::vec::Vec;
use device::UsbDevice;


pub fn enumerate() -> Result<Vec<UsbDevice>> {
    let mut items: Vec<UsbDevice> = Vec::new();
    let root = Path::new("/sys/bus/usb/devices");

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        //println!("path: {:?}", path);

        let id_vendor = if let Ok(id_vendor) = read_u16(&path.join("idVendor")) {
            id_vendor
        } else {
            continue
        };
        let id_product = if let Ok(id_product) = read_u16(&path.join("idProduct")) {
            id_product
        } else {
            continue
        };
        items.push(UsbDevice {
            vendor_id: id_vendor,
            vendor_string: read_file(&path.join("manufacturer")).unwrap_or(String::new()),
            product_id: id_product,
            product_string: read_file(&path.join("product")).unwrap_or(String::new()),
            serial_number: read_file(&path.join("serial")).unwrap_or(String::new()),
            location_id: None,
        });
    }
    
    Ok(items)
}

fn read_u16(path: &Path) -> Result<u16> {
    let s = read_file(path)?;
    Ok(u16::from_str_radix(&s[..4], 16)?)
}

fn read_file(path: &Path) -> Result<String> {
    let mut f = File::open(path)?;
    let mut s = Vec::new();
    f.read_to_end(&mut s)?;
    Ok(String::from(String::from_utf8_lossy(&s).split('\n').next().unwrap_or("")))
}    
