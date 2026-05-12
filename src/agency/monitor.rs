#[derive(Debug, Clone, Default)]
pub struct Monitor {
    pub failures: Vec<String>,
}

impl Monitor {
    pub fn record_failure(&mut self, failure: impl Into<String>) {
        self.failures.push(failure.into());
    }
}
