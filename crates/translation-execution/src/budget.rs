//! Shared byte accounting for bounded transport and retained observations.
use anyhow::{Result, ensure};

#[derive(Clone, Debug)]
pub struct ByteBudget {
    limit: u64,
    used: u64,
}

impl ByteBudget {
    pub fn new(limit: u64) -> Self {
        Self { limit, used: 0 }
    }

    pub fn reserve(&mut self, bytes: u64) -> Result<()> {
        let used = self.used.checked_add(bytes).ok_or_else(|| anyhow::anyhow!("byte budget overflow"))?;
        ensure!(used <= self.limit, "aggregate byte budget exceeded: {used} > {}", self.limit);
        self.used = used;
        Ok(())
    }

    pub fn release(&mut self, bytes: u64) -> Result<()> {
        self.used = self.used.checked_sub(bytes).ok_or_else(|| anyhow::anyhow!("invalid byte budget release"))?;
        Ok(())
    }

    pub fn used(&self) -> u64 {
        self.used
    }
    pub fn remaining(&self) -> u64 {
        self.limit - self.used
    }
}
