use sha1;
use ioreg;
use clap::ArgMatches;
use Result;

#[derive(Debug)]
pub struct UsbDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor_string: String,
    pub product_string: String,
    pub serial_number: String,    
    pub location_id: Option<i64>,
}

impl UsbDevice {
    pub fn hash(&self) -> String {
        let mut h = sha1::Sha1::new();
        h.update(self.vendor_string.as_bytes());
        h.update(self.product_string.as_bytes());
        h.update(self.serial_number.as_bytes());
        h.digest().to_string()
    }
}

pub trait Device {
    fn usb(&self) -> &UsbDevice;
    fn hash(&self) -> String {
        self.usb().hash()
    }
    fn is_unknown(&self) -> bool { false }
    fn device_type(&self) -> Option<&str>;
    fn serial_path(&self) -> Option<String> { None }
}

pub struct UnknownDevice {
    usb: UsbDevice,
}

impl Device for UnknownDevice {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }
    fn is_unknown(&self) -> bool { 
       true
    }
    fn device_type(&self) -> Option<&str> {
        None
    }
}

pub struct JLinkDevice {
    usb: UsbDevice,
}

impl Device for JLinkDevice {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("JLink")
    }

    fn serial_path(&self) -> Option<String> {
        None
    }    
}

pub struct StLinkV2Device {
    usb: UsbDevice,
}

impl Device for StLinkV2Device {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("STLinkV2")
    }

    fn serial_path(&self) -> Option<String> {
        None
    }    
}

pub struct StLinkV21Device {
    usb: UsbDevice,
}

impl Device for StLinkV21Device {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("STLinkV21")
    }

    fn serial_path(&self) -> Option<String> {
        None
    }    
}

pub struct DeviceFilter {
    all: bool,
    device: Option<String>,
}

impl<'a> From<&'a ArgMatches<'a>> for DeviceFilter {
    fn from(other: &ArgMatches) -> DeviceFilter {
        DeviceFilter {
            all: other.is_present("all"),
            device: other.value_of("device").map(String::from)
        }
    }
}

pub fn lookup(usb: UsbDevice) -> Box<Device> {
    match (usb.vendor_id, usb.product_id) {
        (0x0483, 0x3748) => Box::new(StLinkV2Device { usb: usb }),
        (0x0483, 0x374b) => Box::new(StLinkV21Device { usb: usb }),
        (0x1366, 0x0101) => Box::new(JLinkDevice { usb: usb }),
        (0x1366, 0x0105) => Box::new(JLinkDevice { usb: usb }),
        _ => Box::new(UnknownDevice { usb: usb })
    }
}


pub fn enumerate() -> Result<Vec<Box<Device>>> {
    Ok(ioreg::enumerate()?.into_iter().map(lookup).collect())
}

pub fn search(filter: &DeviceFilter) -> Result<Vec<Box<Device>>> {
    Ok(enumerate()?.into_iter().filter(|d| {
        if !filter.all {
            if d.is_unknown() {
                return false
            }
        }

        if let Some(ref device) = filter.device {
            if !d.hash().starts_with(device) {
                return false
            }
        }


        true
    }).collect())
}