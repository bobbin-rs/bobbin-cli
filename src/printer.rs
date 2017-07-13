use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};


pub fn printer() -> Printer {
    Printer {
        out: StandardStream::stdout(ColorChoice::Always),
        verbose: false,
    }
}

pub struct Printer {
    out: StandardStream,
    verbose: bool,
}

impl Printer {
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn with_verbose(self, value: bool) -> Self {
        Printer {
            out: self.out,
            verbose: value,
        }
    }

    pub fn out(&mut self) -> &mut StandardStream {
        &mut self.out
    }

    pub fn msg(&mut self, color: Color, label: &str, msg: &str) -> ::std::io::Result<()> {
        self.out.set_color(
            ColorSpec::new().set_bold(true).set_fg(Some(color)),
        )?;
        write!(self.out, "{:>12}", label)?;
        self.out.reset()?;
        writeln!(self.out(), " {}", msg)
    }

    pub fn verbose(&mut self, label: &str, msg: &str) -> ::std::io::Result<()> {
        if self.verbose {
            self.msg(Color::White, label, msg)
        } else {
            Ok(())
        }
    }

    pub fn info(&mut self, label: &str, msg: &str) -> ::std::io::Result<()> {
        self.msg(Color::Green, label, msg)
    }

    pub fn error(&mut self, label: &str, msg: &str) -> ::std::io::Result<()> {
        self.msg(Color::Red, label, msg)
    }
}

impl Write for Printer {
    fn write(&mut self, buf: &[u8]) -> ::std::io::Result<usize> {
        self.out.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}
