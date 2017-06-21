use sha1;
#[cfg(target_os="macos")]
use ioreg;
#[cfg(target_os="linux")]
use sysfs;
use clap::ArgMatches;
use std::path::PathBuf;
use std::fs;
use std::io::Read;
use std::fmt::Write;
use config::Config;
use stlink;
use Result;

#[derive(Debug)]
pub struct UsbDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor_string: String,
    pub product_string: String,
    pub serial_number: String,
    pub location_id: Option<i64>,
    pub path: Option<PathBuf>,
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
    fn is_unknown(&self) -> bool { self.device_type().is_none() }
    fn device_type(&self) -> Option<&str> { None }
    fn loader_type(&self) -> Option<&str> { None }
    fn debugger_type(&self) -> Option<&str> { None }
    fn cdc_path(&self) -> Option<String> { None }
    fn msd_path(&self) -> Option<PathBuf> { None }
    fn bossa_path(&self) -> Option<String> { None }
    
    fn jlink_supported(&self) -> bool {
        self.device_type() == Some("JLink")
    }

    fn openocd_supported(&self) -> bool {
        self.openocd_serial().is_some()
    }
    fn openocd_serial(&self) -> Option<String> { None }

    fn can_trace_itm(&self) -> bool { false }
    fn trace_itm(&self) -> Result<()> { unimplemented!() }
}

pub struct UnknownDevice {
    usb: UsbDevice,
}

impl Device for UnknownDevice {
    fn usb(&self) -> &UsbDevice {
        &self.usb
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

    fn loader_type(&self) -> Option<&str> {
        Some("JLink")
    }

    fn debugger_type(&self) -> Option<&str> {
        Some("JLink")
    }

    #[cfg(target_os="macos")]
    fn cdc_path(&self) -> Option<String> {
        Some(format!("/dev/cu.usbmodem{}{}", 
            format!("{:x}", self.usb.location_id.unwrap_or(0)).replace("0",""),
            1,
        ))
    }

    #[cfg(target_os="linux")]
    fn cdc_path(&self) -> Option<String> {
        if let Some(ref path) = self.usb().path {
            sysfs::cdc_path(path, "1.0")
        } else {
            None
        }
    }    

    
    fn openocd_serial(&self) -> Option<String> {
        Some(format!("jlink_serial {}", self.usb.serial_number))
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

    fn loader_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }

    fn debugger_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }    

    fn openocd_serial(&self) -> Option<String> {
        // see https://armprojects.wordpress.com/2016/08/21/debugging-multiple-stm32-in-eclipse-with-st-link-v2-and-openocd/
        // This assumes serial number is an 8-bit ASCII string that has been directly encoded as UTF-8
        // Additionally, assume that OpenOCD will replace non-ASCII characters with a question mark.

        let serial = self.usb.serial_number.clone().into_bytes();
        let mut out = String::from("hla_serial \"");

        for c in self.usb.serial_number.chars() {
            let c = c as u8;
            let b = if c > 0x7f {
                0x3f
            } else {
                c
            };
            write!(out, "\\x{:02X}", b).unwrap();
        }
        write!(out,"\"").unwrap();
        Some(out)
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

    fn loader_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }    

    fn debugger_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }    

    #[cfg(target_os="macos")]
    fn cdc_path(&self) -> Option<String> {
        Some(format!("/dev/cu.usbmodem{}{}", 
            format!("{:x}", self.usb.location_id.unwrap_or(0)).replace("0",""),
            3,
        ))
    }    

    #[cfg(target_os="linux")]
    fn cdc_path(&self) -> Option<String> {
        if let Some(ref path) = self.usb().path {
            sysfs::cdc_path(path, "1.2")
        } else {
            None
        }
    }    

    fn openocd_serial(&self) -> Option<String> {
        Some(format!("hla_serial {}", self.usb.serial_number))
    }        

    fn can_trace_itm(&self) -> bool { true }

    #[allow(unreachable_code)]
    fn trace_itm(&self) -> Result<()> {
        use std::time::Duration;
        use std::thread;
        use std::io::{self, Write};
        
        let mut ctx = stlink::context()?;
        let cfg = stlink::Config::new(&self.usb.serial_number);
        if let Some(mut d) = ctx.connect(cfg)? {
            d.configure(false)?;
            let mode = match d.mode() {
                Ok(mode) => mode,
                Err(_) => {
                    d.reinit()?;
                    d.configure(true)?;
                    d.mode()?
                }
            };        
            if mode == stlink::Mode::Dfu {
                d.exit_dfu_mode()?;
            }
            if mode != stlink::Mode::Debug {
                d.enter_swd_mode()?;
            }

            if mode != stlink::Mode::Debug {
                bail!("Could not enter Debug mode");
            }  

            d.halt()?;

            d.trace_setup(0xffffffff, 0, 168_000_000, 2_000_000)?;
            d.trace_start_rx(2_000_000)?;
            d.run()?;

            let mut trace_buf = [0u8; 4096];
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            loop {            
                let n = d.trace_read(&mut trace_buf)?;
                if n > 0 {
                    let mut r = stlink::Reader::new(&trace_buf[..n]);
                    while let Some((port, data)) = r.next() {                    
                        if port == 0 {
                            stdout.write_all(data)?
                        }
                    }
                    stdout.flush()?;
                }
                thread::sleep(Duration::from_millis(10));
            }

        } else {
            bail!("No device found");
        }        
        unreachable!()
    }
}

pub struct TiIcdiDevice {
    usb: UsbDevice,
}

impl Device for TiIcdiDevice {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("TI-ICDI")
    }

    fn loader_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }    

    fn debugger_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }    

    #[cfg(target_os="macos")]
    fn cdc_path(&self) -> Option<String> {
        Some(format!("/dev/cu.usbmodem{}{}", &self.usb.serial_number[..7], 1))
    }    

    #[cfg(target_os="linux")]
    fn cdc_path(&self) -> Option<String> {
        if let Some(ref path) = self.usb().path {
            sysfs::cdc_path(path, "1.0")
        } else {
            None
        }
    }    

    fn openocd_serial(&self) -> Option<String> {
        Some(format!("hla_serial {}", self.usb.serial_number))
    }        
}

pub struct CmsisDapDevice {
    usb: UsbDevice,
}

impl Device for CmsisDapDevice {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("DAPLink")
    }

    fn loader_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }

    fn debugger_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }    

    #[cfg(target_os="macos")]
    fn cdc_path(&self) -> Option<String> {
        Some(format!("/dev/cu.usbmodem{}{}", 
            format!("{:x}", self.usb.location_id.unwrap_or(0)).replace("0",""),
            2,
        ))
    }

    #[cfg(target_os="linux")]
    fn cdc_path(&self) -> Option<String> {
        if let Some(ref path) = self.usb().path {
            sysfs::cdc_path(path, "1.1")
        } else {
            None
        }
    }    

    
    fn openocd_serial(&self) -> Option<String> {
        Some(format!("cmsis_dap_serial {}", self.usb.serial_number))
    }       
}

pub struct DapLinkDevice {
    usb: UsbDevice,
}

impl Device for DapLinkDevice {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("DAPLink")
    }

    fn loader_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }
    
    fn debugger_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }    

    #[cfg(target_os="macos")]
    fn cdc_path(&self) -> Option<String> {
        Some(format!("/dev/cu.usbmodem{}{}", 
            format!("{:x}", self.usb.location_id.unwrap_or(0)).replace("0",""),
            2,
        ))
    }

    #[cfg(target_os="linux")]
    fn cdc_path(&self) -> Option<String> {
        if let Some(ref path) = self.usb().path {
            sysfs::cdc_path(path, "1.1")
        } else {
            None
        }
    }    

    fn msd_path(&self) -> Option<PathBuf> {
        // Look in /Volumes/DAPLINK*/ for DETAILS.TXT
        // Look for Unique ID line == serial number
        if let Ok(volumes) = fs::read_dir("/Volumes/") {
            for volume in volumes {                
                if let Ok(volume) = volume {                    
                    //println!("checking {:?} {}", volume.path(), volume.path().to_string_lossy().starts_with("/Volumes/DAPLINK") );
                    if volume.path().to_string_lossy().starts_with("/Volumes/DAPLINK") {                        
                        let details = volume.path().join("DETAILS.TXT");
                        let mut f = fs::File::open(details).expect("Error opening DETAILS.TXT");
                        let mut s = String::new();
                        f.read_to_string(&mut s).expect("Error reading details");
                        if s.contains(&self.usb.serial_number) {
                            return Some(volume.path())
                        }                        
                    }
                }
            }
        }
        None
    }

    fn openocd_serial(&self) -> Option<String> {
        Some(format!("cmsis_dap_serial {}", self.usb.serial_number))
    }       
}

pub struct FeatherDevice {
    usb: UsbDevice,
}

impl Device for FeatherDevice {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("Feather")
    }

    fn loader_type(&self) -> Option<&str> {
        Some("Bossa")
    }

    #[cfg(target_os="macos")]
    fn bossa_path(&self) -> Option<String> {
        Some(format!("/dev/cu.usbmodem{}{}", 
            format!("{:x}", self.usb.location_id.unwrap_or(0)).replace("0",""),
            1,
        ))
    }    
}

pub struct TeensyDevice {
    usb: UsbDevice,
}

impl Device for TeensyDevice {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("Teensy")
    }

    fn loader_type(&self) -> Option<&str> {
        Some("Teensy")
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

pub fn filter(cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches) -> DeviceFilter {        
    let device = if let Some(d) = args.value_of("device") {
        Some(String::from(d))
    } else {
        if let Some(default_filter) = cfg.default_filter() {
            if let Some(default_filter) = default_filter.as_table() {
                if let Some(device) = default_filter.get("device") {
                    if let Some(device) = device.as_str() {
                        Some(String::from(device))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    DeviceFilter {
        all: args.is_present("all"),
        device: device,
    }    
}

pub fn lookup(usb: UsbDevice) -> Box<Device> {
    match (usb.vendor_id, usb.product_id) {
        (0x0d28, 0x0204) => Box::new(DapLinkDevice { usb: usb }),
        (0x03eb, 0x2157) => Box::new(CmsisDapDevice { usb: usb }),
        (0x0483, 0x3748) => Box::new(StLinkV2Device { usb: usb }),
        (0x0483, 0x374b) => Box::new(StLinkV21Device { usb: usb }),
        (0x1366, 0x0101) => Box::new(JLinkDevice { usb: usb }),
        (0x1366, 0x0105) => Box::new(JLinkDevice { usb: usb }),
        (0x1cbe, 0x00fd) => Box::new(TiIcdiDevice { usb: usb }),
        (0x239a, 0x800b) => Box::new(FeatherDevice { usb: usb }),
        (0x239a, 0x000b) => Box::new(FeatherDevice { usb: usb }),
        (0x16c0, 0x0486) => Box::new(TeensyDevice { usb: usb }),
        (0x16c0, 0x0478) => Box::new(TeensyDevice { usb: usb }),
        _ => Box::new(UnknownDevice { usb: usb })
    }
}


pub fn enumerate() -> Result<Vec<Box<Device>>> {    
    #[cfg(target_os="macos")]
    return Ok(ioreg::enumerate()?.into_iter().map(lookup).collect());
    
    #[cfg(target_os="linux")]
    return Ok(sysfs::enumerate()?.into_iter().map(lookup).collect());
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
