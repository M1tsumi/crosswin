#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawHandle(pub isize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle {
    raw: RawHandle,
}

impl Handle {
    pub fn from_raw(raw: isize) -> Self {
        Handle {
            raw: RawHandle(raw),
        }
    }

    pub fn raw(&self) -> RawHandle {
        self.raw
    }
}
