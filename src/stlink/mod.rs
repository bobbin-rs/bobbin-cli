mod util;
mod constants;
mod chip_ids;

use libusb;
pub use self::constants::*;


use byteorder::{ByteOrder, LittleEndian};

use std::time::Duration;
use std::thread;
use std::convert::{AsRef, AsMut};

mod errors {
    error_chain!{
        foreign_links {
            LibUsb(::libusb::Error);
        }
    }
}

pub use errors::*;

#[derive(Debug, Clone)]
pub struct Config {
    serial_number: String,
}

impl Config {
    pub fn new(serial_number: &str) -> Self {
        Config { serial_number: String::from(serial_number) }
    }
}

pub struct Context {
    inner: libusb::Context,
    timeout: Duration,
}

impl Context {
    pub fn connect(&mut self, cfg: Config) -> Result<Option<Debugger>> {
        let timeout = self.timeout;
        //self.inner.set_log_level(libusb::LogLevel::Debug);
        for device in self.inner.devices()?.iter() {
            let device_desc = device.device_descriptor()?;
            let device_handle = device.open()?;
            let lang = device_handle.read_languages(timeout)?[0];

            if device_desc.serial_number_string_index().is_some() {
                if device_handle.read_serial_number_string(lang, &device_desc, timeout)? == cfg.serial_number {
                    return Ok(Some(Debugger {
                        config: cfg,
                        timeout: self.timeout,
                        device: device,
                        desc: device_desc,
                        handle: device_handle,
                    }))

                }
            }            
        }
        Ok(None)
    }    
}

pub struct Debugger<'a> {
    config: Config,
    timeout: Duration,
    device: libusb::Device<'a>,
    desc: libusb::DeviceDescriptor,
    handle: libusb::DeviceHandle<'a>,    
}

impl<'a> Debugger<'a> {
    pub fn reinit(&mut self) -> Result<()> {
        println!("resetting device");
        self.handle.release_interface(0)?;
        self.handle.reset()?;
        thread::sleep(Duration::from_millis(1000));
        self.handle = self.device.open()?;
        Ok(())
    }

    pub fn configure(&mut self, force: bool) -> Result<()> {
        if self.handle.active_configuration()? != 1 || force {
            self.handle.set_active_configuration(1)?;
        }
        self.handle.claim_interface(0)?;
        // self.handle.set_alternate_setting(0, 0)?;
        Ok(())
    }

    pub fn dump(&mut self) -> Result<()> {
        let timeout = self.timeout;
        let lang = self.handle.read_languages(timeout)?[0];
        println!("Vendor ID:    {:04x}", self.desc.vendor_id());
        println!("Product ID:   {:04x}", self.desc.product_id());
        println!("Manufacturer: {}", self.handle.read_manufacturer_string(lang, &self.desc, timeout)?);
        println!("Product:      {}", self.handle.read_product_string(lang, &self.desc, timeout)?);
        println!("Serial #:     {}", self.handle.read_serial_number_string(lang, &self.desc, timeout)?);
        Ok(())
    }

    pub fn identify_core(&mut self) -> Result<Option<(&'static str, u32)>> {
        let id = self.core_id()?;
        for &(label, code) in chip_ids::CORE_IDS.iter() {
            if id == code {
                return Ok(Some((label, code)))
            }
        }
        return Ok(None)        
    }

    pub fn identify_chip(&mut self) -> Result<Option<(&'static str, u16)>> {
        let id = self.read_32(DBGMCU_IDCODE)?;
        let chip_id = (id & 0xfff) as u16;
        for &(label, code) in chip_ids::CHIP_IDS.iter() {
            if chip_id == code {
                return Ok(Some((label, code)))
            }
        }
        return Ok(None)
    }

    pub fn send(&self, src: &[u8]) -> Result<usize> {
        Ok(self.handle.write_bulk(0x01, src, self.timeout)?)
    }

    pub fn recv(&self, dst: &mut [u8]) -> Result<usize> {
        Ok(self.handle.read_bulk(0x81, dst, self.timeout)?)
    }

    pub fn trace_recv(&self, dst: &mut [u8]) -> Result<usize> {
        Ok(self.handle.read_bulk(0x82, dst, self.timeout)?)
    }    

    pub fn send_u32(&self, src: &[u32]) -> Result<usize> {
        self.send(util::u32_as_u8(src)).map(|v| v >> 2)
    }

    pub fn recv_u32(&self, dst: &mut [u32]) -> Result<usize> {
        self.recv(util::u32_as_u8_mut(dst)).map(|v| v >> 2)
    }

    pub fn send_req<A: AsRef<[u8]>>(&mut self, a: A) -> Result<()> {
        self.send(Request::new(a).as_ref())?;
        Ok(())
    }

    pub fn recv_res(&mut self, len: usize) -> Result<Response> {
        let mut res = Response::new(len);
        self.recv(res.as_mut())?;
        Ok(res)
    }

    pub fn xfer<A: AsRef<[u8]>>(&mut self, a: A, len: usize) -> Result<Response> {
        self.send_req(a)?;
        self.recv_res(len)
    }

    pub fn cmd<A: AsRef<[u8]>>(&mut self, a: A) -> Result<()> {
        self.send_req(a)?;
        self.recv_res(2)?;
        Ok(())
    }

    pub fn version(&mut self) -> Result<Version> {
        let mut buf = [0u8; 6];
        self.send_req([GET_VERSION])?;
        self.recv(&mut buf)?;
        Ok(Version(buf))
    }    

    pub fn voltage(&mut self) -> Result<f32> {
        let mut res = self.xfer([GET_TARGET_VOLTAGE], 8)?;
        
        let adc0 = res.read_u32();
        let adc1 = res.read_u32();

        let v = if adc0 > 0 {
            2.4 * (adc1 as f32) / (adc0 as f32)
        } else {
            0.0
        };

        Ok(v)
    }

    pub fn mode(&mut self) -> Result<Mode> {
        Ok(match self.xfer([GET_CURRENT_MODE], 2)?.read_u16() {
            0x00 => Mode::Dfu,
            0x01 => Mode::Mass,
            0x02 => Mode::Debug,
            0x03 => Mode::Swim,
            0x04 => Mode::Bootloader,
            m @ _ => bail!("Unrecognized Mode: 0x{:02x}", m),
        })
    }

    pub fn core_id(&mut self) -> Result<u32> {
        Ok(self.xfer([DEBUG_COMMAND, DEBUG_READCOREID], 4)?.read_u32())
    }     

    pub fn enter_swd_mode(&mut self) -> Result<()> {
        self.cmd([DEBUG_COMMAND, DEBUG_APIV2_ENTER, DEBUG_ENTER_SWD])
    }

    pub fn exit_dfu_mode(&mut self) -> Result<()> {
        self.send_req([DFU_COMMAND, DFU_EXIT])
    }

    pub fn reset(&mut self) -> Result<()> {
        self.cmd([DEBUG_COMMAND, DEBUG_APIV2_RESETSYS])
    }

    pub fn run(&mut self) -> Result<()> {
        self.cmd([DEBUG_COMMAND, DEBUG_RUNCORE])
    }

    pub fn halt(&mut self) -> Result<()> {
        self.cmd([DEBUG_COMMAND, DEBUG_FORCEDEBUG])
    }
    
    pub fn step(&mut self) -> Result<()> {
        self.cmd([DEBUG_COMMAND, DEBUG_STEPCORE])
    }
   

    pub fn read_regs(&mut self) -> Result<Response> {
        let cmd = Request::new([DEBUG_COMMAND, DEBUG_APIV2_READALLREGS]);
        let mut res = self.xfer(cmd, 88)?;
        let _ = res.read_u32();
        Ok(res)
    }

    pub fn read_reg(&mut self, reg: u8) -> Result<u32> {
        let cmd = Request::new([DEBUG_COMMAND, DEBUG_APIV2_READREG]).write_u8(reg);
        let mut res = self.xfer(cmd, 8)?;
        let _ = res.read_u32();
        Ok(res.read_u32())
    }

    pub fn write_reg(&mut self, reg: u8, value: u32) -> Result<()> {
        let cmd = Request::new([DEBUG_COMMAND, DEBUG_APIV2_WRITEREG]).write_u8(reg).write_u32(value);
        let mut res = self.xfer(cmd, 2)?;
        let _ = res.read_u16();
        Ok(())
    }    

    pub fn check_rw_status(&mut self) -> Result<()> {
        self.cmd([DEBUG_COMMAND, DEBUG_APIV2_GETLASTRWSTATUS])
    }

    pub fn read_mem8(&mut self, addr: u32, dst: &mut [u8]) -> Result<()> {
        let cmd = Request::new([DEBUG_COMMAND, DEBUG_READMEM_8BIT])
            .write_u32(addr)
            .write_u16(dst.len() as u16);
        self.send_req(cmd)?;
        // Two bytes are returned if single byte is requested
        if dst.len() == 1 {
            let mut tmp = [0u8; 2];
            self.recv(&mut tmp)?;
            dst[0] = tmp[0];
        } else {
            self.recv(dst)?;
        }
        self.check_rw_status()
    }

    pub fn write_mem8(&mut self, addr: u32, src: &[u8]) -> Result<()> {
        let cmd = Request::new([DEBUG_COMMAND, DEBUG_WRITEMEM_8BIT])
            .write_u32(addr)
            .write_u16(src.len() as u16);
        self.send_req(cmd)?;
        self.send(src)?;
        self.check_rw_status()
    }


    pub fn read_mem32(&mut self, addr: u32, dst: &mut [u32]) -> Result<()> {
        let cmd = Request::new([DEBUG_COMMAND, DEBUG_READMEM_32BIT])
            .write_u32(addr)
            .write_u16((dst.len() * 4) as u16);
        self.send_req(cmd)?;
        self.recv_u32(dst)?;
        self.check_rw_status()
    }

    pub fn write_mem32(&mut self, addr: u32, src: &[u32]) -> Result<()> {
        let cmd = Request::new([DEBUG_COMMAND, DEBUG_WRITEMEM_32BIT])
            .write_u32(addr)
            .write_u16((src.len() * 4) as u16);
        self.send_req(cmd)?;
        self.send_u32(src)?;
        self.check_rw_status()
    }

    pub fn read_32(&mut self, addr: u32) -> Result<u32> {
        let mut tmp = [0u32];
        self.read_mem32(addr, &mut tmp)?;
        Ok(tmp[0])
    }

    pub fn write_32(&mut self, addr: u32, value: u32) -> Result<()> {
        self.write_mem32(addr, &[value])
    }

    pub fn read_mem(&mut self, addr: u32, dst: &mut [u8]) -> Result<()> {
        // TODO: Read chunks, add support for 32bit
        for i in 0..dst.len() {
            self.read_mem8(addr + i as u32, &mut dst[i..i+1])?;
        }
        Ok(())
    }

    pub fn write_mem(&mut self, addr: u32, src: &[u8]) -> Result<()> {
        // TODO: Write chunks, add support for 32bit
        for i in 0..src.len() {
            self.write_mem8(addr + i as u32, &src[i..i+1])?;
        }
        Ok(())
    }

    pub fn read_debug(&mut self, addr: u32) -> Result<u32> {
        let cmd = Request::new([DEBUG_COMMAND, DEBUG_APIV2_READDEBUGREG]).write_u32(addr);
        let mut res = self.xfer(cmd, 8)?;
        Ok(res.skip(4).read_u32())
    }

    pub fn write_debug(&mut self, addr: u32, value: u32) -> Result<()> {
        self.cmd(Request::new([DEBUG_COMMAND, DEBUG_APIV2_WRITEDEBUGREG]).write_u32(addr).write_u32(value))
    }

    pub fn trace_start_rx(&mut self, source_hz: u32) -> Result<()> {
        self.cmd(Request::new([DEBUG_COMMAND, DEBUG_APIV2_START_TRACE_RX])
            .write_u16(4096)
            .write_u32(source_hz))
    }

    pub fn trace_stop_rx(&mut self) -> Result<()> {
        self.cmd([DEBUG_COMMAND, DEBUG_APIV2_STOP_TRACE_RX])
    }    

    pub fn trace_bytes_available(&mut self) -> Result<u16> {
        Ok(self.xfer([DEBUG_COMMAND, DEBUG_APIV2_GET_TRACE_NB], 2)?.read_u16())
    }

    pub fn trace_set_swdclk(&mut self, divisor: u16) -> Result<()> {
        self.cmd(Request::new([DEBUG_COMMAND, DEBUG_APIV2_SWD_SET_FREQ]).write_u16(divisor))
    }    

    pub fn trace_read(&mut self, buf: &mut [u8]) -> Result<usize> {        
        let bytes_available = self.trace_bytes_available()?;
        let buf = &mut buf[..bytes_available as usize];
        if buf.len() > 0 {
            self.trace_recv(buf)
        } else {
            Ok(0)
        }
    }

    pub fn trace_setup(&mut self, stim_bits: u32, sync_packets: u32, cpu_hz: u32, swo_hz: u32) -> Result<()> {
        self.read_debug(DCB_DHCSR)?;
        self.write_debug(DCB_DEMCR, DCB_DEMCR_TRCENA)?;

        let mut reg = self.read_32(DBGMCU_CR)?;
        reg |= DBGMCU_CR_DEBUG_TRACE_IOEN | DBGMCU_CR_DEBUG_STOP | DBGMCU_CR_DEBUG_STANDBY | DBGMCU_CR_DEBUG_SLEEP;
        self.write_32(DBGMCU_CR, reg)?;
        // ST ref man says we set this to 1 even in async mode, it's still "one" pin wide
        self.write_32(TPIU_CSPSR, 1)?; // currently selelct parallel size register ==> 1 bit wide.
        let prescaler = (cpu_hz / swo_hz) - 1;
        self.write_32(TPIU_ACPR, prescaler)?; // async prescalar
        self.write_32(TPIU_SPPR, TPIU_SPPR_TXMODE_NRZ)?;
        self.write_32(TPIU_FFCR, 0)?; // Disable tpiu formatting
        self.write_32(ITM_LAR, SCS_LAR_KEY)?;
        self.write_32(ITM_TCR, ((1<<16) | ITM_TCR_SYNCENA | ITM_TCR_ITMENA))?;
        self.write_32(ITM_TER, stim_bits)?;
        self.write_32(ITM_TPR, stim_bits)?;
        self.set_dwt_sync_tap(sync_packets)?;
        Ok(())
    }

    pub fn set_dwt_sync_tap(&mut self, syncbits: u32) -> Result<()> {
        // Selects the position of the synchronization packet counter tap
        // on the CYCCNT counter. This determines the
        // Synchronization packet rate:
        // 00 = Disabled. No Synchronization packets.
        // 01 = Synchronization counter tap at CYCCNT[24]
        // 10 = Synchronization counter tap at CYCCNT[26]
        // 11 = Synchronization counter tap at CYCCNT[28]
        // For more information see The synchronization packet timer
        // on page C1-874.

        let mut reg = self.read_32(DWT_CTRL)?;
        reg &= !(3 << 10);
        reg |= (syncbits << 10) | 1; // Must have cyccnt to have cyccnt tap!
        self.write_32(DWT_CTRL, reg)?;
        Ok(())
    }
}

pub fn context() -> Result<Context> {
    Ok(Context {
        inner: libusb::Context::new()?,
        timeout: Duration::from_millis(100),
    })
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    Dfu = 0x0,
    Mass = 0x1,
    Debug = 0x2,
    Swim = 0x3,
    Bootloader = 0x4,
}

pub struct Version(pub [u8; 6]);

impl Version {
    pub fn version(&self) -> u16 {
        LittleEndian::read_u16(&self.0[..2])
    }

    pub fn stlink(&self) -> u8 {
        ((self.version() >> 12) & 0x0f) as u8
    }

    pub fn jtag(&self) -> u8 {
        ((self.version() >> 6) & 0x3f) as u8
    }

    pub fn swim(&self) -> u8 {
        (self.version() & 0x3f) as u8
    }

    pub fn vid(&self) -> u16 {
        LittleEndian::read_u16(&self.0[2..4])
    }

    pub fn pid(&self) -> u16 {
        LittleEndian::read_u16(&self.0[4..6])
    }   
}

pub struct Request {
    buf: [u8; 64],
    len: usize,
}

impl Request {
    pub fn new<A: AsRef<[u8]>>(a: A) -> Self {
        let mut buf = [0u8; 64];
        let arg = a.as_ref();
        &mut buf[..arg.len()].copy_from_slice(arg);
        Request { buf: buf, len: arg.len() }
    }

    pub fn write_u8(mut self, value: u8) -> Self {
        self.buf[self.len] = value;
        self.len += 1;
        self
    }

    pub fn write_u16(mut self, value: u16) -> Self {
        LittleEndian::write_u16(&mut self.buf[self.len..], value);
        self.len += 2;
        self
    }

    pub fn write_u32(mut self, value: u32) -> Self {
        LittleEndian::write_u32(&mut self.buf[self.len..], value);
        self.len += 4;
        self
    }
}

impl AsRef<[u8]> for Request {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..16]
    }
}

pub struct Response {
    buf: [u8; 128],
    cap: usize,
    pos: usize,
}

impl Response {
    pub fn new(cap: usize) -> Self {
        Response {
            buf: [0u8; 128],
            cap: cap,
            pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.cap - self.pos
    }    

    pub fn skip(&mut self, n: usize) -> &mut Self {
        self.pos += n;
        self
    }

    pub fn read_u8(&mut self) -> u8 {
        let v = self.buf[self.pos];
        self.pos += 1;
        v
    }

    pub fn read_u16(&mut self) -> u16 {
        let v = LittleEndian::read_u16(&self.buf[self.pos..]);
        self.pos += 2;
        v
    }

    pub fn read_u32(&mut self) -> u32 {
        let v = LittleEndian::read_u32(&self.buf[self.pos..]);
        self.pos += 4;
        v
    }

}

impl AsRef<[u8]> for Response {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.pos]
    }
}


impl AsMut<[u8]> for Response {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..self.cap]
    }
}


pub struct Reader<'a> { 
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &[u8]) -> Reader {
        Reader { buf: buf, pos: 0 }
    }

    pub fn next(&mut self) -> Option<(u8, &'a [u8])> {
        if self.pos >= self.buf.len() { return None }        
        let pos = self.pos;
        let header = self.buf[pos];
        let port = header >> 3;
        self.pos += 1;
        if self.pos >= self.buf.len() { return None }        
        //println!("{}", header & 0b111);
        match header & 0b111 {
            0b01 => {                
                self.pos += 1;
                if self.pos >= self.buf.len() { return None }        
                Some((port, &self.buf[pos+1..pos+2]))
            },
            0b10 => {
                self.pos += 2;
                if self.pos >= self.buf.len() { return None }        
                Some((port, &self.buf[pos+1..pos+3]))
            },
            0b11 => {
                self.pos += 4;
                if self.pos >= self.buf.len() { return None }
                Some((port, &self.buf[pos+1..pos+5]))
            },
            _ => None,
        }
    }
}