use serial::{self, SerialPort};
use std::process;
use std::time::Duration;
use std::io::{Read, Write};
use std::str;
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
    Ok(Console{ port: port, test_filter: None })
}

pub struct Console {
    port: serial::SystemPort,
    test_filter: Option<String>
}

impl Console {
    pub fn test_filter(&self) -> &Option<String> {
        &self.test_filter
    }

    pub fn set_test_filter(&mut self, value: Option<String>) -> &Self {
        self.test_filter = value;
        self
    }

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
            let mut n = ::std::io::stdin().read(&mut buf).unwrap();
            if n > 0 { 
                // println!("read {}: {:?}", n, &buf[..n]);
                let n = self.port.write(&mut buf[..n]).unwrap();
                self.port.flush().unwrap();
                println!("Sent {}: {:?}", n, &buf[..n]);
                // let n = self.port.write(&mut buf[..n]).unwrap();
                // println!("sending null");
                // buf[0] = b'A';                
                // buf[1] = 0;
                // let n = self.port.write(&mut buf[..1]).unwrap();
                // println!("sent {}", n);
            }
            match self.port.read(&mut buf[..]) {
                Ok(n) => {
                    try!(out.write(&buf[..n]));
                },
                Err(_) => {},
            }
        }
        //Ok(())
    }

    pub fn view_packet(&mut self) -> Result<()> {
        self.port.set_timeout(Duration::from_millis(1000))?;
        
        let mut buf = [0u8; 1024];
        let mut b = cobs::Buffer::new(&mut buf);
        loop {
            match self.port.read(b.as_mut()) {
                Ok(n) => {
                    b.extend(n);
                    while let Some(packet) = b.next_packet() {
                        if packet.len() == 0 {
                            continue;
                        }
                        let mut msg_buf = [0u8; 1024];
                        match cobs::decode(packet, &mut msg_buf) {
                            Ok(n) => {
                                self.handle_packet(&msg_buf[..n])?;
                            },
                            Err(cobs::Error::SourceTooShort) => {
                                println!("Error:      Incomplete Packet")
                            },
                            Err(e) => {
                                println!("Error:      {:?}", e)
                            }
                        }

                        
                    }

                },
                Err(_) => {},
            }
            b.compact();
        }        
    }

    fn handle_packet(&mut self, packet: &[u8]) -> Result<()> {
        if packet.len() == 0 { return Ok(())}
        let msg = packet::decode_message(&packet).unwrap();
        self.handle_message(msg)
    }

    fn handle_message(&mut self, msg: Message) -> Result<()> {
        let mut out = ::std::io::stdout();
        let mut err = ::std::io::stderr();
        match msg {
            Message::Boot(value) => {
                write!(out, "Boot:       {}\r\n", String::from_utf8_lossy(value))?;
                if let Some(ref filter) = self.test_filter {
                    let msg = packet::Message::Run(b"test");
                    let mut tmp = [0u8; 256];
                    let pkt = packet::encode_message(&mut tmp, msg).unwrap();
                    let mut buf = [0u8; 256];
                    let mut w = cobs::Writer::new(&mut buf);
                    w.encode_packet(pkt).unwrap();
                    ::std::thread::sleep(Duration::from_millis(100));
                    println!("Sending {:?}", w.as_ref());
                    let n = self.port.write(w.as_ref()).unwrap();
                    println!("Sent {} bytes", n);

                }
            },
            Message::Stdout(ref value) => {
                out.write(value)?;
            },
            Message::Stderr(ref value) => {
                err.write(value)?;
            },
            Message::Exit(ref value) => {
                write!(err, "Exit:       {}\r\n", value[0])?;
                process::exit(value[0] as i32);
                
            },
            Message::Exception(ref value) => {
                write!(err, "Exception:  {}\r\n", String::from_utf8_lossy(value))?;            
            },
            Message::Panic(ref value) => {
                write!(err, "Panic:      {}\r\n", String::from_utf8_lossy(value))?;
            },            
            _ => {
                write!(err, "{:?}", msg)?;
            }
        }
        Ok(())
    }
}
