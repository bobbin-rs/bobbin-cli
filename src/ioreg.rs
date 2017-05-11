use ::errors::*;

use device::UsbDevice;
use std::process::Command;
use std::io::Cursor;
use std::vec::Vec;
use plist::Plist;

// To list all serial devices:
// ioreg -c IOSerialBSDClient -r -t
// To list all media devices:
// ioreg -c IOMedia -r -t

impl<'a> From<&'a Plist> for UsbDevice {
    fn from(other: &Plist) -> Self {
        fn get_number(p: &Plist, key: &str) -> Option<i64> {
            if let Some(p_dict) = p.as_dictionary() {
                if let Some(v) = p_dict.get(key) {
                    v.as_integer()
                } else {
                    None
                }
            } else {
                None
            }            
        }
        fn get_string(p: &Plist, key: &str) -> String {
            if let Some(p_dict) = p.as_dictionary() {
                if let Some(v) = p_dict.get(key) {
                    if let Some(v_str) = v.as_string() {
                        String::from(v_str)
                    } else {
                        String::from("")
                    }
                } else {
                    String::from("")
                }
            } else {
                String::from("")
            }
        }
        UsbDevice {
            vendor_id: get_number(other, "idVendor").map(|v| v as u16).unwrap(),
            vendor_string: get_string(other, "USB Vendor Name"),
            product_id: get_number(other, "idProduct").map(|v| v as u16).unwrap(),
            product_string: get_string(other, "USB Product Name"),
            serial_number: get_string(other, "USB Serial Number"),
            location_id: get_number(other, "locationID"),
        }
    }
}


pub fn enumerate() -> Result<Vec<UsbDevice>> {
    let output = try!(Command::new("ioreg").arg("-p").arg("IOUSB").arg("-lxa").output());    
    let top = try!(Plist::read(Cursor::new(&output.stdout)));
    let mut items: Vec<UsbDevice> = Vec::new();
    visit(&top, &mut |p| {
        items.push(UsbDevice::from(p));
    });
    Ok(items)
}

fn visit<F: FnMut(&Plist)>(p: &Plist, mut f: &mut F) {
    if let Some(p_dict) = p.as_dictionary() {
        if p_dict.contains_key("idVendor") {
            f(p);
        }
    }    
    if let Some(p_children) = children(p) {
        for c in p_children.iter() {
            visit(c, f);
        }
    }
}

fn children(p: &Plist) -> Option<&Vec<Plist>> {
    if let Some(p_dict) = p.as_dictionary() {
        if let Some(p_dict_entry) = p_dict.get("IORegistryEntryChildren") {
            return p_dict_entry.as_array()
        }
    }
    None
}