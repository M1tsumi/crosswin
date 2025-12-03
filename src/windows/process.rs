use crate::error::Result;
use crate::windows::handles::Handle;

#[derive(Debug)]
pub struct Process {
    handle: Option<Handle>,
}

impl Process {
    pub fn new_current() -> Result<Self> {
        Ok(Process { handle: None })
    }

    pub fn handle(&self) -> Option<Handle> {
        self.handle
    }
}
