//! Configuration of the emulator

/// Whether to log things such as opcodes being executed
#[derive(Debug)]
pub enum Log {
    #[allow(missing_docs)]
    Enabled,
    #[allow(missing_docs)]
    Disabled,
}

impl Log {
    /// Returns whether logging is enabled
    pub fn is_enabled(&self) -> bool {
        if let &Log::Enabled = self {
            true
        } else {
            false
        }
    }
}

impl From<bool> for Log {
    fn from(val: bool) -> Self {
        if val { Log::Enabled } else { Log::Disabled }
    }
}
