use std::ffi::CString;

pub struct Config {
    pub name: Option<CString>,
    pub num_queues: Option<usize>,
    pub no_pi: bool,
    pub blocking: bool,
    pub address: Option<std::net::IpAddr>,
    pub netmask: Option<cidr::IpCidr>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: None,
            num_queues: None,
            no_pi: true,
            blocking: true,
            address: None,
            netmask: None,
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

    pub fn with_blocking(mut self, blocking: bool) -> Self {
        self.config.blocking = blocking;
        self
    }

    pub fn with_address(mut self, address: std::net::IpAddr) -> Self {
        self.config.address = Some(address);
        self
    }

    pub fn with_netmask(mut self, netmask: cidr::IpCidr) -> Self {
        self.config.netmask = Some(netmask);
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}