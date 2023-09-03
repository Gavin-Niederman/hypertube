use std::ffi::CString;

pub struct Config {
    pub name: Option<CString>,
    pub num_queues: Option<usize>,
    pub(crate) multi_queue: bool,
    pub no_pi: bool,
    pub address: Option<std::net::IpAddr>,
    pub netmask: Option<cidr::IpCidr>,
}

impl Config {
    /// Creates a new Config.
    /// It is reccommended to use ConfigBuilder instead.
    pub fn new(
        name: Option<CString>,
        num_queues: Option<usize>,
        no_pi: bool,
        address: Option<std::net::IpAddr>,
        netmask: Option<cidr::IpCidr>,
    ) -> Self {
        Self {
            name,
            num_queues,
            multi_queue: num_queues.unwrap_or(1) > 1,
            no_pi,
            address,
            netmask,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: None,
            num_queues: None,
            multi_queue: false,
            no_pi: true,
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

    pub fn with_pi(mut self, pi: bool) -> Self {
        self.config.no_pi = !pi;
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

    pub fn with_num_queues(mut self, num_queues: usize) -> Self {
        if num_queues < 1 {
            panic!("number of queues must be at least 1")
        }
        self.config.num_queues = Some(num_queues);
        self.config.multi_queue = if num_queues > 1 { true } else { false };
        self
    }
}
