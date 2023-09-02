use std::ffi::CString;

pub struct Config {
    pub name: Option<CString>,
    pub num_queues: Option<usize>,
    pub no_pi: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: None,
            num_queues: None,
            no_pi: true,
        }
    }
}

pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn with_name(mut self, name: CString) -> Self {
        self.config.name = Some(name);
        self
    }

    pub fn with_num_queues(mut self, num_queues: usize) -> Self {
        self.config.num_queues = Some(num_queues);
        self
    }

    pub fn with_pi(mut self, pi: bool) -> Self {
        self.config.no_pi = !pi;
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}