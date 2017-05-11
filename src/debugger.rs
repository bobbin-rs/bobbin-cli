use Result;

pub fn debugger(debugger_type: &str) -> Option<Box<Control>> {
    match debugger_type.to_lowercase().as_ref() {
        "openocd" => Some(Box::new(OpenOcdDebugger {})),
        "jlink" => Some(Box::new(JLinkDebugger {})),
        _ => None
    }
}

pub trait Control {
    fn halt(&self) -> Result<()>;
    fn resume(&self) -> Result<()>;
    fn reset_halt(&self) -> Result<()>;
    fn reset_run(&self) -> Result<()>;
    fn reset_init(&self) -> Result<()>;
}

pub struct OpenOcdDebugger {}

impl Control for OpenOcdDebugger {
    fn halt(&self) -> Result<()> {
        unimplemented!()
    }
    fn resume(&self) -> Result<()> {
        unimplemented!()
    }
    fn reset_halt(&self) -> Result<()> {
        unimplemented!()
    }
    fn reset_run(&self) -> Result<()> {
        unimplemented!()
    }
    fn reset_init(&self) -> Result<()> {
        unimplemented!()
    }
}


pub struct JLinkDebugger {}

impl Control for JLinkDebugger {
    fn halt(&self) -> Result<()> {
        unimplemented!()
    }
    fn resume(&self) -> Result<()> {
        unimplemented!()
    }
    fn reset_halt(&self) -> Result<()> {
        unimplemented!()
    }
    fn reset_run(&self) -> Result<()> {
        unimplemented!()
    }
    fn reset_init(&self) -> Result<()> {
        unimplemented!()
    }
}