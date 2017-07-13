use serial::{self, SerialPort};
use clap::ArgMatches;
use std::time::{Duration, Instant};
use std::io::{Read, Write};
use std::process;

use Result;

pub fn open(path: &str) -> Result<Console> {
    let mut port = try!(serial::open(path));
    try!(port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200).unwrap();
        settings.set_flow_control(serial::FlowControl::FlowNone);
        Ok(())
    }));
    Ok(Console { port: port })
}

pub struct Console {
    port: serial::SystemPort,
}

impl Console {
    pub fn clear(&mut self) -> Result<()> {
        let mut buf = [0u8; 1024];
        self.port.set_timeout(Duration::from_millis(10))?;
        loop {
            match self.port.read(&mut buf[..]) {
                Ok(0) => return Ok(()),
                Ok(_) => {}
                Err(_) => return Ok(()),
            }
        }
    }

    pub fn view(&mut self) -> Result<()> {
        self.port.set_timeout(Duration::from_millis(1000))?;
        let mut buf = [0u8; 1024];
        let mut out = ::std::io::stdout();
        loop {
            match self.port.read(&mut buf[..]) {
                Ok(n) => {
                    try!(out.write(&buf[..n]));
                }
                Err(_) => {}
            }
        }
        //Ok(())
    }

    pub fn test(&mut self, args: &ArgMatches, cmd_args: &ArgMatches) -> Result<()> {
        const LINE_TIMEOUT_MS: u64 = 5000;
        const TEST_TIMEOUT_MS: u64 = 15000;

        self.port.set_timeout(Duration::from_millis(100))?;
        let mut buf = [0u8; 1024];
        let mut line: Vec<u8> = Vec::new();
        let start_time: Instant = Instant::now();
        let mut line_time: Instant = start_time;
        loop {
            match self.port.read(&mut buf[..]) {
                Ok(n) => {
                    for b in (&buf[..n]).iter() {
                        if *b == b'\n' {
                            self.handle_line(line.as_ref())?;
                            line_time = Instant::now();
                            line.clear();
                        } else {
                            line.push(*b);
                        }
                    }
                }
                Err(_) => {}
            }
            let now = Instant::now();
            if now.duration_since(line_time) > Duration::from_millis(LINE_TIMEOUT_MS) {
                println!("[timeout:line]");
                process::exit(1);
            }
            if now.duration_since(start_time) > Duration::from_millis(TEST_TIMEOUT_MS) {
                println!("[timeout:test]");
                process::exit(1);
            }
        }
        //Ok(())
    }

    fn handle_line(&mut self, line: &[u8]) -> Result<()> {
        let mut out = ::std::io::stdout();
        let line_str = String::from_utf8_lossy(line);
        out.write(line_str.as_bytes())?;
        out.write(b"\n")?;
        out.flush()?;
        if line_str.starts_with("[done]") {
            process::exit(0);
        } else if line_str.starts_with("[fail]") {
            process::exit(1);
        } else if line_str.contains("[exception]") {
            process::exit(2);
        } else if line_str.contains("[panic]") {
            process::exit(3);
        }
        Ok(())
    }
}
