use sha1;
#[cfg(target_os = "macos")]
use ioreg;
#[cfg(target_os = "linux")]
use sysfs;
use clap::ArgMatches;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;
use std::fmt::Write;
use config::Config;
#[cfg(feature = "stlink")]
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
    fn is_unknown(&self) -> bool {
        self.device_type().is_none()
    }
    fn device_type(&self) -> Option<&str> {
        None
    }
    fn loader_type(&self) -> Option<&str> {
        None
    }
    fn debugger_type(&self) -> Option<&str> {
        None
    }
    fn cdc_path(&self) -> Option<String> {
        None
    }
    fn msd_path(&self) -> Option<PathBuf> {
        None
    }
    fn bossa_path(&self) -> Option<String> {
        None
    }
    fn gdb_path(&self) -> Option<String> {
        None
    }    

    fn jlink_supported(&self) -> bool {
        self.device_type() == Some("JLink")
    }

    fn openocd_supported(&self) -> bool {
        self.openocd_serial().is_some()
    }
    fn openocd_serial(&self) -> Option<String> {
        None
    }

    fn can_trace_itm(&self) -> bool {
        false
    }
    fn trace_itm(&self, target_clk: u32, trace_clk: u32) -> Result<()> {
        unimplemented!()
    }
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

    #[cfg(target_os = "macos")]
    fn cdc_path(&self) -> Option<String> {
        let path = format!("/dev/cu.usbmodem{}{}",
            &format!("{:x}", self.usb.location_id.unwrap_or(0))[..4],
            1,
        );
        if Path::new(&path).exists() {
            Some(path)
        } else {
            None
        }
    }

    #[cfg(target_os = "linux")]
    fn cdc_path(&self) -> Option<String> {
        if let Some(ref path) = self.usb().path {
            if let Some(cdc_path) = sysfs::cdc_path(path, "1.0") {
                if Path::new(&cdc_path).exists() {
                    Some(cdc_path)
                } else {
                    None
                }
            } else {
                None
            }
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
            let b = if c > 0x7f { 0x3f } else { c };
            write!(out, "\\x{:02X}", b).unwrap();
        }
        write!(out, "\"").unwrap();
        Some(out)
    }

    #[cfg(feature = "stlink")]
    fn can_trace_itm(&self) -> bool {
        true
    }

    #[allow(unreachable_code)]
    #[cfg(feature = "stlink")]
    fn trace_itm(&self, target_clk: u32, trace_clk: u32) -> Result<()> {

        let mut ctx = stlink::context()?;
        let cfg = stlink::Config::new(
            self.usb.vendor_id,
            self.usb.product_id,
            0x2,
            0x81,
            0x83,
            target_clk,
            trace_clk,
            &self.usb.serial_number,
        );
        if let Some(mut d) = ctx.connect(cfg)? {
            println!("configure");
            d.configure(false)?;
            println!("run trace");
            d.run_trace()?;

        } else {
            bail!("No device found");
        }
        unreachable!()
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

    #[cfg(target_os = "macos")]
    fn cdc_path(&self) -> Option<String> {       
        Some(format!("/dev/cu.usbmodem{}{}",
            &format!("{:x}", self.usb.location_id.unwrap_or(0))[..4],
            3,
        ))            
    }

    #[cfg(target_os = "linux")]
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

    #[cfg(feature = "stlink")]
    fn can_trace_itm(&self) -> bool {
        true
    }

    #[cfg(feature = "stlink")]
    #[allow(unreachable_code)]
    fn trace_itm(&self, target_clk: u32, trace_clk: u32) -> Result<()> {
        let mut ctx = stlink::context()?;
        let cfg = stlink::Config::new(
            self.usb.vendor_id,
            self.usb.product_id,
            0x1,
            0x81,
            0x82,
            target_clk,
            trace_clk,
            &self.usb.serial_number,
        );
        if let Some(mut d) = ctx.connect(cfg)? {
            d.configure(false)?;
            d.run_trace()?;

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

    #[cfg(target_os = "macos")]
    fn cdc_path(&self) -> Option<String> {
        Some(format!(
            "/dev/cu.usbmodem{}{}",
            &self.usb.serial_number[..7],
            1
        ))
    }

    #[cfg(target_os = "linux")]
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

    #[cfg(target_os = "macos")]
    fn cdc_path(&self) -> Option<String> {
        Some(format!("/dev/cu.usbmodem{}{}",
            &format!("{:x}", self.usb.location_id.unwrap_or(0))[..4],
            2,
        ))
    }

    #[cfg(target_os = "linux")]
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

    #[cfg(target_os = "macos")]
    fn cdc_path(&self) -> Option<String> {
        Some(format!("/dev/cu.usbmodem{}{}",
            &format!("{:x}", self.usb.location_id.unwrap_or(0))[..4],
            2,
        ))
    }

    #[cfg(target_os = "linux")]
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
                    if volume.path().to_string_lossy().starts_with(
                        "/Volumes/DAPLINK",
                    )
                    {
                        let details = volume.path().join("DETAILS.TXT");
                        let mut f = fs::File::open(details).expect("Error opening DETAILS.TXT");
                        let mut s = String::new();
                        f.read_to_string(&mut s).expect("Error reading details");
                        if s.contains(&self.usb.serial_number) {
                            return Some(volume.path());
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

    #[cfg(target_os = "macos")]
    fn bossa_path(&self) -> Option<String> {
        // eprintln!("location id: 0x{:0x}", self.usb.location_id.unwrap());
        let loc = format!("{:x}", self.usb.location_id.unwrap_or(0));
        let loc = loc.split("0").next().unwrap();
        if os_version_match("10.14") {
            Some(format!("/dev/cu.usbmodem{}01", loc))
        } else {
            Some(format!("/dev/cu.usbmodem{}{}",
                &format!("{:x}", self.usb.location_id.unwrap_or(0))[..4],
                1,
            ))
        }
    }
}

pub struct TeensyDevice {
    usb: UsbDevice,
}

fn os_version_match(required_version: &str) -> bool {
    let os = os_type::current_platform();
    let os_version = semver::Version::parse(&os.version).unwrap();

    let ver_str = ">= ".to_string() + required_version;
    let r = semver::VersionReq::parse(&ver_str).unwrap();

    r.matches(&os_version)
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

pub struct Stm32Device {
    usb: UsbDevice,
}

impl Device for Stm32Device {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("STM32")
    }

    fn loader_type(&self) -> Option<&str> {
        Some("dfu-util")
    }
}

pub struct BlackMagicDevice {
    usb: UsbDevice,
}

impl Device for BlackMagicDevice {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("BlackMagicProbe")
    }

    fn loader_type(&self) -> Option<&str> {
        Some("blackmagic")
    }

    fn debugger_type(&self) -> Option<&str> {
        Some("blackmagic")
    }

    #[cfg(target_os = "macos")]
    fn cdc_path(&self) -> Option<String> {

        // Since Macos 10.14 the usbmodem serial number behaviour has changed
        // instead of replacing the last character with 1 or 3, it is actually
        // added after the last character
        if os_version_match("10.14") {
            Some(format!("/dev/cu.usbmodem{}3", &self.usb.serial_number))
        } else {
            let serial_len = self.usb.serial_number.len();
            Some(format!("/dev/cu.usbmodem{}3", &self.usb.serial_number[..serial_len  - 1]))
        }
    }

    #[cfg(target_os = "linux")]
    fn cdc_path(&self) -> Option<String> {
        if let Some(ref path) = self.usb().path {
            sysfs::cdc_path(path, "1.2")
        } else {
            None
        }
    }

    #[cfg(target_os = "macos")]
    fn gdb_path(&self) -> Option<String> {

        // Since Macos 10.14 the cu.usbmodem serial number behaviour has changed
        // instead of replacing the last character with 1 or 3, it is actually
        // added after the last character
        if os_version_match("10.14") {
            Some(format!("/dev/cu.usbmodem{}1", &self.usb.serial_number))
        } else {
            let serial_len = self.usb.serial_number.len();
            Some(format!("/dev/cu.usbmodem{}1", &self.usb.serial_number[..serial_len  - 1]))
        }
    }

    #[cfg(target_os = "linux")]
    fn gdb_path(&self) -> Option<String> {
        if let Some(ref path) = self.usb().path {
            sysfs::cdc_path(path, "1.0")
        } else {
            None
        }
    }
}

pub struct Xds110Device {
    usb: UsbDevice,
}

impl Device for Xds110Device {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("XDS110")
    }

    fn loader_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }

    fn debugger_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }

    #[cfg(target_os = "macos")]
    fn cdc_path(&self) -> Option<String> {
        Some(format!(
            "/dev/cu.usbmodem{}{}",
            &self.usb.serial_number[..7],
            4
        ))
    }

    #[cfg(target_os = "linux")]
    fn cdc_path(&self) -> Option<String> {
        if let Some(ref path) = self.usb().path {
            sysfs::cdc_path(path, "1.0")
        } else {
            None
        }
    }

    fn openocd_serial(&self) -> Option<String> {
        Some(format!("cmsis_dap_serial {}", self.usb.serial_number))
    }
}

pub struct OlimexDevice {
    usb: UsbDevice,
}

impl Device for OlimexDevice {
    fn usb(&self) -> &UsbDevice {
        &self.usb
    }

    fn device_type(&self) -> Option<&str> {
        Some("Olimex")
    }

    fn loader_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }

    fn debugger_type(&self) -> Option<&str> {
        Some("OpenOCD")
    }

    fn openocd_serial(&self) -> Option<String> {
        Some(format!("ftdi_serial {}", self.usb.serial_number))
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
            device: other.value_of("device").map(String::from),
        }
    }
}

pub fn filter(cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches) -> DeviceFilter {
    let device = if let Some(d) = args.value_of("device") {
        Some(String::from(d))
    } else if let Some(d) = cfg.filter_device() {
        Some(String::from(d))
    } else {
        None
    };

    DeviceFilter {
        all: args.is_present("all") || cmd_args.is_present("all"),
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
        (0x0451, 0xbef3) => Box::new(Xds110Device { usb: usb }),
        (0x16c0, 0x0486) => Box::new(TeensyDevice { usb: usb }),
        (0x16c0, 0x0478) => Box::new(TeensyDevice { usb: usb }),
        (0x0483, 0xdf11) => Box::new(Stm32Device { usb: usb }),
        (0x1d50, 0x6018) => Box::new(BlackMagicDevice { usb: usb }),
        (0x15ba, 0x002a) => Box::new(OlimexDevice { usb: usb }),
        // Vendor Prefix Only
        (0x1366, _) => Box::new(JLinkDevice { usb: usb }),        
        // Assume all Adafruit devices are FeatherDevices, which use
        // the BOSSA loader.
        (0x239a, _) => Box::new(FeatherDevice { usb: usb }),
        _ => Box::new(UnknownDevice { usb: usb }),
    }
}


pub fn enumerate() -> Result<Vec<Box<Device>>> {
    #[cfg(target_os = "macos")] return Ok(ioreg::enumerate()?.into_iter().map(lookup).collect());

    #[cfg(target_os = "linux")] return Ok(sysfs::enumerate()?.into_iter().map(lookup).collect());
}

pub fn search(filter: &DeviceFilter) -> Result<Vec<Box<Device>>> {
    Ok(
        enumerate()?
            .into_iter()
            .filter(|d| {
                if !filter.all {
                    if d.is_unknown() {
                        return false;
                    }
                }

                if let Some(ref device) = filter.device {
                    if !filter.all && !d.hash().starts_with(device) {
                        return false;
                    }
                }


                true
            })
            .collect(),
    )
}
