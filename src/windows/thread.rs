use crate::error::Result;
use crate::windows::handles::Handle;

#[derive(Debug)]
pub struct Thread {
    handle: Option<Handle>,
}

impl Thread {
    pub fn current() -> Result<Self> {
        Ok(Thread { handle: None })
    }

    pub fn handle(&self) -> Option<Handle> {
        self.handle
    }
}
