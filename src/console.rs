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

fn write_bytes(port: &mut SerialPort, buf: &[u8]) -> Result<usize> {
    let mut n = 0;
    for chunk in buf.chunks(7) {
        ::std::thread::sleep(Duration::from_millis(1));
        n += port.write(&chunk)?
    }
    Ok(n)
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
            let n = ::std::io::stdin().read(&mut buf).unwrap();
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

    pub fn view_packet_5(&mut self) -> Result<()> {
        ::std::thread::sleep(Duration::from_millis(500));
        self.port.set_timeout(Duration::from_millis(100))?;
        let mut i = 0;
        let total = 0;
        let out = b"Hello, World";
        let mut buf = [0u8; 1024];        
        loop {
            ::std::thread::sleep(Duration::from_millis(500));            
            let msg = packet::Message::Stdin(b"hello, world");
            let mut tmp = [0u8; 1024];
            let pkt = packet::encode_message(&mut tmp, msg).unwrap();
            let mut pbuf = [0u8; 1024];
            let mut w = cobs::Writer::new(&mut pbuf);
            w.encode_packet(pkt).unwrap();
            let n = write_bytes(&mut self.port, w.as_ref())?;
            println!("Sent {} bytes", n);

            let x = match self.port.read(&mut buf) {
                Ok(x) => x,
                _ => 0,
            };
            println!("{}: {} {} in {} {:?}", i, n, total, x, &buf[..x]);
            i += 1;
        }
    }

    pub fn view_packet4(&mut self) -> Result<()> {
        ::std::thread::sleep(Duration::from_millis(500));
        self.port.set_timeout(Duration::from_millis(100))?;
        let mut i = 0;
        let mut total = 0;
        let out = b"Hello, World ABCDEFGH 12345678";
        let mut buf = [0u8; 1024];
        loop {
            let n = write_bytes(&mut self.port, &out[..])?;
            total += n;
            let x = match self.port.read(&mut buf) {
                Ok(x) => x,
                _ => 0,
            };
            println!("{}: {} {} in {} {}", i, n, total, x, String::from_utf8_lossy(&buf[..x]));
            i += 1;
        }
    }
    pub fn view_packet_3(&mut self) -> Result<()> {
        ::std::thread::sleep(Duration::from_millis(500));
        self.port.set_timeout(Duration::from_millis(100))?;
        let l_count = 1;
        let l_delay = 1;        
        let p_delay = 0;
        let out = [0u8; 4];
        let mut total = 0;
        loop {
            for i in 0..l_count {
                //::std::thread::sleep(Duration::from_millis(p_delay));
                match self.port.write(&out[..]) {
                    Ok(n) => println!("{}: {} {}", i, n, total),
                    Err(e) => {
                        println!("{}: {:?}", i, e);
                        bail!("write error")
                    },
                }
                total += out.len();
            }
            // println!("");
            ::std::thread::sleep(Duration::from_millis(l_delay));
        }
    }

    pub fn view_packet_2(&mut self) -> Result<()> {
        //self.port.set_timeout(Duration::from_millis(100))?;
        let mut i = 0;
        let delay = 0;
        let mut total = 0;
        ::std::thread::sleep(Duration::from_millis(500));
        loop {
            if i == 4 {
                println!("");
                ::std::thread::sleep(Duration::from_millis(500));                
                i = 0;
            }
            //println!("{}", i);
            let out = [0u8; 4];
            // println!("out: {} / {}{:?}", &out[..]);
            match write_bytes(&mut self.port, &out[..]) {
                Ok(n) => {
                    total += n;
                    println!("{}: {} / {} - {}", i, n, out.len(), total);
                },
                _ => {
                    println!("...");
                }
            }
            // ::std::thread::sleep(Duration::from_millis(delay));
            // let n = write_bytes(&mut self.port, b"b")?;
            // ::std::thread::sleep(Duration::from_millis(delay));
            i += 1;
        }
    }
    pub fn view_packet(&mut self) -> Result<()> {        
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

    // fn write(&mut self, buf: &[u8]) -> Result<usize> {
    //     for i in 0..buf.len() {
    //         while self.port.write(&buf[i..i+1])? != 1 {
    //             ::std::thread::sleep(Duration::from_millis(10));
    //         }
    //     }
    //     Ok(buf.len())
    // }

    fn handle_message(&mut self, msg: Message) -> Result<()> {
        let mut out = ::std::io::stdout();
        let mut err = ::std::io::stderr();
        match msg {
            Message::Boot(value) => {
                write!(out, "Boot:       {}\r\n", String::from_utf8_lossy(value))?;
                self.send_message(packet::Message::Run(b"test"))?;            
                ::std::thread::sleep(Duration::from_millis(500));    
                self.send_message(packet::Message::Stdin(b"hello, world!"))?;                
                // if let Some(ref filter) = self.test_filter {
                    // let buf = &[6, 2, 3, 116, 101, 115, 116, 0];
                    // let buf = &[6, 2, 3, 116, 101, 115, 115, 0];
                    // let buf = &[10, 10, 10, 10, 10, 10, 10, 10, 0]; // good 
                    // let buf = &[10, 10, 10, 10, 10, 10, 10, 0]; // bad
                    // let buf = &[10, 10, 10, 10, 10, 10, 0]; // good
                    // let buf = &[10, 10, 10, 10, 10, 10, 10, 10]; // bad
                    // let buf = &[1, 2, 3, 4, 5, 6, 7, 8]; // bad
                    // let buf = &[116, 101, 115, 116, 116, 101, 115, 116]; // bad
                    // println!("Sending {:?}", buf);
                    // // let n = self.port.write(buf).unwrap();
                    // let n = write_bytes(&mut self.port, buf)?;
                    // // for i in 0..buf.len() {
                    // //     while self.port.write(&buf[i..i+1])? != 1 {
                    // //         ::std::thread::sleep(Duration::from_millis(0));
                    // //     }
                    // // }                    
                    // println!("Sent {} bytes", n);


                    // // let msg = packet::Message::Stdin(b"tes");
                    // let msg = packet::Message::Run(b"tes");
                    // let mut tmp = [0u8; 256];
                    // let pkt = packet::encode_message(&mut tmp, msg).unwrap();
                    // let mut buf = [0u8; 256];
                    // let mut w = cobs::Writer::new(&mut buf);
                    // w.encode_packet(pkt).unwrap();
                    // //::std::thread::sleep(Duration::from_millis(0));
                    // println!("Sending {:?}", w.as_ref());
                    // let n = self.port.write(w.as_ref()).unwrap();
                    // println!("Sent {} bytes", n);


                    // let buf = b"Hello, World";
                    // println!("Sending {:?}", buf);
                    // let n = self.port.write(buf).unwrap();
                    // println!("Sent {} bytes", n);



                    // let mut tmp = [0u8; 256];
                    // let pkt = packet::encode_message(&mut tmp, msg).unwrap();
                    // let mut buf = [0u8; 256];
                    // let mut w = cobs::Writer::new(&mut buf);
                    // w.encode_packet(pkt).unwrap();
                    // // let n = self.port.write(w.as_ref()).unwrap();
                    // println!("Sending {:?}", w.as_ref());
                    // let n = write_bytes(&mut self.port, w.as_ref())?;
                    // println!("Sent {} bytes", n);

                // }
            },
            Message::Stdout(ref value) => {
                out.write(value)?;
                // if let Some(ref filter) = self.test_filter {
                //     let msg = packet::Message::Stdin(b"hello, world");
                //     let mut tmp = [0u8; 1024];
                //     let pkt = packet::encode_message(&mut tmp, msg).unwrap();
                //     let mut buf = [0u8; 1024];
                //     let mut w = cobs::Writer::new(&mut buf);
                //     w.encode_packet(pkt).unwrap();
                //     ::std::thread::sleep(Duration::from_millis(10));
                //     // let n = self.port.write(w.as_ref()).unwrap();
                //     println!("Sending {:?}", w.as_ref());
                //     let n = write_bytes(&mut self.port, w.as_ref())?;
                //     println!("Sent {} bytes", n);

                // }                
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
    pub fn send_message(&mut self, msg: packet::Message) -> Result<()> {
        let mut tmp = [0u8; 256];
        let pkt = packet::encode_message(&mut tmp, msg).unwrap();
        let mut buf = [0u8; 256];
        let mut w = cobs::Writer::new(&mut buf);
        w.encode_packet(pkt).unwrap();
        // let n = self.port.write(w.as_ref()).unwrap();
        println!("Sending {:?}", w.as_ref());
        let n = write_bytes(&mut self.port, w.as_ref())?;
        println!("Sent {} bytes", n);
        Ok(())
    }
}
