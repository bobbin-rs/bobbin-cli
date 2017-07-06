use serial::{self, SerialPort};
use std::process;
use std::time::Duration;
use std::io::{Read, Write};
//use sctl::{self, Message};
use cobs;
use packet::{self, Message};

use Result;

pub fn open(path: &str) -> Result<Console> {
    let mut port = try!(serial::open(path));
    try!(port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud115200).unwrap();
            Ok(())
    }));
    Ok(Console{ port: port })
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
                Ok(_) => {},
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
                },
                Err(_) => {},
            }
        }
        //Ok(())
    }

    pub fn view_sctl(&mut self) -> Result<()> {
        self.port.set_timeout(Duration::from_millis(1000))?;
        
        let mut buf = [0u8; 1024];
        let mut c = cobs::Reader::new(&mut buf);        
        loop {
            match self.port.read(c.as_mut()) {
                Ok(n) => {
                    c.extend(n);
                    loop {
                        let mut dst = [0u8; 1024];
                        match c.decode_packet(&mut dst) {
                            Ok(Some(n)) => {
                                self.handle_packet(&dst[..n])?;
                            },
                            Ok(None) => {
                                break;
                            },
                            Err(e) => {
                                println!("Error: {:?}", e);
                            }
                        }
                    }

                },
                Err(_) => {},
            }
            c.compact();
        }        
    }

    fn handle_packet(&mut self, packet: &[u8]) -> Result<()> {
        //println!("Packet: {:?}", packet);
        if packet.len() == 0 { return Ok(())}

        let msg = packet::decode_message(&packet).unwrap();
        self.handle_message(msg)

        // let mut r = sctl::Reader::new(packet);
        // loop {
        //     let mut tmp = [0u8; 1024];
        //     match r.read(&mut tmp) {
        //         Ok(Some(ref msg)) => {
        //             self.handle_message(msg)?;                    
        //         },
        //         Ok(None) => {
        //             break;
        //         }
        //         Err(e) => {
        //             println!("Error: {:?}", e);
        //             break;
        //         }
        //     }
        // }
        // Ok(())    
    }

    fn handle_message(&mut self, msg: Message) -> Result<()> {
        let mut out = ::std::io::stdout();
        match msg {
            Message::Boot(value) => {
                try!(out.write(b"boot: "));
                try!(out.write(value));
                try!(out.write(b"\r\n"));
            },
            Message::Stdout(ref value) => {
                try!(out.write(value));
            },
            Message::Stderr(ref value) => {
                try!(out.write(value));
            },
            Message::Exit(ref value) => {
                println!("Exit: {}", value[0]);
                process::exit(value[0] as i32);
                
            },
            _ => {
                println!("Message: {:?}", msg);
            }
        }
        Ok(())
    }

}
